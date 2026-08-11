#!/usr/bin/env bash
# Deny destructive / multi-writer git that caused Bun-rewrite and console collisions.
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

# Integration owner / explicit SKIP escapes
if [[ "${CURSOR_ALLOW_GIT_DANGEROUS:-}" == "1" ]]; then
  allow
fi

# Patterns forbidden for lane workers (BASE_LOCK)
if printf '%s' "$CMD" | grep -Eq '(^|[[:space:]])git[[:space:]]+(stash|reset|clean)([[:space:]]|$)'; then
  deny "BASE_LOCK: git stash/reset/clean forbidden in lane worktrees (fix forward)."
fi
if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+checkout[[:space:]]+(-b|[[:alnum:]_./-]+)'; then
  # allow checkout -- <files>
  if ! printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+checkout[[:space:]]+--[[:space:]]'; then
    deny "BASE_LOCK: git checkout <branch> forbidden; stay on assigned worktree branch."
  fi
fi
# Lane workers must not rebase/merge. Integration restack onto origin/main is the
# approved path when main moves (never gh pr update-branch). Mid-restack
# continue/abort/skip must also pass or a conflict freezes the worktree.
if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+rebase[[:space:]]+origin/main([[:space:]]|$)'; then
  : # allow integration restack
elif printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+rebase[[:space:]]+(--continue|--abort|--skip)([[:space:]]|$)'; then
  : # allow finishing or abandoning an in-progress origin/main restack
elif printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+(rebase|merge)([[:space:]]|$)'; then
  deny "BASE_LOCK: git rebase/merge forbidden for lane workers (integration owner serializes). Restack only: git rebase origin/main"
fi
# Force-with-lease only after an integration restack onto origin/main (PR refresh).
# Plain --force stays forbidden.
if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push[[:space:]]+.*--force([^-]|$)'; then
  deny "BASE_LOCK: git push --force forbidden. Use --force-with-lease only after rebase onto origin/main."
fi
if printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+push[[:space:]]+.*--force-with-lease'; then
  : # allow PR tip refresh after rebase origin/main
fi
# Lane/admission provisioning is the programme's approved parallelism mechanism:
# a NEW worktree under <hub>/.worktrees/<name> on a NEW lane/* or admission/*
# branch cut from origin/main. Sibling paths like ../console-lane-* are denied —
# they sit outside the Cursor workspace root and trigger External-File Protection
# ("allow edit") prompts on every agent write.
# Everything else (remove, arbitrary add, update-branch) stays denied — including
# the `git -C <path> worktree` form the old pattern missed.
# Path must be `.worktrees/<name>` or absolute/relative ending in `/.worktrees/<name>`.
_WT_ADD_RE='git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?worktree[[:space:]]+add[[:space:]]+(\.worktrees/|[^[:space:]]*/\.worktrees/)[[:alnum:]_.-]+[[:space:]]+-b[[:space:]]+(lane|admission)/[[:alnum:]_./-]+[[:space:]]+origin/main([[:space:]]|$)'
if printf '%s' "$CMD" | grep -Eq "$_WT_ADD_RE"; then
  : # allow in-repo .worktrees/ lane provisioning from origin/main
elif printf '%s' "$CMD" | grep -Eq 'git[[:space:]]+(-C[[:space:]]+[^[:space:]]+[[:space:]]+)?worktree[[:space:]]+(add|remove)|gh[[:space:]]+pr[[:space:]]+update-branch'; then
  deny "BASE_LOCK: worktree add/remove outside in-repo provisioning (allowed: git [-C <hub>] worktree add <hub>/.worktrees/<name> -b lane/<id>|admission/<id> origin/main). Sibling ../console-lane-* paths are forbidden (Cursor External-File Protection). gh pr update-branch forbidden."
fi

allow
