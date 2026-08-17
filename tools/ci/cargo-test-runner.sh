#!/usr/bin/env bash
# Run a stream of cargo test invocations (JSONL on stdin) with fail-slow
# keep-going semantics. Each stdin line is one JSON object:
#   {"name": "...", "package": "...", "argv": ["cargo", "test", ...]}
#
# Extracted from cargo_needs_postgres.sh so the keep-going loop is unit-testable
# without Docker or a real Cargo workspace: feed it JSONL and point PATH at a
# stubbed `cargo`.
#
# Exit 0 only when every invocation passed; exit 1 when any failed (the workflow
# relies on non-zero == red). Keep-going is the default; --fail-fast aborts after
# the first failure (local use). `set -e` is deliberately OUT of the per-binary
# loop: failures are captured explicitly so one red binary never hides the rest.
#
# Environment (set by the caller):
#   CARGO_REPO_ROOT  repository root used as the cargo cwd
#   RUST_TEST_THREADS number of test threads per invocation (default 1)
set -uo pipefail

repo_root="${CARGO_REPO_ROOT:-}"
threads="${RUST_TEST_THREADS:-1}"
fail_fast=0
case "${1:-}" in
  --keep-going) fail_fast=0 ;;
  --fail-fast) fail_fast=1 ;;
  "")
    ;;
  *)
    echo "usage: cargo-test-runner.sh [--keep-going|--fail-fast]" >&2
    exit 2
    ;;
esac

if [[ -z "${repo_root}" ]]; then
  echo "cargo-test-runner: CARGO_REPO_ROOT must name the repository root" >&2
  exit 2
fi

passed=0
failed=0
results=()

# Per-target durations, appended as JSONL when CARGO_POSTGRES_TIMINGS names a
# path. The shard partitioner bin-packs by ENTRY COUNT today, which does not
# predict time: measured per-entry cost across the five facets ranged 17.8s
# (app) to 32.4s (ontology), so equal-count shards ran 1097s and 1345s. Nothing
# could balance by duration because duration was never recorded. This is the
# measurement; the sharder consumes it separately. Unset => no file, same
# behaviour as before.
timings_path="${CARGO_POSTGRES_TIMINGS:-}"
if [[ -n "${timings_path}" ]]; then
  : >"${timings_path}"
fi

# Emitted through python so the name/package land as real JSON strings; a
# printf-built line would break on the first target whose name needs escaping.
record_timing() {
  [[ -n "${timings_path}" ]] || return 0
  python3 - "$1" "$2" "$3" "${timings_path}" <<'PY'
import json, sys
row, seconds, status, path = json.loads(sys.argv[1]), int(sys.argv[2]), sys.argv[3], sys.argv[4]
with open(path, "a", encoding="utf-8") as handle:
    handle.write(json.dumps({
        "name": row["name"],
        "package": row["package"],
        "seconds": seconds,
        "status": status,
    }, sort_keys=True) + "\n")
PY
}

while IFS= read -r row || [[ -n "${row}" ]]; do
  name="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["name"])' "${row}")"
  echo "cargo-postgres: === ${name} ==="
  started_at=${SECONDS}
  # JSON array exec avoids the word-splitting bugs of `set -- $(...)`; the argv
  # is taken from the map entry verbatim.
  if python3 - "${repo_root}" "${row}" "${threads}" <<'PY'
import json, os, subprocess, sys
root, row, threads = sys.argv[1], json.loads(sys.argv[2]), sys.argv[3]
argv = row["argv"]
env = os.environ.copy()
env["SQLX_OFFLINE"] = "true"
env["RUST_TEST_THREADS"] = threads
env["CARGO_TERM_COLOR"] = "always"
print("cargo-postgres:", " ".join(argv), flush=True)
raise SystemExit(subprocess.call(argv, cwd=root, env=env))
PY
  then
    elapsed=$((SECONDS - started_at))
    passed=$((passed + 1))
    results+=("PASS  ${name}  (${elapsed}s)")
    record_timing "${row}" "${elapsed}" pass
  else
    elapsed=$((SECONDS - started_at))
    failed=$((failed + 1))
    results+=("FAIL  ${name}  (${elapsed}s)")
    record_timing "${row}" "${elapsed}" fail
    if [[ "${fail_fast}" == 1 ]]; then
      break
    fi
  fi
done

echo ""
echo "cargo-postgres: summary"
for entry in "${results[@]}"; do
  printf '  %s\n' "${entry}"
done
echo "cargo-postgres: ${passed} passed, ${failed} failed"

if [[ "${failed}" != 0 ]]; then
  echo "cargo-postgres: one or more invocations failed" >&2
  exit 1
fi
echo "cargo-postgres: all ${passed} invocations passed"
