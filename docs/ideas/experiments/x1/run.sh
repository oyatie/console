#!/usr/bin/env bash
# EXPERIMENT X1 — see docs/ideas/experiment-x1-x2.md. Not a deliverable, not a
# regression witness.
#
# Why probe.rs lives here and not in the crate's `tests/` directory: it asserts
# TODAY'S DEFECT — that a link-type-only relationship writes no edge and that
# `validate_draft` accepts the draft that produces it. §0.12 of the ecosystem
# plan proposes a guard that makes exactly this draft fail. A probe committed as
# a CI target would therefore turn into a wall against the fix it exists to
# justify: the guard lands, the probe goes red, and someone deletes the guard.
# So the file is copied into the crate for the length of one run and removed on
# every exit path.
#
# Container hygiene is delegated to tools/lanes/pgtest.sh, which asserts on its
# own container name only.
set -euo pipefail

repo_root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)}"
crate="$repo_root/backend/crates/ontology/adapter-postgres"
dest="$crate/tests/x1probe_link_type_alone_experiment.rs"

if [ -e "$dest" ]; then
  echo "refusing to overwrite an existing $dest" >&2
  exit 1
fi

# The migrations path inside probe.rs is relative to the crate, so the copy has
# to land in this exact directory for `#[sqlx::test]` to resolve it.
cleanup() { rm -f "$dest"; echo "clean: removed the borrowed test file"; }
trap cleanup EXIT

cp "$(dirname "${BASH_SOURCE[0]}")/probe.rs" "$dest"

bash "$repo_root/tools/lanes/pgtest.sh" "$repo_root" \
  cargo test -p console-ontology-adapter-postgres \
  --test x1probe_link_type_alone_experiment -- --nocapture
