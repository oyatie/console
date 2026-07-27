#!/usr/bin/env bash
# Red-proof for the §61 statutory windows.
#
# This lane replaced a *fabricated* statutory rule, so the tests must be shown to
# fail when the fix is removed. Each mutation below is applied to promotion.rs,
# run against both the pure-logic suite and the database-backed runtime-role
# test, then restored from a `cp` backup (never `git checkout`, which would
# discard uncommitted work).
#
# Usage:  DATABASE_URL=... bash docs/evidence/console/hotfix/leave-promotion/redproof.sh
# The credential lives in the environment and in no committed file.
set -uo pipefail

: "${DATABASE_URL:?set DATABASE_URL to the dev Postgres (see the app-tests Buck2 PG harness note)}"

W="$(git rev-parse --show-toplevel)"
SRC="$W/backend/crates/leave/domain/src/promotion.rs"
BAK="$(mktemp -t promotion.rs.bak.XXXXXX)"

cp "$SRC" "$BAK"
restore() { cp "$BAK" "$SRC"; }
trap restore EXIT

run() {
  cd "$W/backend" || exit 1
  echo "--- domain ---"
  cargo test -p console-leave-domain 2>&1 | grep -E "^(test result|error\[|error:)" | head -5
  echo "--- adapter statutory (DB, asserts as console_rt) ---"
  cargo test -p console-leave-adapter-postgres --test leave_rls_surfaces_as_runtime_role statutory \
    -- --test-threads=1 2>&1 | grep -E "^(test result|error\[|error:)" | head -5
}

mutate() { # <description> <perl-expression> <grep-assertion>
  echo
  echo "================ $1 ================"
  perl -0pi -e "$2" "$SRC"
  grep -q "$3" "$SRC" || { echo "[MUTATION FAILED TO APPLY]"; exit 1; }
  echo "[mutation applied]"
  run
  restore
}

echo "================ BASELINE (fix in place) ================"
run

# M1 — the deleted validate_round: any round 1|2 accepted, no window, no ordering.
mutate "MUTANT 1: the deleted validate_round" \
  's/(pub fn validate_promotion\(context: &PromotionContext, round: i16\) -> Result<i16, KernelError> \{\n)/$1    if round == 1 || round == 2 { return Ok(round); } \/\/ MUTANT\n/' \
  'MUTANT'

# M2 — off-by-one: proves the assertions pin the boundary day, not just the window.
mutate "MUTANT 2: TEN_DAY_SPAN 10 -> 11 (window closes one day late)" \
  's/const TEN_DAY_SPAN: i64 = 10;/const TEN_DAY_SPAN: i64 = 11;/' \
  'TEN_DAY_SPAN: i64 = 11'

# M3 — the worker's reply window collapsed: round 2 may follow the 촉구 immediately.
mutate "MUTANT 3: REPLY_WINDOW_DAYS 10 -> 0" \
  's/const REPLY_WINDOW_DAYS: i64 = 10;/const REPLY_WINDOW_DAYS: i64 = 0;/' \
  'REPLY_WINDOW_DAYS: i64 = 0'

echo
echo "================ RESTORED ================"
cd "$W" && git diff --stat -- backend/crates/leave/domain/src/promotion.rs
echo "(no diff line above = promotion.rs is byte-identical to the committed fix)"
