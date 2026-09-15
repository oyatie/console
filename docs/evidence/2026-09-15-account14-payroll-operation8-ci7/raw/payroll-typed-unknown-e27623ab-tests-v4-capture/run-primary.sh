#!/bin/sh
set -eu
repo=${1:?exact root-owned candidate checkout required}
cd "$repo"
CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "$repo" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery durability_composition_tests::required_api_completion_unknown_reconciles_same_command_after_replay -- --exact --test-threads=1 --nocapture
CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "$repo" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery durability_composition_tests::required_api_receipt_absent_unknown_renews_approval_with_same_command -- --exact --test-threads=1 --nocapture
CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "$repo" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery durability_composition_tests::required_api_confirmed_owner_audit_failure_replays_and_repairs -- --exact --test-threads=1 --nocapture
CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "$repo" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery durability_composition_tests::required_workflow_typed_unknown_preserves_pending_and_failed_events -- --exact --test-threads=1 --nocapture
cargo test --locked --manifest-path backend/Cargo.toml -p console-ontology-rest --lib projected_dispatch_derivation::diagnostic_unknown_prefix_does_not_classify_an_operation_error -- --exact --test-threads=1 --nocapture
