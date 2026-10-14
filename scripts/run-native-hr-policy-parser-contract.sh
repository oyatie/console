#!/bin/sh
set -eu
case "${CONSOLE_HR_POLICY_NODE:-}" in /*) ;; *) exit 126 ;; esac
[ -f "$CONSOLE_HR_POLICY_NODE" ] && [ -x "$CONSOLE_HR_POLICY_NODE" ] || exit 126
[ -n "${CONSOLE_HR_POLICY_BINDING:-}" ] && [ -n "${CONSOLE_HR_POLICY_BINDING_SHA256:-}" ] || exit 126
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd) || exit 126
[ -f "$SCRIPT_DIR/check-native-hr-policy-parser-contract.mjs" ] || exit 126
[ -f "$SCRIPT_DIR/check-native-hr-policy-parser-contract.test.mjs" ] || exit 126
status=0
"$CONSOLE_HR_POLICY_NODE" "$SCRIPT_DIR/check-native-hr-policy-parser-contract.mjs" || status=$?
case "$status" in 0|101) exit "$status" ;; *) exit 126 ;; esac
