#!/usr/bin/env bash
# EXPERIMENT X2 — see docs/ideas/experiment-x1-x2.md. Not a deliverable.
#
# X2 needed no new probe: the experiment it asks for is already a shipped test,
# `an_attached_permit_is_the_only_thing_that_makes_instances_visible`
# (backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs),
# landed by #525. It publishes a type, seeds instances, lists them with no
# policy attached, attaches an enforced permit over the real HTTP route, and
# lists again.
#
# What it does NOT do is print, and a green test whose observations nobody can
# see is weaker evidence than the four lines below. instrumentation.patch adds
# the `println!`s and nothing else — no assertion is changed, added or removed,
# which `git diff --stat` after applying it will show. It is applied for the
# length of one run and reversed on every exit path, because the printouts have
# no business in the shipped suite.
#
# Expected output:
#   X2 CONTROL 0  current_user=console_rt rolsuper=false
#   X2 HALF 1     no policy attached -> 200 OK []
#   X2 HALF 2     policy attached (201 Created) -> 200 OK titles=Some([String("visible-to-owner")])
#   X2 HALF 1 bis  second unpoliced type in the SAME request -> 200 OK []
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="${1:-$(cd "$here/../../../.." && pwd)}"
target="backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs"

cd "$repo_root"
if ! git diff --quiet -- "$target"; then
  echo "$target already has uncommitted changes; refusing to patch over them" >&2
  exit 1
fi

cleanup() {
  git apply --reverse "$here/instrumentation.patch" 2>/dev/null || true
  if git diff --quiet -- "$target"; then
    echo "clean: instrumentation reversed"
  else
    echo "LEAK: $target is still modified -- revert it by hand" >&2
  fi
}
trap cleanup EXIT

git apply "$here/instrumentation.patch"

bash "$repo_root/tools/lanes/pgtest.sh" "$repo_root" \
  cargo test -p console-ontology-rest --test object_policy_attach_as_runtime_role \
  an_attached_permit_is_the_only_thing_that_makes_instances_visible -- --nocapture

# The same mechanism one layer down, unmodified: :228 asserts an unfiltered list
# sees all three rows (the list CAN return rows) and :244-248 asserts
# `list_instances_filtered(.., &[])` is empty.
bash "$repo_root/tools/lanes/pgtest.sh" "$repo_root" \
  cargo test -p console-ontology-adapter-postgres \
  --test instances_residual_filter_as_runtime_role
