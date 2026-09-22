#!/usr/bin/env bash
set -euo pipefail
cd /private/tmp/console-mvp-dev-integrate-20260921
export PATH="/var/folders/66/4qlvtbgn6r1gl9bvfttp6sww0000gn/T/console-dotslash/bin:$PATH"
export CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK=/private/tmp/console-policy-buck-prerequisite-20260922/buck2-one-action
for test_name in \
 native_form_helpers_issue_recheck_and_reuse_same_proof_across_transactions \
 native_form_helpers_reject_missing_mismatched_and_submitted_issuance_proofs \
 native_form_helpers_reject_expired_signed_proof_and_accept_current_control \
 native_form_issuer_verifier_mismatch_is_unavailable_without_effects
do
 export CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT="account_browser::native_business_session::${test_name}"
 bash tools/buck/test_needs_postgres.sh --num-threads=1 //tools/buck:app-auth-rest-pg > "/private/tmp/console-policy-buck-prerequisite-20260922/${test_name}.log" 2>&1
 printf '%s passed\n' "$test_name"
done
