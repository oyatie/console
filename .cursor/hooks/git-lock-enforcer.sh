#!/usr/bin/env bash
# Deny destructive / multi-writer git that caused Bun-rewrite and console collisions.
# Also deny cross-agent pkill/killall of git and unlock-by-rm (process.git-pkill-lock-race).
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

# ---------------------------------------------------------------------------
# process.git-pkill-lock-race — apply BEFORE CURSOR_ALLOW_GIT_DANGEROUS.
# Agents were unsticking hung shells with `pkill -f git` / `killall git` /
# `rm …/index.lock`, racing sibling writers on the shared hub .git.
# Escape only: CURSOR_ALLOW_GIT_PKILL=1 (human operator), or the allowlisted
# safe-stale helper below.
# ---------------------------------------------------------------------------
_PKILL_MSG='BASE_LOCK: process.git-pkill-lock-race — do not pkill/killall/kill-by-pattern git (or git hooks) across worktrees. Wait/timeout/flock; escalate. Stale locks: bash scripts/cursor/safe-stale-git-lock.sh <lock-path>. Escape: CURSOR_ALLOW_GIT_PKILL=1 (operator only).'
_LOCK_RM_MSG='BASE_LOCK: process.git-pkill-lock-race — do not rm git *.lock / gc.pid. Use bash scripts/cursor/safe-stale-git-lock.sh <lock-path> (PID dead + age gate). Escape: CURSOR_ALLOW_GIT_PKILL=1 (operator only).'

# Escape if env is set on the hook process OR assigned in the shell command
# (Cursor may not forward prefixed assignments into the hook environment).
if [[ "${CURSOR_ALLOW_GIT_PKILL:-}" != "1" ]] \
  && ! printf '%s' "$CMD" | grep -Eq '(^|[[:space:];|&])CURSOR_ALLOW_GIT_PKILL=1([[:space:];|&]|$)'; then
  # Allow the safe helper itself when invoked as a command (not nested after pkill).
  if printf '%s' "$CMD" | grep -Eq '(^|[[:space:];|&])(bash[[:space:]]+)?([^[:space:]]*/)?scripts/cursor/safe-stale-git-lock\.sh([[:space:]]|$)' \
    && ! printf '%s' "$CMD" | grep -Eq '(^|[[:space:];|&])(pkill|killall)([[:space:]]|$)'; then
    :
  else
    # Collapse newlines so multi-line agent shells still match.
    # Match pkill/killall/pgrep as COMMANDS (word boundaries), never as path substrings
    # like lane-console-pkill-lock-ban/.../git-lock-enforcer.sh.
    _FLAT="$(printf '%s' "$CMD" | tr '\n' ' ')"
    _HAS_PKILL=0
    _HAS_KILLALL=0
    _HAS_PGREP=0
    _HAS_KILL_CMD=0
    printf '%s' "$_FLAT" | grep -Eq '(^|[[:space:];|&])pkill([[:space:]]|$)' && _HAS_PKILL=1
    printf '%s' "$_FLAT" | grep -Eq '(^|[[:space:];|&])killall([[:space:]]|$)' && _HAS_KILLALL=1
    printf '%s' "$_FLAT" | grep -Eq '(^|[[:space:];|&])pgrep([[:space:]]|$)' && _HAS_PGREP=1
    printf '%s' "$_FLAT" | grep -Eq '(^|[[:space:];|&])kill([[:space:]]|$)' && _HAS_KILL_CMD=1

    _GIT_TARGET=0
    # Args / patterns that mean "aimed at git or shared console git state".
    if printf '%s' "$_FLAT" | grep -Eqi \
      '(^|[[:space:]-/'\''\"])git([[:space:]'\''\"/]|$)|git-lock|hooks/git|Cellar/git|/opt/homebrew[^;&|]*bin/git|admission-[A-Za-z0-9_-]+|/\.worktrees/'; then
      _GIT_TARGET=1
    fi

    if [[ "$_HAS_PKILL" -eq 1 || "$_HAS_KILLALL" -eq 1 ]]; then
      if [[ "$_GIT_TARGET" -eq 1 ]]; then
        deny "$_PKILL_MSG"
      fi
      # killall git / pkill git with bare name
      if printf '%s' "$_FLAT" | grep -Eqi '(^|[[:space:];|&])(pkill|killall)[[:space:]]+(-[A-Za-z0-9]+[[:space:]]+)*[`'\''\"]?git[`'\''\"]?([[:space:]]|$)'; then
        deny "$_PKILL_MSG"
      fi
    fi

    # pgrep … git piped/looped into kill — diagnostic pgrep alone is fine.
    if [[ "$_HAS_PGREP" -eq 1 && "$_HAS_KILL_CMD" -eq 1 && "$_GIT_TARGET" -eq 1 ]]; then
      deny "$_PKILL_MSG"
    fi
    # pgrep -x git is almost always a prelude to killing every git on the machine.
    if printf '%s' "$_FLAT" | grep -Eq '(^|[[:space:];|&])pgrep[[:space:]]+(-[A-Za-z0-9]+[[:space:]]+)*-x[[:space:]]+git([[:space:]]|$)'; then
      deny "$_PKILL_MSG"
    fi
    # lsof -t <git-binary> | kill
    if printf '%s' "$_FLAT" | grep -Eqi 'lsof[[:space:]]+-t[^;&|]*git[^;&|]*(^|[[:space:];|&])kill([[:space:]]|$)'; then
      deny "$_PKILL_MSG"
    fi

    # Raw unlock-by-rm of git lock files (command-shaped rm).
    if printf '%s' "$_FLAT" | grep -Eq \
      '(^|[[:space:];|&])rm[[:space:]]+[^;&|]*index\.lock|(^|[[:space:];|&])rm[[:space:]]+[^;&|]*HEAD\.lock|(^|[[:space:];|&])rm[[:space:]]+[^;&|]*gc\.pid|(^|[[:space:];|&])rm[[:space:]]+[^;&|]*MERGE_RR\.lock|(^|[[:space:];|&])rm[[:space:]]+[^;&|]*/\.git/[^;&|]*\.lock'; then
      deny "$_LOCK_RM_MSG"
    fi
  fi
fi

# Integration owner / explicit SKIP escapes (other destructive git)
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
