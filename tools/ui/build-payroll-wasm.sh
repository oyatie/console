#!/usr/bin/env bash
# Produce and publish the native Buck hydration action outputs together.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
candidate="$(tools/buck2 build //backend/crates/payroll/ui:console-payroll-ui-wasm-bundle --show-full-simple-output)"
python3 tools/ui/wasm_bundle.py publish --candidate "$candidate" --root "$root"
