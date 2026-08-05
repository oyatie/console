#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { fileURLToPath, pathToFileURL } from "node:url";
import yaml from "js-yaml";

const dotSlashBootstrap = "tools/buck/install_dotslash.sh";
const reindeerToolchainLock = "third-party/rust/reindeer/upstream.lock";
const reindeerToolchainSource = `source ${reindeerToolchainLock}`;
const reindeerToolchainInstall = 'rustup toolchain install "$REINDEER_TOOLCHAIN" --profile minimal';
const strictShellMode = "set -euo pipefail";
const reindeerToolchainOverride = /^(?:export\s+)?REINDEER_TOOLCHAIN\s*=/;
const ciPreflightTestCommand = "node --test scripts/check-ci-preflight.test.mjs";
const reasoningLensTestCommand = "node --test scripts/check-reasoning-lens-contract.test.mjs";
const reasoningLensRegressionName = "Reasoning lens contract regression";
const reasoningLensAdmissionName = "Reasoning lens changed-record admission";
const reasoningLensAdmissionEnvironment = [
  "REASONING_PR_BASE_SHA: ${{ github.event.pull_request.base.sha }}",
  "REASONING_PUSH_BEFORE_SHA: ${{ github.event.before }}",
];
const reasoningLensAdmissionScript = [
  "set -euo pipefail",
  'case "$GITHUB_EVENT_NAME" in',
  "  pull_request)",
  '    test -n "$REASONING_PR_BASE_SHA"',
  '    node scripts/check-reasoning-lens-contract.mjs --changed-since "$REASONING_PR_BASE_SHA"',
  "    ;;",
  "  push)",
  '    case "$GITHUB_REF_TYPE" in',
  "      branch)",
  '        test -n "$REASONING_PUSH_BEFORE_SHA"',
  '        if [[ "$REASONING_PUSH_BEFORE_SHA" == "0000000000000000000000000000000000000000" ]]; then',
  "          printf '%s\\n' 'push.before is all-zero; running structural reasoning-lens validation'",
  "          node scripts/check-reasoning-lens-contract.mjs",
  "        else",
  '          node scripts/check-reasoning-lens-contract.mjs --changed-since "$REASONING_PUSH_BEFORE_SHA"',
  "        fi",
  "        ;;",
  "      tag)",
  "        node scripts/check-reasoning-lens-contract.mjs",
  "        ;;",
  "      *)",
  "        printf 'unsupported push ref type: %s\\n' \"$GITHUB_REF_TYPE\" >&2",
  "        exit 2",
  "        ;;",
  "    esac",
  "    ;;",
  "  workflow_dispatch)",
  "    node scripts/check-reasoning-lens-contract.mjs",
  "    ;;",
  "  *)",
  "    printf 'unsupported reasoning-lens event: %s\\n' \"$GITHUB_EVENT_NAME\" >&2",
  "    exit 2",
  "    ;;",
  "esac",
].join("\n");
const consoleRouteInventoryTestCommand = "node --test scripts/console/route-inventory.test.mjs";
const consoleTruthLedgerCommand = "npm run check:console-truth-ledger";
const consoleAuthorityTrainTestCommand = "node --test scripts/console/verify-console-authority-train.test.mjs";
const consoleTruthLedgerTestCommand = "node --test scripts/console/validate-console-truth-ledger.test.mjs";
const consoleFanoutPlannerTestCommand = "node --test scripts/console/plan-fanout.test.mjs";
// This one gates the highest-privilege script in the repository — the `pull_request_target`
// bootstrap verifier — and executed NOWHERE until it was wired: `package.json` declared
// `test:console-authority-bootstrap` and no workflow ever invoked it. Breaking the verifier
// turned all its tests red locally while CI stayed green.
const consoleBootstrapTestCommand = "node --test scripts/console/verify-console-pr-authority-bootstrap.test.mjs";
const consoleFanoutPlannerAdmissionCommand = 'node scripts/console/plan-fanout.mjs --candidate "$CONSOLE_CANDIDATE_SHA" --authority-tip "$CONSOLE_AUTHORITY_TIP_SHA" --synthetic-merge "$CONSOLE_SYNTHETIC_MERGE_SHA"';
const consolePrCondition = "${{ github.event_name == 'pull_request' }}";
const consoleTrainDerivation = [
  "set -euo pipefail",
  'CONSOLE_SYNTHETIC_MERGE_SHA="$(git rev-parse "$GITHUB_SHA^{commit}")"',
  'test "$(git rev-parse HEAD)" = "$CONSOLE_SYNTHETIC_MERGE_SHA"',
  'CONSOLE_AUTHORITY_TIP_SHA="$(git rev-parse "$CONSOLE_SYNTHETIC_MERGE_SHA^2")"',
  'CONSOLE_CANDIDATE_SHA="$(git rev-parse "$CONSOLE_AUTHORITY_TIP_SHA^")"',
  "{",
  "printf 'CONSOLE_CANDIDATE_SHA=%s\\n' \"$CONSOLE_CANDIDATE_SHA\"",
  "printf 'CONSOLE_AUTHORITY_TIP_SHA=%s\\n' \"$CONSOLE_AUTHORITY_TIP_SHA\"",
  "printf 'CONSOLE_SYNTHETIC_MERGE_SHA=%s\\n' \"$CONSOLE_SYNTHETIC_MERGE_SHA\"",
  '} >> "$GITHUB_ENV"',
];
const buckPostgresEnvironmentTestCommand = "tools/buck/run_test_with_postgres_env.test.sh";
const buckPostgresHarnessTestCommand = "tools/buck/test_needs_postgres.test.sh";
// The domain-unit step is now a multi-line run. What must be asserted is not the exact
// text but that every audit-relevant package added on 2026-07-31 is still named — dropping
// one silently returns its tests to executing nowhere, which is the condition
// check:executed-tests exists to measure and this assertion exists to prevent.
const domainUnitPackages = [
  "console-support-domain",
  "console-payroll-domain",
  "console-payroll-adapter-postgres",
  "console-attendance-application",
  "console-compliance-domain",
  "console-governance-domain",
  "console-platform-audit-chain",
  "console-platform-authz",
  "console-policy-application",
  "console-policy-domain",
  // Added 2026-07-31. Every crate below held test code that executed in no workflow step.
  // They are domain and application crates — no database, no fixture, no wrapper target —
  // so the only thing that had ever kept them dark was that nobody named them here.
  "console-action-inbox-application",
  "console-attendance-domain",
  "console-benefit-application",
  "console-benefit-domain",
  "console-comms-application",
  "console-comms-domain",
  "console-dispatch-application",
  "console-dispatch-domain",
  "console-docs-application",
  "console-docs-domain",
  "console-equipment-domain",
  "console-erp-domain",
  "console-evaluation-application",
  "console-evaluation-domain",
  "console-finance-gl-domain",
  "console-identity-domain",
  "console-inbox-application",
  "console-inbox-domain",
  "console-inventory-domain",
  "console-leave-domain",
  "console-logistics-domain",
  "console-messenger-application",
  "console-notices-domain",
  "console-notifications-domain",
  "console-ontology-application",
  "console-ontology-domain",
  "console-orgchange-domain",
  "console-recruiting-application",
  "console-recruiting-domain",
  "console-reporting-application",
  "console-reporting-domain",
  "console-sales-domain",
  "console-support-application",
  "console-todos-domain",
  "console-workflow-domain",
  "console-workorder-application",
  // Second tranche, same day. These are `--lib` unit tests in `rest`, `adapter-postgres`
  // and CI-gate crates. FOUR sibling crates were excluded because their lib carries a
  // `#[sqlx::test]` and needs DATABASE_URL — console-platform-group, console-platform-storage,
  // console-gate-rls-arming, console-support-rest. "--lib means no database" is false here,
  // and the only reason that is known is that running them produced
  // "DATABASE_URL must be set" rather than a green summary.
  "console-analytics-quant-service",
  "console-attendance-adapter-postgres",
  "console-attendance-rest",
  "console-benefit-rest",
  "console-comms-adapter-imap",
  "console-comms-adapter-mox",
  "console-comms-adapter-smtp",
  "console-comms-credential-cipher",
  "console-comms-mailbox",
  "console-comms-rest",
  "console-compliance-rest",
  "console-consulting-rest",
  "console-dispatch-rest",
  "console-docs-adapter-postgres",
  "console-docs-rest",
  "console-evaluation-adapter-postgres",
  "console-facilities-rest",
  "console-financial-rest",
  "console-gate-dev-auth-absence",
  "console-gate-iac-tier",
  "console-gate-layer-boundary",
  "console-gate-tenant-isolation",
  "console-gate-vendor-lockin",
  "console-identity-rest",
  "console-inventory-adapter-postgres",
  "console-inventory-rest",
  "console-leave-adapter-postgres",
  "console-leave-rest",
  "console-ontology-adapter-postgres",
  "console-ontology-rest",
  "console-payroll-rest",
  "console-platform-email",
  "console-platform-jobs",
  "console-platform-push",
  "console-platform-realtime",
  "console-platform-request-context",
  "console-production-rest",
  "console-reporting-adapter-postgres",
  "console-support-adapter-postgres",
  "console-kernel-core",
  // Added 2026-08-03: the consolidation changed authorization decisions in these
  // REST libraries. Their pure unit tests must remain part of the protected job,
  // rather than becoming implementation-specific tests that exist but never run.
  "console-registry-rest",
  "console-reporting-rest",
  "console-workflow-runtime-adapter-postgres",
  "console-workorder-rest",
  "console-financial-domain",
  "console-identity-application",
  "console-integrity",
  "console-platform-auth",
  "console-platform-authz-rest",
  "console-workflow-runtime",
];
const domainUnitIntegrationInvocations = [
  ["console-attendance-application", ["attendance_policy"]],
  ["console-compliance-domain", ["location_consent_fsm", "location_ping_policy"]],
  ["console-platform-authz", ["cedar_pbac_readiness_cases", "cedar_pbac_legacy_only_observe_and_record"]],
  ["console-attendance-domain", ["range_and_history"]],
  ["console-financial-domain", ["quote_and_residual"]],
  ["console-registry-domain", ["equipment"]],
  ["console-messenger-domain", ["mentions", "object_code_refs", "parity", "thread_kind"]],
  ["console-workorder-domain", ["approval_and_assignment", "serde_roundtrips", "settlement_fsm", "workorder_fsm"]],
  ["console-platform-auth", ["jwt_es256", "jwt_verifier", "well_known"]],
  ["console-platform-excel", ["template_fidelity", "template_fill_engine"]],
  ["console-platform-realtime", ["hub", "notify_payload"]],
  ["console-app", ["config", "dev_seed_notification_links", "openslo_files", "well_known", "workbench_api"]],
];
const domainUnitTestFiles = domainUnitIntegrationInvocations.flatMap(([, tests]) => tests);
const domainCargoPrefix = [
  "SQLX_OFFLINE=true",
  "cargo",
  "test",
  "--locked",
  "--manifest-path",
  "backend/Cargo.toml",
];
const domainUnitExpectedCommands = [
  [...domainCargoPrefix, "--lib", ...domainUnitPackages.flatMap((pkg) => ["-p", pkg])],
  [...domainCargoPrefix, "--doc", "-p", "console-kernel-core"],
  ...domainUnitIntegrationInvocations.map(([pkg, tests]) => [
    ...domainCargoPrefix,
    "-p",
    pkg,
    ...tests.flatMap((test) => ["--test", test]),
  ]),
];
const postgresDomainReachabilityCommands = [
  "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
  "//tools/buck:dispatch-p1-postgres \\",
  "//tools/buck:attendance-cancel-substitution-postgres \\",
  "//tools/buck:attendance-concurrency-postgres \\",
  "//tools/buck:ontology-object-type-lifecycle-postgres \\",
  "//tools/buck:ontology-object-type-cas-postgres \\",
  "//tools/buck:ontology-publish-auto-create-action-postgres \\",
  "//tools/buck:ontology-object-policy-attach-postgres \\",
  "//tools/buck:ontology-action-execute-postgres \\",
  "//tools/buck:ontology-gaps-postgres \\",
  "//tools/buck:ontology-projected-dispatch-postgres \\",
  "//tools/buck:platform-erasure-ledger-postgres \\",
  "//tools/buck:platform-db-rls-isolation \\",
  "//tools/buck:platform-db-rls-rollout-isolation \\",
  "//tools/buck:platform-audit-chain-rls \\",
  "//tools/buck:platform-provisioning-rls-auth-chain \\",
  "//tools/buck:platform-rest-remove-tenant \\",
  "//tools/buck:compliance-location-consent-status-rls \\",
  "//tools/buck:compliance-location-store \\",
  "//tools/buck:payroll-rls-surfaces \\",
  "//tools/buck:platform-db-feature-catalog-coverage \\",
  "//tools/buck:app-action-inbox-api-pg \\",
  "//tools/buck:app-audit-api-pg \\",
  "//tools/buck:app-auth-rest-pg \\",
  "//tools/buck:app-board-ack-api-pg \\",
  "//tools/buck:app-cedar-freshness-mint-pg \\",
  "//tools/buck:app-cedar-parity-shadow-pg \\",
  "//tools/buck:app-cedar-shadow-role-manage-pg \\",
  "//tools/buck:app-compliance-api-pg \\",
  "//tools/buck:app-console-kill-switch-pg \\",
  "//tools/buck:app-console-route-telemetry-pg \\",
  "//tools/buck:app-dev-auth-persona-guard-pg \\",
  "//tools/buck:app-dispatch-pipeline-api-pg \\",
  "//tools/buck:app-equipment-3r-api-pg \\",
  "//tools/buck:app-evaluation-cycle-api-pg \\",
  "//tools/buck:app-field-visit-api-pg \\",
  "//tools/buck:app-finance-gl-voucher-sod-pg \\",
  "//tools/buck:app-health-readiness-pg \\",
  "//tools/buck:app-hr-attendance-manager-scope-pg \\",
  "//tools/buck:app-hr-attendance-self-read-pg \\",
  "//tools/buck:app-hr-ingest-checklist-gate-pg \\",
  "//tools/buck:app-hr-people-create-api-pg \\",
  "//tools/buck:app-m2-real-engine-drive-pg \\",
  "//tools/buck:app-maintenance-chain-api-pg \\",
  "//tools/buck:app-mobile-api-pg \\",
  "//tools/buck:app-notif-routing-api-pg \\",
  "//tools/buck:app-notifications-api-pg \\",
  "//tools/buck:app-object-graph-api-pg \\",
  "//tools/buck:app-object-links-api-pg \\",
  "//tools/buck:app-object-ontology-api-pg \\",
  "//tools/buck:app-object-resolve-api-pg \\",
  "//tools/buck:app-office-versions-pg \\",
  "//tools/buck:app-org-change-api-pg \\",
  "//tools/buck:app-platform-onboarding-e2e-pg \\",
  "//tools/buck:app-purchase-request-collection-api-pg \\",
  "//tools/buck:app-realtime-ws-pg \\",
  "//tools/buck:app-recruiting-pipeline-api-pg \\",
  "//tools/buck:app-registry-api-pg \\",
  "//tools/buck:app-router-layers-pg \\",
  "//tools/buck:app-search-api-pg \\",
  "//tools/buck:app-submittable-definitions-api-pg \\",
  "//tools/buck:app-tenant-context-e2e-pg \\",
  "//tools/buck:app-workbench-native-api-pg \\",
  "//tools/buck:app-workflow-automation-triggers-pg \\",
  "//tools/buck:app-workflow-dynamics-branch-pg \\",
  "//tools/buck:app-workflow-four-eyes-publish-pg \\",
  "//tools/buck:app-workflow-object-context-api-pg \\",
  "//tools/buck:app-workflow-object-kind-dynamics-pg \\",
  "//tools/buck:app-workflow-run-read-surface-pg \\",
  "//tools/buck:app-workflow-runtime-finalize-api-pg \\",
  "//tools/buck:app-workflow-runtime-instance-api-pg \\",
  "//tools/buck:app-workorder-api-pg \\",
  "//tools/buck:rls-arming-lib-pg \\",
  "//tools/buck:attendance-adapter-postgres-self-service-pg \\",
  "//tools/buck:benefit-adapter-postgres-catalog-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:comms-adapter-postgres-mail-account-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:comms-adapter-postgres-mail-sync-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:comms-adapter-postgres-send-rate-limit-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:comms-rest-mox-webhook-pg \\",
  "//tools/buck:comms-rest-readiness-pg \\",
  "//tools/buck:consulting-rest-audit-atomicity-pg \\",
  "//tools/buck:dispatch-worker-timer-delivery-pg \\",
  "//tools/buck:docs-rest-evidence-rest-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:finance-gl-adapter-postgres-voucher-rls-and-fsm-as-runtime-role-pg \\",
  "//tools/buck:financial-adapter-postgres-lifecycle-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:financial-adapter-postgres-period-lock-blocks-ledger-as-runtime-role-pg \\",
  "//tools/buck:financial-adapter-postgres-use-cases-pg \\",
  "//tools/buck:financial-rest-purchase-request-list-pg \\",
  "//tools/buck:governance-adapter-postgres-approvals-create-as-runtime-role-pg \\",
  "//tools/buck:governance-adapter-postgres-four-eyes-bind-consume-pg \\",
  "//tools/buck:governance-adapter-postgres-governance-rls-as-runtime-role-pg \\",
  "//tools/buck:identity-adapter-postgres-deactivate-revokes-credentials-pg \\",
  "//tools/buck:identity-adapter-postgres-me-workspace-layouts-rls-pg \\",
  "//tools/buck:identity-adapter-postgres-region-branch-crud-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:identity-adapter-postgres-subject-authz-versions-freshness-rls-pg \\",
  "//tools/buck:inbox-adapter-postgres-inbox-docs-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:inbox-rest-api-pg \\",
  "//tools/buck:inspection-adapter-postgres-lifecycle-pg \\",
  "//tools/buck:inspection-adapter-postgres-schedule-window-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:inventory-adapter-postgres-consume-idempotency-concurrency-pg \\",
  "//tools/buck:leave-adapter-postgres-leave-migration-expand-contract-pg \\",
  "//tools/buck:leave-adapter-postgres-leave-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:leave-rest-leave-http-personas-pg \\",
  "//tools/buck:messenger-adapter-postgres-parity-tables-rls-as-runtime-role-pg \\",
  "//tools/buck:messenger-adapter-postgres-use-cases-pg \\",
  "//tools/buck:messenger-rest-api-pg \\",
  "//tools/buck:notices-adapter-postgres-notices-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:notices-rest-api-pg \\",
  "//tools/buck:notifications-adapter-postgres-notifications-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:notifications-rest-api-pg \\",
  "//tools/buck:ontology-adapter-postgres-builtin-catalog-additive-upgrade-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-c-chain-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-config-object-types-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-instances-residual-filter-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-instances-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-key-revision-migration-upgrade-pg \\",
  "//tools/buck:ontology-adapter-postgres-key-write-cas-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-niche-config-object-types-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-projected-instances-read-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-property-derivation-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-property-link-sync-as-runtime-role-pg \\",
  "//tools/buck:ontology-adapter-postgres-registry-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:payroll-adapter-postgres-payroll-lifecycle-rls-as-runtime-role-pg \\",
  "//tools/buck:payroll-rest-api-pg \\",
  "//tools/buck:payroll-rest-payslip-draft-api-pg \\",
  "//tools/buck:payroll-rest-run-lifecycle-api-pg \\",
  "//tools/buck:platform-auth-rest-dev-auth-absence-pg \\",
  "//tools/buck:platform-auth-refresh-tokens-pg \\",
  "//tools/buck:platform-auth-webauthn-ceremony-pg \\",
  "//tools/buck:platform-auth-webauthn-ceremony-replay-pg \\",
  "//tools/buck:platform-authz-rest-cedar-authoring-rls-as-runtime-role-pg \\",
  "//tools/buck:platform-authz-rest-decision-feed-as-runtime-role-pg \\",
  "//tools/buck:platform-authz-policy-pg \\",
  "//tools/buck:platform-db-attendance-console-migration-contract-pg \\",
  "//tools/buck:platform-db-code-issuance-pg \\",
  "//tools/buck:platform-db-group-resolvers-pg \\",
  "//tools/buck:platform-db-lifecycle-maker-checker-pg \\",
  "//tools/buck:platform-db-m2-flag-on-runtime-drain-pg \\",
  "//tools/buck:platform-db-period-locks-and-lifecycle-pg \\",
  "//tools/buck:platform-db-personal-data-classification-pg \\",
  "//tools/buck:platform-group-lib-pg \\",
  "//tools/buck:platform-jobs-apalis-adapter-pg \\",
  "//tools/buck:platform-jobs-apalis-schema-contract-pg \\",
  "//tools/buck:platform-platform-rest-onboard-seeds-config-objects-pg \\",
  "//tools/buck:platform-platform-rest-ops-dashboard-pg \\",
  "//tools/buck:platform-platform-rest-platform-groups-pg \\",
  "//tools/buck:platform-platform-rest-view-as-pg \\",
  "//tools/buck:platform-provisioning-bootstrap-passkey-pg \\",
  "//tools/buck:platform-provisioning-bootstrap-passkey-replay-pg \\",
  "//tools/buck:platform-provisioning-roster-import-pg \\",
  "//tools/buck:platform-provisioning-self-enroll-handoff-as-runtime-role-pg \\",
  "//tools/buck:platform-realtime-postgres-bridge-pg \\",
  "//tools/buck:platform-storage-lib-pg \\",
  "//tools/buck:platform-storage-evidence-processing-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:policy-adapter-postgres-draft-storage-pg \\",
  "//tools/buck:registry-adapter-postgres-create-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:registry-adapter-postgres-equipment-list-rls-as-runtime-role-pg \\",
  "//tools/buck:registry-adapter-postgres-equipment-lookup-normalization-rls-as-runtime-role-pg \\",
  "//tools/buck:registry-adapter-postgres-equipment-versioning-as-runtime-role-pg \\",
  "//tools/buck:registry-adapter-postgres-master-list-import-pg \\",
  "//tools/buck:registry-adapter-postgres-master-list-import-rls-as-runtime-role-pg \\",
  "//tools/buck:registry-adapter-postgres-site-address-postal-roundtrip-rls-as-runtime-role-pg \\",
  "//tools/buck:registry-rest-equipment-admin-pg \\",
  "//tools/buck:reporting-adapter-postgres-excel-exports-pg \\",
  "//tools/buck:reporting-adapter-postgres-kpi-golden-dataset-pg \\",
  "//tools/buck:reporting-adapter-postgres-ops-summary-pg \\",
  "//tools/buck:reporting-adapter-postgres-work-diary-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:sales-adapter-postgres-inquiry-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:sales-adapter-postgres-sales-store-pg \\",
  "//tools/buck:support-adapter-postgres-assignee-name-join-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:support-adapter-postgres-create-internal-ticket-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:support-adapter-postgres-support-tickets-pg \\",
  "//tools/buck:support-rest-lib-pg \\",
  "//tools/buck:support-rest-authz-pg \\",
  "//tools/buck:support-rest-intake-pg \\",
  "//tools/buck:todos-adapter-postgres-todos-rls-surfaces-as-runtime-role-pg \\",
  "//tools/buck:workflow-adapter-postgres-notification-bridge-pg \\",
  "//tools/buck:workflow-adapter-postgres-payroll-drain-period-lock-pg \\",
  "//tools/buck:workorder-adapter-postgres-m2-flag-off-parity-pg \\",
  "//tools/buck:workorder-adapter-postgres-rls-read-surfaces-as-runtime-role-pg \\",
  "//tools/buck:workorder-adapter-postgres-use-cases-pg \\",
  "//tools/buck:workorder-rest-mobile-device-registration-pg \\",
  "//tools/buck:workorder-rest-mobile-evidence-pg \\",
  "//tools/buck:workorder-rest-mobile-sync-pg",
];
const companyConformanceCommands = [
  "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
  "//tools/buck:company-conformance-postgres",
];
const postgresWrapperContracts = [
  ["platform-db-feature-catalog-coverage", "//backend/crates/platform/db:console-platform-db-itest-feature_catalog_covers_every_feature"],

  // The remaining 167, 2026-07-31.
  ["app-action-inbox-api-pg", "//backend/app:console-app-itest-action_inbox_api"],
  ["app-audit-api-pg", "//backend/app:console-app-itest-audit_api"],
  ["app-auth-rest-pg", "//backend/app:console-app-itest-auth_rest"],
  ["app-board-ack-api-pg", "//backend/app:console-app-itest-board_ack_api"],
  ["app-cedar-freshness-mint-pg", "//backend/app:console-app-itest-cedar_freshness_mint"],
  ["app-cedar-parity-shadow-pg", "//backend/app:console-app-itest-cedar_parity_shadow"],
  ["app-cedar-shadow-role-manage-pg", "//backend/app:console-app-itest-cedar_shadow_role_manage"],
  ["app-compliance-api-pg", "//backend/app:console-app-itest-compliance_api"],
  ["app-console-kill-switch-pg", "//backend/app:console-app-itest-console_kill_switch"],
  ["app-console-route-telemetry-pg", "//backend/app:console-app-itest-console_route_telemetry"],
  ["app-dev-auth-persona-guard-pg", "//backend/app:console-app-itest-dev_auth_persona_guard"],
  ["app-dispatch-pipeline-api-pg", "//backend/app:console-app-itest-dispatch_pipeline_api"],
  ["app-equipment-3r-api-pg", "//backend/app:console-app-itest-equipment_3r_api"],
  ["app-evaluation-cycle-api-pg", "//backend/app:console-app-itest-evaluation_cycle_api"],
  ["app-field-visit-api-pg", "//backend/app:console-app-itest-field_visit_api"],
  ["app-finance-gl-voucher-sod-pg", "//backend/app:console-app-itest-finance_gl_voucher_sod"],
  ["app-health-readiness-pg", "//backend/app:console-app-itest-health_readiness"],
  ["app-hr-attendance-manager-scope-pg", "//backend/app:console-app-itest-hr_attendance_manager_scope"],
  ["app-hr-attendance-self-read-pg", "//backend/app:console-app-itest-hr_attendance_self_read"],
  ["app-hr-ingest-checklist-gate-pg", "//backend/app:console-app-itest-hr_ingest_checklist_gate"],
  ["app-hr-people-create-api-pg", "//backend/app:console-app-itest-hr_people_create_api"],
  ["app-m2-real-engine-drive-pg", "//backend/app:console-app-itest-m2_real_engine_drive"],
  ["app-maintenance-chain-api-pg", "//backend/app:console-app-itest-maintenance_chain_api"],
  ["app-mobile-api-pg", "//backend/app:console-app-itest-mobile_api"],
  ["app-notif-routing-api-pg", "//backend/app:console-app-itest-notif_routing_api"],
  ["app-notifications-api-pg", "//backend/app:console-app-itest-notifications_api"],
  ["app-object-graph-api-pg", "//backend/app:console-app-itest-object_graph_api"],
  ["app-object-links-api-pg", "//backend/app:console-app-itest-object_links_api"],
  ["app-object-ontology-api-pg", "//backend/app:console-app-itest-object_ontology_api"],
  ["app-object-resolve-api-pg", "//backend/app:console-app-itest-object_resolve_api"],
  ["app-office-versions-pg", "//backend/app:console-app-itest-office_versions"],
  ["app-org-change-api-pg", "//backend/app:console-app-itest-org_change_api"],
  ["app-platform-onboarding-e2e-pg", "//backend/app:console-app-itest-platform_onboarding_e2e"],
  ["app-purchase-request-collection-api-pg", "//backend/app:console-app-itest-purchase_request_collection_api"],
  ["app-realtime-ws-pg", "//backend/app:console-app-itest-realtime_ws"],
  ["app-recruiting-pipeline-api-pg", "//backend/app:console-app-itest-recruiting_pipeline_api"],
  ["app-registry-api-pg", "//backend/app:console-app-itest-registry_api"],
  ["app-router-layers-pg", "//backend/app:console-app-itest-router_layers"],
  ["app-search-api-pg", "//backend/app:console-app-itest-search_api"],
  ["app-submittable-definitions-api-pg", "//backend/app:console-app-itest-submittable_definitions_api"],
  ["app-tenant-context-e2e-pg", "//backend/app:console-app-itest-tenant_context_e2e"],
  ["app-workbench-native-api-pg", "//backend/app:console-app-itest-workbench_native_api"],
  ["app-workflow-automation-triggers-pg", "//backend/app:console-app-itest-workflow_automation_triggers"],
  ["app-workflow-dynamics-branch-pg", "//backend/app:console-app-itest-workflow_dynamics_branch"],
  ["app-workflow-four-eyes-publish-pg", "//backend/app:console-app-itest-workflow_four_eyes_publish"],
  ["app-workflow-object-context-api-pg", "//backend/app:console-app-itest-workflow_object_context_api"],
  ["app-workflow-object-kind-dynamics-pg", "//backend/app:console-app-itest-workflow_object_kind_dynamics"],
  ["app-workflow-run-read-surface-pg", "//backend/app:console-app-itest-workflow_run_read_surface"],
  ["app-workflow-runtime-finalize-api-pg", "//backend/app:console-app-itest-workflow_runtime_finalize_api"],
  ["app-workflow-runtime-instance-api-pg", "//backend/app:console-app-itest-workflow_runtime_instance_api"],
  ["app-workorder-api-pg", "//backend/app:console-app-itest-workorder_api"],
  ["rls-arming-lib-pg", "//backend/ci/gates/rls-arming:console-gate-rls-arming-unit"],
  ["attendance-adapter-postgres-self-service-pg", "//backend/crates/attendance/adapter-postgres:console-attendance-adapter-postgres-itest-self_service"],
  ["benefit-adapter-postgres-catalog-rls-surfaces-as-runtime-role-pg", "//backend/crates/benefit/adapter-postgres:console-benefit-adapter-postgres-itest-catalog_rls_surfaces_as_runtime_role"],
  ["comms-adapter-postgres-mail-account-rls-surfaces-as-runtime-role-pg", "//backend/crates/comms/adapter-postgres:console-comms-adapter-postgres-itest-mail_account_rls_surfaces_as_runtime_role"],
  ["comms-adapter-postgres-mail-sync-rls-surfaces-as-runtime-role-pg", "//backend/crates/comms/adapter-postgres:console-comms-adapter-postgres-itest-mail_sync_rls_surfaces_as_runtime_role"],
  ["comms-adapter-postgres-send-rate-limit-rls-surfaces-as-runtime-role-pg", "//backend/crates/comms/adapter-postgres:console-comms-adapter-postgres-itest-send_rate_limit_rls_surfaces_as_runtime_role"],
  ["comms-rest-mox-webhook-pg", "//backend/crates/comms/rest:console-comms-rest-itest-mox_webhook"],
  ["comms-rest-readiness-pg", "//backend/crates/comms/rest:console-comms-rest-itest-readiness"],
  ["consulting-rest-audit-atomicity-pg", "//backend/crates/consulting/rest:console-consulting-rest-itest-audit_atomicity"],
  ["dispatch-worker-timer-delivery-pg", "//backend/crates/dispatch/worker:console-dispatch-worker-itest-timer_delivery"],
  ["docs-rest-evidence-rest-rls-surfaces-as-runtime-role-pg", "//backend/crates/docs/rest:console-docs-rest-itest-evidence_rest_rls_surfaces_as_runtime_role"],
  ["finance-gl-adapter-postgres-voucher-rls-and-fsm-as-runtime-role-pg", "//backend/crates/finance-gl/adapter-postgres:console-finance-gl-adapter-postgres-itest-voucher_rls_and_fsm_as_runtime_role"],
  ["financial-adapter-postgres-lifecycle-rls-surfaces-as-runtime-role-pg", "//backend/crates/financial/adapter-postgres:console-financial-adapter-postgres-itest-lifecycle_rls_surfaces_as_runtime_role"],
  ["financial-adapter-postgres-period-lock-blocks-ledger-as-runtime-role-pg", "//backend/crates/financial/adapter-postgres:console-financial-adapter-postgres-itest-period_lock_blocks_ledger_as_runtime_role"],
  ["financial-adapter-postgres-use-cases-pg", "//backend/crates/financial/adapter-postgres:console-financial-adapter-postgres-itest-use_cases"],
  ["financial-rest-purchase-request-list-pg", "//backend/crates/financial/rest:console-financial-rest-itest-purchase_request_list"],
  ["governance-adapter-postgres-approvals-create-as-runtime-role-pg", "//backend/crates/governance/adapter-postgres:console-governance-adapter-postgres-itest-approvals_create_as_runtime_role"],
  ["governance-adapter-postgres-four-eyes-bind-consume-pg", "//backend/crates/governance/adapter-postgres:console-governance-adapter-postgres-itest-four_eyes_bind_consume"],
  ["governance-adapter-postgres-governance-rls-as-runtime-role-pg", "//backend/crates/governance/adapter-postgres:console-governance-adapter-postgres-itest-governance_rls_as_runtime_role"],
  ["identity-adapter-postgres-deactivate-revokes-credentials-pg", "//backend/crates/identity/adapter-postgres:console-identity-adapter-postgres-itest-deactivate_revokes_credentials"],
  ["identity-adapter-postgres-me-workspace-layouts-rls-pg", "//backend/crates/identity/adapter-postgres:console-identity-adapter-postgres-itest-me_workspace_layouts_rls"],
  ["identity-adapter-postgres-region-branch-crud-rls-surfaces-as-runtime-role-pg", "//backend/crates/identity/adapter-postgres:console-identity-adapter-postgres-itest-region_branch_crud_rls_surfaces_as_runtime_role"],
  ["identity-adapter-postgres-subject-authz-versions-freshness-rls-pg", "//backend/crates/identity/adapter-postgres:console-identity-adapter-postgres-itest-subject_authz_versions_freshness_rls"],
  ["inbox-adapter-postgres-inbox-docs-rls-surfaces-as-runtime-role-pg", "//backend/crates/inbox/adapter-postgres:console-inbox-adapter-postgres-itest-inbox_docs_rls_surfaces_as_runtime_role"],
  ["inbox-rest-api-pg", "//backend/crates/inbox/rest:console-inbox-rest-itest-api"],
  ["inspection-adapter-postgres-lifecycle-pg", "//backend/crates/inspection/adapter-postgres:console-inspection-adapter-postgres-itest-lifecycle"],
  ["inspection-adapter-postgres-schedule-window-rls-surfaces-as-runtime-role-pg", "//backend/crates/inspection/adapter-postgres:console-inspection-adapter-postgres-itest-schedule_window_rls_surfaces_as_runtime_role"],
  ["inventory-adapter-postgres-consume-idempotency-concurrency-pg", "//backend/crates/inventory/adapter-postgres:console-inventory-adapter-postgres-itest-consume_idempotency_concurrency"],
  ["leave-adapter-postgres-leave-migration-expand-contract-pg", "//backend/crates/leave/adapter-postgres:console-leave-adapter-postgres-itest-leave_migration_expand_contract"],
  ["leave-adapter-postgres-leave-rls-surfaces-as-runtime-role-pg", "//backend/crates/leave/adapter-postgres:console-leave-adapter-postgres-itest-leave_rls_surfaces_as_runtime_role"],
  ["leave-rest-leave-http-personas-pg", "//backend/crates/leave/rest:console-leave-rest-itest-leave_http_personas"],
  ["messenger-adapter-postgres-parity-tables-rls-as-runtime-role-pg", "//backend/crates/messenger/adapter-postgres:console-messenger-adapter-postgres-itest-parity_tables_rls_as_runtime_role"],
  ["messenger-adapter-postgres-use-cases-pg", "//backend/crates/messenger/adapter-postgres:console-messenger-adapter-postgres-itest-use_cases"],
  ["messenger-rest-api-pg", "//backend/crates/messenger/rest:console-messenger-rest-itest-api"],
  ["notices-adapter-postgres-notices-rls-surfaces-as-runtime-role-pg", "//backend/crates/notices/adapter-postgres:console-notices-adapter-postgres-itest-notices_rls_surfaces_as_runtime_role"],
  ["notices-rest-api-pg", "//backend/crates/notices/rest:console-notices-rest-itest-api"],
  ["notifications-adapter-postgres-notifications-rls-surfaces-as-runtime-role-pg", "//backend/crates/notifications/adapter-postgres:console-notifications-adapter-postgres-itest-notifications_rls_surfaces_as_runtime_role"],
  ["notifications-rest-api-pg", "//backend/crates/notifications/rest:console-notifications-rest-itest-api"],
  ["ontology-adapter-postgres-builtin-catalog-additive-upgrade-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-builtin_catalog_additive_upgrade_as_runtime_role"],
  ["ontology-adapter-postgres-c-chain-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-c_chain_as_runtime_role"],
  ["ontology-adapter-postgres-config-object-types-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-config_object_types_as_runtime_role"],
  ["ontology-adapter-postgres-instances-residual-filter-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-instances_residual_filter_as_runtime_role"],
  ["ontology-adapter-postgres-instances-rls-surfaces-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-instances_rls_surfaces_as_runtime_role"],
  ["ontology-adapter-postgres-key-revision-migration-upgrade-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-key_revision_migration_upgrade"],
  ["ontology-adapter-postgres-key-write-cas-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-key_write_cas_as_runtime_role"],
  ["ontology-adapter-postgres-niche-config-object-types-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-niche_config_object_types_as_runtime_role"],
  ["ontology-adapter-postgres-projected-instances-read-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-projected_instances_read_as_runtime_role"],
  ["ontology-adapter-postgres-property-derivation-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-property_derivation_as_runtime_role"],
  ["ontology-adapter-postgres-property-link-sync-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-property_link_sync_as_runtime_role"],
  ["ontology-adapter-postgres-registry-rls-surfaces-as-runtime-role-pg", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-registry_rls_surfaces_as_runtime_role"],
  ["payroll-adapter-postgres-payroll-lifecycle-rls-as-runtime-role-pg", "//backend/crates/payroll/adapter-postgres:console-payroll-adapter-postgres-itest-payroll_lifecycle_rls_as_runtime_role"],
  ["payroll-rest-api-pg", "//backend/crates/payroll/rest:console-payroll-rest-itest-api"],
  ["payroll-rest-payslip-draft-api-pg", "//backend/crates/payroll/rest:console-payroll-rest-itest-payslip_draft_api"],
  ["payroll-rest-run-lifecycle-api-pg", "//backend/crates/payroll/rest:console-payroll-rest-itest-run_lifecycle_api"],
  ["platform-auth-rest-dev-auth-absence-pg", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-dev_auth_absence"],
  ["platform-auth-refresh-tokens-pg", "//backend/crates/platform/auth:console-platform-auth-itest-refresh_tokens"],
  ["platform-auth-webauthn-ceremony-pg", "//backend/crates/platform/auth:console-platform-auth-itest-webauthn_ceremony"],
  ["platform-auth-webauthn-ceremony-replay-pg", "//backend/crates/platform/auth:console-platform-auth-itest-webauthn_ceremony_replay"],
  ["platform-authz-rest-cedar-authoring-rls-as-runtime-role-pg", "//backend/crates/platform/authz-rest:console-platform-authz-rest-itest-cedar_authoring_rls_as_runtime_role"],
  ["platform-authz-rest-decision-feed-as-runtime-role-pg", "//backend/crates/platform/authz-rest:console-platform-authz-rest-itest-decision_feed_as_runtime_role"],
  ["platform-authz-policy-pg", "//backend/crates/platform/authz:console-platform-authz-itest-policy"],
  ["platform-db-attendance-console-migration-contract-pg", "//backend/crates/platform/db:console-platform-db-itest-attendance_console_migration_contract"],
  ["platform-db-code-issuance-pg", "//backend/crates/platform/db:console-platform-db-itest-code_issuance"],
  ["platform-db-group-resolvers-pg", "//backend/crates/platform/db:console-platform-db-itest-group_resolvers"],
  ["platform-db-lifecycle-maker-checker-pg", "//backend/crates/platform/db:console-platform-db-itest-lifecycle_maker_checker"],
  ["platform-db-m2-flag-on-runtime-drain-pg", "//backend/crates/platform/db:console-platform-db-itest-m2_flag_on_runtime_drain"],
  ["platform-db-period-locks-and-lifecycle-pg", "//backend/crates/platform/db:console-platform-db-itest-period_locks_and_lifecycle"],
  ["platform-db-personal-data-classification-pg", "//backend/crates/platform/db:console-platform-db-itest-personal_data_classification"],
  ["platform-group-lib-pg", "//backend/crates/platform/group:console-platform-group-unit"],
  ["platform-jobs-apalis-adapter-pg", "//backend/crates/platform/jobs:console-platform-jobs-itest-apalis_adapter"],
  ["platform-jobs-apalis-schema-contract-pg", "//backend/crates/platform/jobs:console-platform-jobs-itest-apalis_schema_contract"],
  ["platform-platform-rest-onboard-seeds-config-objects-pg", "//backend/crates/platform/platform-rest:console-platform-rest-itest-onboard_seeds_config_objects"],
  ["platform-platform-rest-ops-dashboard-pg", "//backend/crates/platform/platform-rest:console-platform-rest-itest-ops_dashboard"],
  ["platform-platform-rest-platform-groups-pg", "//backend/crates/platform/platform-rest:console-platform-rest-itest-platform_groups"],
  ["platform-platform-rest-view-as-pg", "//backend/crates/platform/platform-rest:console-platform-rest-itest-view_as"],
  ["platform-provisioning-bootstrap-passkey-pg", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-bootstrap_passkey"],
  ["platform-provisioning-bootstrap-passkey-replay-pg", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-bootstrap_passkey_replay"],
  ["platform-provisioning-roster-import-pg", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-roster_import"],
  ["platform-provisioning-self-enroll-handoff-as-runtime-role-pg", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-self_enroll_handoff_as_runtime_role"],
  ["platform-realtime-postgres-bridge-pg", "//backend/crates/platform/realtime:console-platform-realtime-itest-postgres_bridge"],
  ["platform-storage-lib-pg", "//backend/crates/platform/storage:console-platform-storage-unit"],
  ["platform-storage-evidence-processing-rls-surfaces-as-runtime-role-pg", "//backend/crates/platform/storage:console-platform-storage-itest-evidence_processing_rls_surfaces_as_runtime_role"],
  ["policy-adapter-postgres-draft-storage-pg", "//backend/crates/policy/adapter-postgres:console-policy-adapter-postgres-itest-draft_storage"],
  ["registry-adapter-postgres-create-rls-surfaces-as-runtime-role-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-create_rls_surfaces_as_runtime_role"],
  ["registry-adapter-postgres-equipment-list-rls-as-runtime-role-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-equipment_list_rls_as_runtime_role"],
  ["registry-adapter-postgres-equipment-lookup-normalization-rls-as-runtime-role-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-equipment_lookup_normalization_rls_as_runtime_role"],
  ["registry-adapter-postgres-equipment-versioning-as-runtime-role-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-equipment_versioning_as_runtime_role"],
  ["registry-adapter-postgres-master-list-import-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-master_list_import"],
  ["registry-adapter-postgres-master-list-import-rls-as-runtime-role-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-master_list_import_rls_as_runtime_role"],
  ["registry-adapter-postgres-site-address-postal-roundtrip-rls-as-runtime-role-pg", "//backend/crates/registry/adapter-postgres:console-registry-adapter-postgres-itest-site_address_postal_roundtrip_rls_as_runtime_role"],
  ["registry-rest-equipment-admin-pg", "//backend/crates/registry/rest:console-registry-rest-itest-equipment_admin"],
  ["reporting-adapter-postgres-excel-exports-pg", "//backend/crates/reporting/adapter-postgres:console-reporting-adapter-postgres-itest-excel_exports"],
  ["reporting-adapter-postgres-kpi-golden-dataset-pg", "//backend/crates/reporting/adapter-postgres:console-reporting-adapter-postgres-itest-kpi_golden_dataset"],
  ["reporting-adapter-postgres-ops-summary-pg", "//backend/crates/reporting/adapter-postgres:console-reporting-adapter-postgres-itest-ops_summary"],
  ["reporting-adapter-postgres-work-diary-rls-surfaces-as-runtime-role-pg", "//backend/crates/reporting/adapter-postgres:console-reporting-adapter-postgres-itest-work_diary_rls_surfaces_as_runtime_role"],
  ["sales-adapter-postgres-inquiry-rls-surfaces-as-runtime-role-pg", "//backend/crates/sales/adapter-postgres:console-sales-adapter-postgres-itest-inquiry_rls_surfaces_as_runtime_role"],
  ["sales-adapter-postgres-sales-store-pg", "//backend/crates/sales/adapter-postgres:console-sales-adapter-postgres-itest-sales_store"],
  ["support-adapter-postgres-assignee-name-join-rls-surfaces-as-runtime-role-pg", "//backend/crates/support/adapter-postgres:console-support-adapter-postgres-itest-assignee_name_join_rls_surfaces_as_runtime_role"],
  ["support-adapter-postgres-create-internal-ticket-rls-surfaces-as-runtime-role-pg", "//backend/crates/support/adapter-postgres:console-support-adapter-postgres-itest-create_internal_ticket_rls_surfaces_as_runtime_role"],
  ["support-adapter-postgres-support-tickets-pg", "//backend/crates/support/adapter-postgres:console-support-adapter-postgres-itest-support_tickets"],
  ["support-rest-lib-pg", "//backend/crates/support/rest:console-support-rest-unit"],
  ["support-rest-authz-pg", "//backend/crates/support/rest:console-support-rest-itest-authz"],
  ["support-rest-intake-pg", "//backend/crates/support/rest:console-support-rest-itest-intake"],
  ["todos-adapter-postgres-todos-rls-surfaces-as-runtime-role-pg", "//backend/crates/todos/adapter-postgres:console-todos-adapter-postgres-itest-todos_rls_surfaces_as_runtime_role"],
  ["workflow-adapter-postgres-notification-bridge-pg", "//backend/crates/workflow/adapter-postgres:console-workflow-runtime-adapter-postgres-itest-notification_bridge"],
  ["workflow-adapter-postgres-payroll-drain-period-lock-pg", "//backend/crates/workflow/adapter-postgres:console-workflow-runtime-adapter-postgres-itest-payroll_drain_period_lock"],
  ["workorder-adapter-postgres-m2-flag-off-parity-pg", "//backend/crates/workorder/adapter-postgres:console-workorder-adapter-postgres-itest-m2_flag_off_parity"],
  ["workorder-adapter-postgres-rls-read-surfaces-as-runtime-role-pg", "//backend/crates/workorder/adapter-postgres:console-workorder-adapter-postgres-itest-rls_read_surfaces_as_runtime_role"],
  ["workorder-adapter-postgres-use-cases-pg", "//backend/crates/workorder/adapter-postgres:console-workorder-adapter-postgres-itest-use_cases"],
  ["workorder-rest-mobile-device-registration-pg", "//backend/crates/workorder/rest:console-workorder-rest-itest-mobile_device_registration"],
  ["workorder-rest-mobile-evidence-pg", "//backend/crates/workorder/rest:console-workorder-rest-itest-mobile_evidence"],
  ["workorder-rest-mobile-sync-pg", "//backend/crates/workorder/rest:console-workorder-rest-itest-mobile_sync"],

  // Tenant-isolation and PII tranche, 2026-07-31.
  ["platform-db-rls-isolation", "//backend/crates/platform/db:console-platform-db-itest-rls_isolation"],
  ["platform-db-rls-rollout-isolation", "//backend/crates/platform/db:console-platform-db-itest-rls_rollout_isolation"],
  ["platform-audit-chain-rls", "//backend/crates/platform/audit-chain:console-platform-audit-chain-itest-audit_chain_rls"],
  ["platform-provisioning-rls-auth-chain", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-rls_auth_chain_as_runtime_role"],
  ["platform-rest-remove-tenant", "//backend/crates/platform/platform-rest:console-platform-rest-itest-remove_tenant"],
  ["compliance-location-consent-status-rls", "//backend/crates/compliance/adapter-postgres:console-compliance-adapter-postgres-itest-location_consent_status_rls_as_runtime_role"],
  ["compliance-location-store", "//backend/crates/compliance/adapter-postgres:console-compliance-adapter-postgres-itest-location_store"],
  ["payroll-rls-surfaces", "//backend/crates/payroll/adapter-postgres:console-payroll-adapter-postgres-itest-payroll_rls_surfaces_as_runtime_role"],

  ["dispatch-p1-postgres", "//backend/crates/dispatch/adapter-postgres:console-dispatch-adapter-postgres-itest-p1_dispatch"],
  ["attendance-cancel-substitution-postgres", "//backend/crates/attendance/adapter-postgres:console-attendance-adapter-postgres-itest-cancel_substitution"],
  ["attendance-concurrency-postgres", "//backend/crates/attendance/adapter-postgres:console-attendance-adapter-postgres-itest-concurrency"],
  ["app-inline-postgres", "//backend/app:console-app-itest-inline-postgres"],
  ["app-dev-auth-persona-guard-postgres", "//backend/app:console-app-itest-dev_auth_persona_guard_feature"],
  ["auth-rest-dev-auth-inline-postgres", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-dev-auth-postgres"],
  ["auth-rest-dev-auth-session-postgres", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-dev_auth_session"],
  ["auth-rest-dev-auth-group-admin-postgres", "//backend/crates/platform/auth-rest:console-platform-auth-rest-itest-group_admin_tenant_context"],
  ["provisioning-dev-principal-upsert-race-postgres", "//backend/crates/platform/provisioning:console-platform-provisioning-itest-dev_principal_upsert_race"],
  ["app-evaluation-cycle-api-postgres", "//backend/app:console-app-itest-evaluation_cycle_api"],
  ["ontology-builtin-catalog-additive-upgrade-postgres", "//backend/crates/ontology/adapter-postgres:console-ontology-adapter-postgres-itest-builtin_catalog_additive_upgrade_as_runtime_role"],
  ["app-org-change-api-postgres", "//backend/app:console-app-itest-org_change_api"],
  ["app-purchase-request-collection-api-postgres", "//backend/app:console-app-itest-purchase_request_collection_api"],
  ["app-workflow-object-context-api-postgres", "//backend/app:console-app-itest-workflow_object_context_api"],
  ["equipment-3r-http-postgres", "//backend/crates/equipment/rest:console-equipment-rest-itest-equipment_3r_http"],
  ["app-equipment-3r-api-postgres", "//backend/app:console-app-itest-equipment_3r_api"],
  ["ontology-object-type-lifecycle-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-object_type_lifecycle_over_http"],
  ["ontology-object-type-cas-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-object_type_cas_as_runtime_role"],
  ["ontology-publish-auto-create-action-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-publish_auto_create_action_as_runtime_role"],
  ["ontology-object-policy-attach-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-object_policy_attach_as_runtime_role"],
  ["ontology-action-execute-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-action_execute_as_runtime_role"],
  ["ontology-gaps-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-ont_gaps_as_runtime_role"],
  ["ontology-projected-dispatch-postgres", "//backend/crates/ontology/rest:console-ontology-rest-itest-projected_dispatch_as_runtime_role"],
  ["platform-erasure-ledger-postgres", "//backend/crates/platform/erasure-ledger:console-platform-erasure-ledger-itest-erasure_ledger_as_runtime_role"],
];
const postgresWrapperLoader = "run_test_with_postgres_env.sh";
const postgresWrapperLabels = '["test.integration", "resource.postgres", "needs-postgres"]';
const postgresWrapperBuildFile = readFileSync(new URL("../tools/buck/BUCK", import.meta.url), "utf8");
const freeRunnerDiskActionFile = readFileSync(
  new URL("../.github/actions/free-runner-disk/action.yml", import.meta.url),
  "utf8",
);
// GENERATED by tools/buck/gen_first_party.py from the crate's tests/ directory,
// which is what makes the check below a TOTALITY check rather than a second
// hand-kept list: adding a test file adds a target here without anyone deciding to.
const ontologyRestCrateBuildFile = readFileSync(
  new URL("../backend/crates/ontology/rest/BUCK", import.meta.url),
  "utf8",
);
const requiredPreflightCommands = [
  "tools/buck/preflight.sh",
  "npm run check:foundation-gates",
  reasoningLensTestCommand,
  ciPreflightTestCommand,
  consoleRouteInventoryTestCommand,
  buckPostgresEnvironmentTestCommand,
  buckPostgresHarnessTestCommand,
  "npm run check:ci-preflight",
  "npm run check:package-lock",
  "cargo metadata --manifest-path backend/Cargo.toml --locked --format-version=1 >/dev/null",
  // Locked on arrival. repo-gates taught this repository that a step wired into
  // ci.yml is not thereby protected — deleting `run: npm run check:adrs` from it
  // returned zero preflight failures. These two are the only thing standing
  // between the Buck2 exit and a silently smaller test population, so they are
  // locked in the same commit that moves them here rather than a commit later.
  "npm run check:executed-tests",
  "npm run check:test-credentials",
];
const protectedJobs = [
  "backend",
  "dev-up-smoke",
  "repo-gates",
  "api-contract",
  "kubernetes-manifests",
  "generated-face-authority",
  "domain-unit",
  "postgres-domain-reachability",
  "company-conformance",
];

function runContract(kind, name, run, options = {}) {
  return {
    kind,
    name,
    run,
    if: options.if ?? null,
    workingDirectory: options.workingDirectory ?? null,
    shell: options.shell ?? null,
  };
}

function runDigestContract(kind, name, runSha256, options = {}) {
  return {
    kind,
    name,
    runSha256,
    if: options.if ?? null,
    workingDirectory: options.workingDirectory ?? null,
    shell: options.shell ?? null,
  };
}

const setupRun = (name, run, options) => runContract("setup", name, run, options);
const proofRun = (name, run, options) => runContract("proof", name, run, options);
const cleanupRun = (name, run, options) => runContract("cleanup", name, run, options);
const setupDigest = (name, digest, options) => runDigestContract("setup", name, digest, options);
const proofDigest = (name, digest, options) => runDigestContract("proof", name, digest, options);

// This is the complete ordered run-step surface for every current and planned
// required job. Setup prepares a proof but is not itself acceptance evidence;
// proof supplies that evidence, and cleanup is the fail-safe teardown. Multiline
// programs are locked by a digest of js-yaml's exact parsed `run` string so shell
// text cannot be retained behind an early successful exit or changed under a
// familiar name.
const requiredJobRunContracts = Object.freeze({
  preflight: [
    setupDigest("Derive exact console C/T/M train", "b8e69e979347fa526773bb2a740bdb198be414077281026c23be96501ffd4da1", { if: consolePrCondition, shell: "bash" }),
    setupRun("Install pinned DotSlash runtime", "tools/buck/install_dotslash.sh"),
    setupRun("Install workspace dependencies", "npm ci"),
    proofRun("Cheap Buck2 generated-face admission", "tools/buck/preflight.sh"),
    proofRun("Foundation gate contract", "npm run check:foundation-gates"),
    proofRun("Reasoning lens contract regression", "node --test scripts/check-reasoning-lens-contract.test.mjs"),
    proofDigest("Reasoning lens changed-record admission", "b4d78de511586e6f3cb7edafcf780fbc0361279dc8f0fe544b6128cfad9d3ab9", { shell: "bash" }),
    proofRun("Console truth-ledger exact-M admission", "npm run check:console-truth-ledger", { if: consolePrCondition }),
    proofRun("Console fanout planner exact-M admission", 'node scripts/console/plan-fanout.mjs --candidate "$CONSOLE_CANDIDATE_SHA" --authority-tip "$CONSOLE_AUTHORITY_TIP_SHA" --synthetic-merge "$CONSOLE_SYNTHETIC_MERGE_SHA"', { if: consolePrCondition }),
    proofRun("CI preflight contract tests", "node --test scripts/check-ci-preflight.test.mjs"),
    proofRun("Console route inventory regression", "node --test scripts/console/route-inventory.test.mjs"),
    proofRun("Console authority-train regression", "node --test scripts/console/verify-console-authority-train.test.mjs"),
    proofRun("Console PR authority bootstrap regression", "node --test scripts/console/verify-console-pr-authority-bootstrap.test.mjs"),
    proofRun("Executed-tests baseline set regression", "npm run test:executed-tests-baseline"),
    proofRun("Local CI mirror contract", "node --test scripts/verify.test.mjs"),
    proofRun("Console truth-ledger validator exact-M regression", "node --test scripts/console/validate-console-truth-ledger.test.mjs"),
    proofRun("Console fanout planner exact-M regression", "node --test scripts/console/plan-fanout.test.mjs"),
    proofRun("Buck PostgreSQL environment wrapper regression", "tools/buck/run_test_with_postgres_env.test.sh"),
    proofRun("Buck disposable PostgreSQL harness regression", "tools/buck/test_needs_postgres.test.sh"),
    proofRun("CI preflight contract", "npm run check:ci-preflight"),
    proofRun("Canonical npm lockfile", "npm run check:package-lock"),
    proofRun("Cargo.lock consistency", "cargo metadata --manifest-path backend/Cargo.toml --locked --format-version=1 >/dev/null"),
    proofRun("Executed-tests ratchet — a test binary must have a path from a workflow step", "npm run check:executed-tests"),
    proofRun("JavaScript test reachability ratchet", "npm run check:js-test-reachability"),
    proofRun("JavaScript test reachability unit tests", "npm run test:js-test-reachability"),
    proofRun("Workflow test-runner credential literals", "npm run check:test-credentials"),
  ],
  "domain-unit": [
    proofDigest("Domain crate unit tests", "5a1888f6a67b92f3448b4a5e532170a3eb53bf1f12679a755670235d764d1edc"),
  ],
  backend: [
    setupRun("Install pinned DotSlash runtime", "../tools/buck/install_dotslash.sh"),
    proofRun("rustfmt check", "cargo fmt --all -- --check"),
    proofRun("clippy -D warnings", "SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings"),
    proofRun("Layer-boundary gate", "cargo run -p console-gate-layer-boundary"),
    proofRun("Audit-coverage gate", "cargo run -p console-gate-audit-coverage"),
    proofRun("Migration-safety gate", "cargo run -p console-gate-migration-safety"),
    proofRun("Tenant-isolation gate", "cargo run -p console-gate-tenant-isolation"),
    proofRun("PII-no-logs gate", "cargo run -p console-gate-pii-no-logs"),
    proofRun("RLS-arming gate", "cargo run -p console-gate-rls-arming"),
    proofRun("Dev-auth-absence gate", "cargo run -p console-gate-dev-auth-absence"),
    proofRun("IaC tier-discipline gate", "cargo run -p console-gate-iac-tier"),
    proofRun("Fabricated-branch gate", "cargo run -p console-gate-fabricated-branch"),
    proofRun("Personal-data-classification gate", "cargo run -p console-gate-personal-data-classification"),
    proofDigest("Buck2 CI-gate mutation suites — every gate proven to still reject", "f6614509bd73220754a83d449b8bf422e616309ba48965f730f0d3dcff9d2cf4", { workingDirectory: "." }),
    proofRun("PR 473 migration operational contract tests", "python3 scripts/check-pr473-migration-operational.test.py -v", { workingDirectory: "." }),
    setupDigest("Reconcile portable PostgreSQL role topology", "5da0f2d8c399657dbc0a9d358c81d71399af1ea6c659074a365653db21fcaded"),
    proofRun("PR 473 migration operational gate", "npm run check:pr473-migration-operational", { workingDirectory: "." }),
    proofDigest("Boot smoke — migrate + serve + /readyz", "d51d75f8cd49be1557c5b5c1f5f641345bc82f842d2384e9608e9872b0714d79"),
    proofDigest("Buck2 dev-auth feature PostgreSQL suites", "f059b50b432f8cafc4e58b14272fe76f5dd3d21842b8683f08c0a5f1f7a84001", { workingDirectory: "." }),
    proofRun("Buck2 platform-authz unit suite", "env -u DATABASE_URL tools/buck2 test //backend/crates/platform/authz:console-platform-authz-unit", { workingDirectory: "." }),
    proofRun("Buck2 console-app unit suite", "env -u DATABASE_URL tools/buck2 test //backend/app:console-app-unit", { workingDirectory: "." }),
    proofRun("Buck2 console-app OpenAPI drift suite", "env -u DATABASE_URL tools/buck2 test //backend/app:console-app-itest-openapi_drift", { workingDirectory: "." }),
    proofDigest("Buck2 console-app inline PostgreSQL suites", "2a59f90874addb48871158b672a9016159caba7382f49252d43beba2372daf63", { workingDirectory: "." }),
  ],
  "dev-up-smoke": [
    proofRun("dev-up compose contract unit test", "node --test scripts/dev-up-compose.test.mjs"),
    setupRun("Install pinned DotSlash runtime", "tools/buck/install_dotslash.sh"),
    proofRun("PostgreSQL topology integration regression", "ops/postgres-topology.integration.test.sh"),
    proofRun("dev-up bootstrap (compose deps + migrate + backend readyz)", "node scripts/dev-up.mjs bootstrap"),
    proofRun("Confirm /readyz reachable", 'curl -fsS "http://127.0.0.1:${CONSOLE_DEV_HTTP_PORT:-8090}/readyz"'),
    cleanupRun("dev-up down", "node scripts/dev-up.mjs down", { if: "always()" }),
  ],
  "kubernetes-manifests": [
    setupDigest("Install kubectl (for kustomize renderer)", "ed237728d562e10247b2ae17f435525b3b71d94efe5c0c63afa3c73cd16e096b"),
    setupDigest("Install kustomize (NetworkPolicy static render proof)", "4cc1bf875027906f3b8a9878c0f13ce9b1438490390d858088c5ca279e4b9b3c"),
    proofRun("Governed command-database DARK wiring regression", "node --test scripts/check-command-database-wiring.test.mjs"),
    proofRun("Render manifests and NetworkPolicy enforcement preflight", "npm run check:k8s", { if: "${{ !cancelled() }}" }),
    proofRun("Production hardening contract", "npm run check:production-hardening", { if: "${{ !cancelled() }}" }),
    proofRun("Production hardening regression tests", "npm run test:production-hardening", { if: "${{ !cancelled() }}" }),
  ],
  "repo-gates": [
    setupRun("Install workspace dependencies", "npm ci"),
    proofRun("ADR governance tests", "npm run test:adrs", { if: "${{ !cancelled() }}" }),
    proofRun("ADR governance gate", "npm run check:adrs", { if: "${{ !cancelled() }}" }),
    proofRun("Documentation link tests", "node --test scripts/check-doc-links.test.mjs", { if: "${{ !cancelled() }}" }),
    proofRun("Documentation manifest gate", "npm run check:doc-manifest", { if: "${{ !cancelled() }}" }),
    proofRun("Documentation local-link gate", "npm run check:doc-links", { if: "${{ !cancelled() }}" }),
    proofRun("Doc citations — every code citation must resolve", "npm run check:doc-citations", { if: "${{ !cancelled() }}" }),
    proofRun("Foundation gate contract", "npm run check:foundation-gates", { if: "${{ !cancelled() }}" }),
    proofRun("Canonical npm lockfile", "npm run check:package-lock", { if: "${{ !cancelled() }}" }),
    proofRun("Shared text gate unit tests", "npm run test:text-gate", { if: "${{ !cancelled() }}" }),
    proofRun("Gate-input provenance instrument", "npm run check:gate-input-provenance", { if: "${{ !cancelled() }}" }),
    proofRun("Gate-input provenance unit tests", "npm run test:gate-input-provenance", { if: "${{ !cancelled() }}" }),
    proofRun("G004 identity group org people policy foundation gate", "npm run check:g004-identity-foundation", { if: "${{ !cancelled() }}" }),
    proofRun("G005 workflow approval Work Hub lifecycle gate", "npm run check:g005-workflow-lifecycle", { if: "${{ !cancelled() }}" }),
    proofRun("Workflow runtime spine gate", "npm run check:workflow-runtime-spine", { if: "${{ !cancelled() }}" }),
    proofRun("Workflow runtime M2 strangler dark-landing gate", "npm run check:workflow-runtime-m2-strangler", { if: "${{ !cancelled() }}" }),
    proofRun("Workflow runtime M2 Cedar-guard observe-and-record gate", "npm run check:workflow-runtime-m2-cedar-guards", { if: "${{ !cancelled() }}" }),
    proofRun("Workflow runtime M2 flag-ON runtime gate", "npm run check:workflow-runtime-m2-runtime", { if: "${{ !cancelled() }}" }),
    proofRun("Workflow runtime M2 outbox-drainer transactional-idempotency gate", "npm run check:workflow-runtime-m2-drainer", { if: "${{ !cancelled() }}" }),
    proofRun("G006 asset equipment dispatch lifecycle gate", "npm run check:g006-asset-dispatch-lifecycle", { if: "${{ !cancelled() }}" }),
    proofRun("G007 collaboration mail calendar poll mobile lifecycle gate", "npm run check:g007-collaboration-mobile-lifecycle", { if: "${{ !cancelled() }}" }),
    proofRun("G008 import HR payroll readiness gate", "npm run check:g008-payroll-readiness", { if: "${{ !cancelled() }}" }),
    proofRun("People HR lifecycle maturity gate", "npm run check:people-hr-maturity", { if: "${{ !cancelled() }}" }),
    proofRun("Payroll release-gate contract", "npm run check:payroll-release-gate", { if: "${{ !cancelled() }}" }),
    proofRun("Undeclared imports — every bare specifier must be declared", "npm run check:undeclared-imports", { if: "${{ !cancelled() }}" }),
    proofRun("Request-body contract — spec fields must exist on the handler", "npm run check:request-body-contract", { if: "${{ !cancelled() }}" }),
  ],
  "api-contract": [
    setupRun("Install Node tooling", "npm ci"),
    proofRun("Platform contract drift gate", "npm run check:platform-contract-drift", { if: "${{ !cancelled() }}" }),
    proofRun("Employee import replay contract", "npm run test:employee-import-contract", { if: "${{ !cancelled() }}" }),
    proofRun("Ontology write precondition contract", "npm run test:ontology-write-precondition", { if: "${{ !cancelled() }}" }),
  ],
  "generated-face-authority": [
    setupRun("Install pinned DotSlash runtime", "tools/buck/install_dotslash.sh"),
    setupDigest("Install lock-pinned Reindeer Rust toolchain", "e138ce62e419d3461df6e45108f0cb5a032e966486527b918d504c6ae604ed4e", { shell: "bash" }),
    setupRun("Install workspace dependencies", "npm ci"),
    proofRun("Full generated-face closure", "tools/buck/preflight.sh --full-generated-faces"),
  ],
  "company-conformance": [
    setupRun("Install pinned DotSlash runtime", "tools/buck/install_dotslash.sh"),
    proofDigest("Company conformance against disposable PostgreSQL", "f2e478d7571d3dd31977783d4a13deeffd8bb09e045cdb8e3d205528ea6fe3c7"),
  ],
  "postgres-domain-reachability": [
    setupRun("Install pinned DotSlash runtime", "tools/buck/install_dotslash.sh"),
    proofDigest("Serialized disposable PostgreSQL integration targets", "37d20ed4ab222157469f5b3dddc8993aec6ef49f2ad582166499f8ad3eea56b0"),
  ],
});

function actionStep(index, name, uses, withInputs) {
  return {
    index,
    step: withInputs === undefined
      ? { name, uses }
      : { name, uses, with: withInputs },
  };
}

// Action setup is executable too. Pin the full parsed step object, including
// action identity and inputs, and its position among run steps. This prevents a
// skipped checkout/toolchain/cache action from inheriting whatever happens to
// be installed on a hosted runner and presenting that accident as proof.
const requiredJobActionContracts = Object.freeze({
  preflight: [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false, "fetch-depth": 0 }),
    actionStep(3, "Install Rust toolchain for Cargo.lock consistency", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1" }),
    actionStep(4, "Set up Node.js", "actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e", { "node-version": "24.16.0", cache: "npm" }),
  ],
  "domain-unit": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false }),
    actionStep(1, "Install Rust toolchain (pinned via rust-toolchain.toml)", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1" }),
    actionStep(2, "Cache Rust dependencies + build artifacts", "Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4", { workspaces: "backend", "shared-key": "backend-cargo", "cache-all-crates": "true", "save-if": false }),
  ],
  backend: [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false }),
    actionStep(2, "Free runner disk for Rust backend", "./.github/actions/free-runner-disk"),
    actionStep(3, "Install Rust toolchain (pinned via rust-toolchain.toml)", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1", components: "rustfmt, clippy" }),
    actionStep(4, "Cache Rust dependencies + build artifacts", "Swatinem/rust-cache@c19371144df3bb44fab255c43d04cbc2ab54d1c4", { workspaces: "backend", "shared-key": "backend-cargo", "cache-all-crates": "true", "save-if": "${{ github.ref == 'refs/heads/main' }}" }),
  ],
  "dev-up-smoke": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false }),
    actionStep(3, "Free runner disk for Rust backend", "./.github/actions/free-runner-disk"),
    actionStep(4, "Install Rust toolchain (pinned via rust-toolchain.toml)", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1" }),
    actionStep(5, "Set up Node.js", "actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e", { "node-version": "24", cache: "npm" }),
  ],
  "kubernetes-manifests": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "fetch-depth": 0 }),
  ],
  "repo-gates": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0"),
    actionStep(1, "Set up Node.js", "actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e", { "node-version": "24.16.0", cache: "npm" }),
  ],
  "api-contract": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0"),
    actionStep(1, "Set up Node.js", "actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e", { "node-version": "24", cache: "npm" }),
  ],
  "generated-face-authority": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false }),
    actionStep(2, "Set up Node.js", "actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e", { "node-version": "24.16.0", cache: "npm" }),
    actionStep(3, "Set up Java", "actions/setup-java@1bcf9fb12cf4aa7d266a90ae39939e61372fe520", { distribution: "temurin", "java-version": "21" }),
    actionStep(4, "Install Rust toolchain (pinned via rust-toolchain.toml)", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1" }),
  ],
  "company-conformance": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false }),
    actionStep(2, "Free runner disk for PostgreSQL Buck2 tests", "./.github/actions/free-runner-disk"),
    actionStep(3, "Install Rust toolchain (pinned via rust-toolchain.toml)", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1" }),
  ],
  "postgres-domain-reachability": [
    actionStep(0, "Checkout", "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", { "persist-credentials": false }),
    actionStep(2, "Free runner disk for PostgreSQL Buck2 tests", "./.github/actions/free-runner-disk"),
    actionStep(3, "Install Rust toolchain (pinned via rust-toolchain.toml)", "dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", { toolchain: "1.97.1" }),
  ],
});

// Digest the complete parsed job envelope except `steps`: runner selection,
// needs, timeout, services, environment, defaults, container, strategy and
// permissions are all executable inputs and must change deliberately together.
const requiredJobMetadataSha256 = Object.freeze({
  preflight: "de92ba05f83032730db75242c9422c62ee5957433ab408ff5469215a85626f63",
  "domain-unit": "4948a02022fffb8b39aa14b4cb9ee3f776fe20c04844942dedd31f90ebe90bef",
  backend: "f4f6b9faa5c4382a00d5639bebfb9ab8db664ecf38b79752d80afa567161393f",
  "dev-up-smoke": "39ca186b8c6093adb4f30f8b2ed82c3eabb34fc5b9721652757d34a86c7922d8",
  "kubernetes-manifests": "1b215a62dac6d9a3decea6d6912792de3d033986833356b403fb157a15cb8b96",
  "repo-gates": "da8a07f3a19a6f46a5901e6a6d8eac2f7f1c11f52818b7dea25caf362335ee92",
  "api-contract": "101b70d29b1776058160ea23296e707a4f682f5987a9873371cb57180a737d41",
  "generated-face-authority": "a9440d3b0b2e351b00a75ded87623c0c776a6dd776b2d8529f23403a1df0c5f6",
  "company-conformance": "bae484f4aea8b0b1ce591642e1b06bd61c0d61d1d50a029d39b4edf864877484",
  "postgres-domain-reachability": "09f80c662ebd37fe076e67d087c4f20c206a2f638ada7918049aa94bce58365c",
});

const workflowExecutionEnvelopeSha256 = "e91330f0f5ccd53cb457ef43231e1c8e59d9f986ec7a2fa68f98e93665b439bd";
const freeRunnerDiskActionSha256 = "1c1a2307321f732c3dcd67e3af2f33a771ce5b81ea814445390b65946b52fc8f";
const exactCiJobIds = Object.freeze([
  "api-contract",
  "backend",
  "company-conformance",
  "dev-up-smoke",
  "domain-unit",
  "generated-face-authority",
  "kubernetes-manifests",
  "postgres-domain-reachability",
  "preflight",
  "repo-gates",
  "required-ci",
]);

const requiredCiAggregator = Object.freeze({
  name: "Required / CI",
  needs: [
    "preflight",
    "domain-unit",
    "postgres-domain-reachability",
    "company-conformance",
    "generated-face-authority",
    "backend",
    "dev-up-smoke",
    "repo-gates",
    "api-contract",
    "kubernetes-manifests",
  ],
  if: "${{ always() }}",
  "runs-on": "ubuntu-latest",
  "timeout-minutes": 5,
  steps: [{
    name: "Require every CI proof to succeed",
    run: [
      'test "${{ needs.preflight.result }}" = success &&',
      '  test "${{ needs.domain-unit.result }}" = success &&',
      '  test "${{ needs.postgres-domain-reachability.result }}" = success &&',
      '  test "${{ needs.company-conformance.result }}" = success &&',
      '  test "${{ needs.generated-face-authority.result }}" = success &&',
      '  test "${{ needs.backend.result }}" = success &&',
      '  test "${{ needs.dev-up-smoke.result }}" = success &&',
      '  test "${{ needs.repo-gates.result }}" = success &&',
      '  test "${{ needs.api-contract.result }}" = success &&',
      '  test "${{ needs.kubernetes-manifests.result }}" = success\n',
    ].join("\n"),
  }],
});

// Environment and job defaults are executable inputs: BASH_ENV, NODE_OPTIONS,
// PATH and RUSTC_WRAPPER can replace the program a visually exact `run:` line
// actually reaches. Keep the small amount of intentional metadata explicit and
// reject every other job/step injection on the jobs this preflight protects.
const protectedJobExecutionMetadata = {
  preflight: {
    stepEnv: [{
      name: reasoningLensAdmissionName,
      env: Object.fromEntries(
        reasoningLensAdmissionEnvironment.map((entry) => entry.split(": ", 2)),
      ),
    }],
  },
  "domain-unit": {},
  "postgres-domain-reachability": {},
  "company-conformance": {},
  "generated-face-authority": {},
  backend: {
    env: {
      DATABASE_URL: "postgres://postgres:postgres@localhost:5432/console_ci",
      SQLX_OFFLINE: "true",
      CARGO_INCREMENTAL: "0",
      CARGO_PROFILE_DEV_DEBUG: "0",
      CARGO_PROFILE_TEST_DEBUG: "0",
    },
    defaults: { run: { "working-directory": "backend" } },
  },
  "dev-up-smoke": {
    env: {
      CARGO_INCREMENTAL: "0",
      CARGO_PROFILE_DEV_DEBUG: "0",
    },
  },
  "repo-gates": {},
  "api-contract": {},
  "kubernetes-manifests": {
    stepEnv: [{
      name: "Install kubectl (for kustomize renderer)",
      env: { KUBECTL_VERSION: "v1.36.2" },
    }, {
      name: "Install kustomize (NetworkPolicy static render proof)",
      env: {
        KUSTOMIZE_VERSION: "v5.8.1",
        KUSTOMIZE_SHA256: "029a7f0f4e1932c52a0476cf02a0fd855c0bb85694b82c338fc648dcb53a819d",
      },
    }, {
      name: "Render manifests and NetworkPolicy enforcement preflight",
      env: { CONSOLE_NETWORKPOLICY_PREFLIGHT: "warn" },
    }],
  },
};

function jobBlock(workflow, job) {
  const jobs = workflow.slice(workflow.indexOf("jobs:\n") + "jobs:\n".length);
  const match = jobs.match(new RegExp(`^  ${job}:\\n([\\s\\S]*?)(?=^  [A-Za-z0-9_-]+:|(?![\\s\\S]))`, "m"));
  return match?.[1] ?? null;
}

function needsPreflight(block) {
  const value = block.match(/^    needs:\s*(.+)$/m)?.[1]?.trim();
  if (!value) return false;
  if (value.startsWith("[") && value.endsWith("]")) {
    return value.slice(1, -1).split(",").map((job) => job.trim()).includes("preflight");
  }
  return value === "preflight";
}

function stepBlocks(block) {
  const steps = block.match(/^    steps:\n([\s\S]*)$/m)?.[1] ?? "";
  return steps.split(/^      - /m).slice(1);
}

function runScalar(step) {
  return step.match(/^        run: ([^\n]+)$/m)?.[1]?.trim();
}

function isUnconditional(step) {
  return !/^        (?:if|continue-on-error):/m.test(step);
}

function hasJobDefaultShell(block) {
  const defaults = block.match(/^    defaults:\n((?:      [^\n]*(?:\n|$))*)/m)?.[1] ?? "";
  return /^        shell:/m.test(defaults);
}

function hasUnsafeStepShell(block) {
  return [...block.matchAll(/^        shell: ([^\n]+)$/gm)]
    .some(([, shell]) => shell.trim() !== "bash");
}

function requireProtectedExecutionMetadata(workflowModel, failures) {
  if (Object.hasOwn(workflowModel, "env") || Object.hasOwn(workflowModel, "defaults")) {
    failures.push("CI workflow must not define workflow-level env or defaults");
  }

  for (const [jobName, expected] of Object.entries(protectedJobExecutionMetadata)) {
    const job = workflowModel.jobs?.[jobName];
    if (!job || typeof job !== "object") continue;

    const expectedEnv = Object.hasOwn(expected, "env") ? expected.env : undefined;
    const expectedDefaults = Object.hasOwn(expected, "defaults") ? expected.defaults : undefined;
    if (!isDeepStrictEqual(job.env, expectedEnv)
      || !isDeepStrictEqual(job.defaults, expectedDefaults)) {
      failures.push(`${jobName} must preserve its exact job env/defaults execution metadata`);
    }

    const actualStepEnv = (Array.isArray(job.steps) ? job.steps : [])
      .filter((step) => step && typeof step === "object" && Object.hasOwn(step, "env"))
      .map((step) => ({ name: step.name, env: step.env }));
    if (!isDeepStrictEqual(actualStepEnv, expected.stepEnv ?? [])) {
      failures.push(`${jobName} must preserve its exact step environment allowlist`);
    }

    const unsafeShell = (Array.isArray(job.steps) ? job.steps : [])
      .some((step) => step
        && typeof step === "object"
        && Object.hasOwn(step, "shell")
        && step.shell !== "bash");
    if (unsafeShell) {
      failures.push(`${jobName} may use only the default shell or canonical shell: bash`);
    }
  }
}

function multilineRunCommands(step) {
  const run = step.match(/^        run: \|\n((?:          [^\n]*(?:\n|$))*)/m)?.[1] ?? "";
  return run
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function runScript(step) {
  const scalar = runScalar(step);
  if (scalar && scalar !== "|") return scalar;
  const multiline = step.match(/^        run: \|\n((?:          [^\n]*(?:\n|$))*)/m)?.[1];
  return multiline?.replace(/^          /gm, "") ?? "";
}

function stripShellComment(line) {
  let quote = null;
  let escaped = false;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (escaped) {
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (quote) {
      if (character === quote) quote = null;
    } else if (character === "'" || character === '"') {
      quote = character;
    } else if (character === "#" && (index === 0 || /\s/.test(line[index - 1]))) {
      return line.slice(0, index);
    }
  }
  return line;
}

function shellCommandTokens(script) {
  const commands = [];
  let command = "";
  for (const physicalLine of script.split(/\r?\n/)) {
    const line = stripShellComment(physicalLine.trim());
    if (!line) continue;
    const continued = /\\\s*$/.test(line);
    command += (command ? " " : "") + (continued ? line.replace(/\\\s*$/, "") : line);
    if (!continued) {
      commands.push(command);
      command = "";
    }
  }
  if (command) commands.push(command + "\\");

  return commands.map((surface) => {
    const tokens = [];
    let token = "";
    let quote = null;
    let escaped = false;
    const flush = () => {
      if (token) tokens.push(token);
      token = "";
    };
    for (let index = 0; index < surface.length; index += 1) {
      const character = surface[index];
      if (escaped) {
        token += character;
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (quote) {
        if (character === quote) quote = null;
        else token += character;
      } else if (character === "'" || character === '"') {
        quote = character;
      } else if (/\s/.test(character)) {
        flush();
      } else if (";|&".includes(character)) {
        flush();
        tokens.push(character);
      } else {
        token += character;
      }
    }
    const malformed = quote !== null || escaped;
    if (escaped) token += "\\";
    flush();
    return { tokens, malformed };
  });
}

const maxExecutablePrefixDepth = 12;
const maxExecutableTokens = 512;
const assignmentToken = /^[A-Za-z_][A-Za-z0-9_]*=/;

function mergeCargoAnalysis(left, right) {
  for (const packageName of right.packages) left.packages.add(packageName);
  left.malformed ||= right.malformed;
  return left;
}

function emptyCargoAnalysis(malformed = false) {
  return { packages: new Set(), malformed };
}

function packageArguments(tokens, index) {
  const packages = new Set();
  for (; index < tokens.length; index += 1) {
    const argument = tokens[index];
    let packageName = null;
    if (argument === "-p" || argument === "--package") {
      packageName = tokens[index + 1];
      index += 1;
    } else if (argument.startsWith("-p=")) {
      packageName = argument.slice(3);
    } else if (argument.startsWith("--package=")) {
      packageName = argument.slice("--package=".length);
    }
    if (!packageName) continue;
    packages.add(packageName);
  }
  return { packages, malformed: false };
}

function cargoTestAnalysis(tokens, depth = 0) {
  if (depth > maxExecutablePrefixDepth || tokens.length > maxExecutableTokens) {
    return emptyCargoAnalysis(true);
  }
  let index = 0;
  while (assignmentToken.test(tokens[index] ?? "")) index += 1;
  if (index === tokens.length) return emptyCargoAnalysis();

  if (tokens[index] === "command") {
    index += 1;
    while (tokens[index]?.startsWith("-")) {
      const option = tokens[index];
      if (option === "--") {
        index += 1;
        break;
      }
      if (option === "-v" || option === "-V") return emptyCargoAnalysis();
      if (option === "-p") {
        index += 1;
        continue;
      }
      return emptyCargoAnalysis(true);
    }
    return index < tokens.length
      ? cargoTestAnalysis(tokens.slice(index), depth + 1)
      : emptyCargoAnalysis(true);
  }

  if (tokens[index] === "env") {
    index += 1;
    while (tokens[index]) {
      const option = tokens[index];
      if (option === "--") {
        index += 1;
        break;
      }
      if (option === "-S" || option === "--split-string") {
        const payload = tokens[index + 1];
        if (!payload || index + 2 !== tokens.length) return emptyCargoAnalysis(true);
        const payloads = shellCommandTokens(payload);
        if (payloads.length === 0) return emptyCargoAnalysis(true);
        return payloads.reduce(
          (analysis, surface) => mergeCargoAnalysis(
            analysis,
            surface.malformed
              ? emptyCargoAnalysis(true)
              : cargoTestAnalysis(surface.tokens, depth + 1),
          ),
          emptyCargoAnalysis(),
        );
      }
      if (["-u", "--unset", "-C", "--chdir"].includes(option)) {
        if (!tokens[index + 1]) return emptyCargoAnalysis(true);
        index += 2;
      } else if (["-i", "--ignore-environment", "-v", "--debug"].includes(option)) {
        index += 1;
      } else if (assignmentToken.test(option)) {
        index += 1;
      } else if (option.startsWith("-")) {
        return emptyCargoAnalysis(true);
      } else {
        break;
      }
    }
    return index < tokens.length
      ? cargoTestAnalysis(tokens.slice(index), depth + 1)
      : emptyCargoAnalysis(true);
  }

  if (tokens[index] !== "cargo") return emptyCargoAnalysis();
  index += 1;
  if (tokens[index] === "test") return packageArguments(tokens, index + 1);
  if (tokens[index] === "nextest" && tokens[index + 1] === "run") {
    return packageArguments(tokens, index + 2);
  }
  return emptyCargoAnalysis();
}

function cargoTestAnalysisInStep(step) {
  const analysis = emptyCargoAnalysis();
  for (const surface of shellCommandTokens(runScript(step))) {
    if (surface.malformed) {
      mergeCargoAnalysis(analysis, emptyCargoAnalysis(true));
      continue;
    }
    let segment = [];
    for (const token of [...surface.tokens, ";"]) {
      if (token === ";" || token === "|" || token === "&") {
        mergeCargoAnalysis(analysis, cargoTestAnalysis(segment));
        segment = [];
      } else {
        segment.push(token);
      }
    }
  }
  return analysis;
}

function requireUnconditionalRun(steps, command, job, failures) {
  const matchingSteps = steps.filter((step) => runScalar(step) === command);
  if (matchingSteps.length === 0) {
    failures.push(`${job} must run ${command}`);
  } else if (matchingSteps.some((step) => !isUnconditional(step))) {
    failures.push(`${job} must run ${command} unconditionally without if or continue-on-error`);
  }
}

function requireReasoningLensContracts(steps, failures) {
  const regressionByName = steps.filter((step) => stepName(step) === reasoningLensRegressionName);
  const regressionByCommand = steps.filter((step) => runScalar(step) === reasoningLensTestCommand);
  if (
    regressionByName.length !== 1
    || regressionByCommand.length !== 1
    || regressionByName[0] !== regressionByCommand[0]
    || !hasOnlyExpectedCondition(regressionByName[0] ?? "", null)
  ) {
    failures.push("preflight must run the exact reasoning-lens regression once and unconditionally");
  }

  const admissions = steps.filter((step) => stepName(step) === reasoningLensAdmissionName);
  const admission = admissions[0] ?? "";
  const shells = [...admission.matchAll(/^        shell: ([^\n]+)$/gm)].map((match) => match[1]);
  const environmentBlock = admission.match(/^        env:\n((?:          [^\n]+\n)+)/m)?.[1] ?? "";
  const environment = environmentBlock
    .split(/\r?\n/)
    .filter(Boolean)
    .map((line) => line.trim());
  const script = runScript(admission).replace(/\n$/, "");
  if (
    admissions.length !== 1
    || JSON.stringify(shells) !== JSON.stringify(["bash"])
    || JSON.stringify(environment) !== JSON.stringify(reasoningLensAdmissionEnvironment)
    || script !== reasoningLensAdmissionScript
    || !hasOnlyExpectedCondition(admission, null)
  ) {
    failures.push("preflight must preserve the exact reasoning-lens event admission contract");
  }
}

function requireConsoleExactMergeProof(workflow, steps, failures) {
  const derive = steps.filter((step) => stepName(step) === "Derive exact console C/T/M train");
  if (derive.length !== 1 || !hasOnlyExpectedCondition(derive[0], consolePrCondition) || multilineRunCommands(derive[0]).join("\n") !== consoleTrainDerivation.join("\n")) {
    failures.push("preflight must derive exact C/T/M from the pull-request synthetic merge");
  }
  for (const command of [consoleTruthLedgerCommand, consoleFanoutPlannerAdmissionCommand]) {
    const matching = steps.filter((step) => runScalar(step) === command);
    if (matching.length !== 1 || !hasOnlyExpectedCondition(matching[0], consolePrCondition)) failures.push(`preflight must run ${command} only after exact C/T/M derivation on pull requests`);
  }
  for (const command of [consoleAuthorityTrainTestCommand, consoleBootstrapTestCommand, consoleTruthLedgerTestCommand, consoleFanoutPlannerTestCommand]) {
    const matching = steps.filter((step) => runScalar(step) === command);
    if (matching.length !== 1 || !isUnconditional(matching[0])) failures.push(`preflight must run ${command} unconditionally on pull requests and main`);
  }
  if (workflow.includes("CONSOLE_INTEGRATION_TIP_SHA")) failures.push("preflight must not reference legacy CONSOLE_INTEGRATION_TIP_SHA");
}

function requirePostgresWrapperContracts(buildFile, failures) {
  for (const [name, binary] of postgresWrapperContracts) {
    const block = buildFile.match(new RegExp(`sh_test\\(\\n    name = "${name}",[\\s\\S]*?\\n\\)`, "m"))?.[0];
    if (!block) {
      failures.push(`tools/buck/BUCK must define PostgreSQL wrapper ${name}`);
      continue;
    }
    const expectedArgs = `args = ["$(location ${binary})"],`;
    const expectedDeps = `deps = ["${binary}"],`;
    const hasExactAttribute = (attribute) => (
      [...block.matchAll(new RegExp(`^    ${attribute.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`, "gm"))].length === 1
    );
    const hasExactlyOneField = (field) => (
      [...block.matchAll(new RegExp(`^    ${field} = .+$`, "gm"))].length === 1
    );
    if (!hasExactAttribute(`test = "${postgresWrapperLoader}",`)
      || !hasExactAttribute(expectedArgs)
      || !hasExactAttribute(expectedDeps)
      || !hasExactAttribute(`labels = ${postgresWrapperLabels},`)
      || !["test", "args", "deps", "labels"].every(hasExactlyOneField)) {
      failures.push(`tools/buck/BUCK must bind PostgreSQL wrapper ${name} to the loader and exact Rust binary`);
    }
  }
}

/**
 * TOTALITY over the crate that decides row visibility: every integration test it
 * has must be wrapped in exactly one PostgreSQL `sh_test` and that wrapper must
 * be named in ci.yml.
 *
 * The lists above are per-target contracts — they check that the entries someone
 * REMEMBERED to add are wired correctly, and say nothing about the ones nobody
 * added. That is the four-coupled-places defect this repo has shipped seven
 * times, most recently to `action_execute`, `ont_gaps` and `projected_dispatch`,
 * all three of which had a `rust_test` target and appeared in no wrapper, no
 * ci.yml step and neither locked list. Two of them were broken for the whole
 * lane that introduced the break, and CI could not see it.
 *
 * Deliberately scoped to this ONE crate and driven off its GENERATED BUCK file
 * rather than a repo-wide sweep. The repo-wide version is not blocked by a lack
 * of signal — `TEST_RESOURCE_REQUIREMENTS` in tools/buck/gen_first_party.py
 * already declares `postgres` per test file, and gen_first_party FAILS on any
 * test file that has no declaration. It is blocked by scale: of the 188
 * postgres-declared integration tests in this repo, 163 have no `sh_test`
 * wrapper (measured 2026-07-29). Turning that into a gate today makes CI
 * permanently red, and a gate nobody can satisfy gets deleted. Widening this is
 * a per-crate decision with the same shape as this one, not a cleverer regex.
 */
function requireOntologyRestItestReachability(buildFile, workflow, failures) {
  const itests = [
    ...ontologyRestCrateBuildFile.matchAll(/^    name = "(console-ontology-rest-itest-[a-z0-9_]+)",$/gm),
  ].map(([, name]) => name);
  if (itests.length === 0) {
    failures.push("backend/crates/ontology/rest/BUCK must declare integration-test targets");
    return;
  }
  const wrappers = [...buildFile.matchAll(/sh_test\(\n([\s\S]*?)\n\)/g)].map(([, body]) => ({
    name: body.match(/^    name = "([^"]+)",$/m)?.[1],
    target: body.match(/^    args = \["\$\(location ([^)]+)\)"\],$/m)?.[1],
  }));
  for (const itest of itests) {
    const target = `//backend/crates/ontology/rest:${itest}`;
    const matching = wrappers.filter((wrapper) => wrapper.target === target);
    if (matching.length !== 1 || !matching[0].name) {
      failures.push(`tools/buck/BUCK must wrap ${itest} in exactly one PostgreSQL sh_test`);
      continue;
    }
    if (!new RegExp(`^\\s+//tools/buck:${matching[0].name}( \\\\)?$`, "m").test(workflow)) {
      failures.push(`ci.yml must execute //tools/buck:${matching[0].name} or ${itest} runs nowhere`);
    }
  }
}

function requireUnconditionalMultilineRun(steps, commands, job, failures) {
  const matchingSteps = steps.filter((step) => multilineRunCommands(step).join("\n") === commands.join("\n"));
  if (matchingSteps.length === 0) {
    failures.push(`${job} must run the locked PostgreSQL reachability targets`);
  } else if (matchingSteps.some((step) => !isUnconditional(step))) {
    failures.push(`${job} must run the locked PostgreSQL reachability targets unconditionally without if or continue-on-error`);
  }
}

function runCommand(step) {
  const scalar = runScalar(step);
  return scalar && scalar !== "|" ? scalar : (multilineRunCommands(step).join("\n") || null);
}

function requireOnlyLockedRuns(steps, commands, job, failures) {
  const actual = steps.map(runCommand).filter(Boolean);
  if (actual.length !== commands.length || actual.some((command, index) => command !== commands[index])) {
    failures.push(`${job} must contain only the locked ordered run steps`);
  }
}

function stepName(step) {
  return step.match(/^name: ([^\n]+)$/m)?.[1] ?? null;
}

function stepWorkingDirectory(step) {
  return step.match(/^        working-directory: ([^\n]+)$/m)?.[1]?.trim() ?? null;
}

function hasOnlyExpectedCondition(step, expectedIf) {
  const conditions = [...step.matchAll(/^        if: ([^\n]+)$/gm)].map((match) => match[1]);
  return (expectedIf === null
    ? conditions.length === 0
    : conditions.length === 1 && conditions[0] === expectedIf)
    && !/^        continue-on-error:/m.test(step);
}

function requireOrderedStepContracts(steps, contracts, job, failures) {
  const indexes = [];
  for (const contract of contracts) {
    const matches = steps
      .map((step, index) => ({ step, index }))
      .filter(({ step }) => stepName(step) === contract.name);
    if (matches.length !== 1
      || (contract.run !== undefined && runCommand(matches[0]?.step) !== contract.run)
      || (contract.workingDirectory !== undefined
        && stepWorkingDirectory(matches[0]?.step ?? "") !== contract.workingDirectory)
      || !hasOnlyExpectedCondition(matches[0]?.step ?? "", contract.if)) {
      failures.push(`${job} must preserve the locked fail-fast step multiset and failure semantics`);
      indexes.push(-1);
    } else {
      indexes.push(matches[0].index);
    }
  }
  if (indexes.some((index) => index < 0) || indexes.some((index, position) => position > 0 && index <= indexes[position - 1])) {
    failures.push(`${job} must preserve the locked fail-fast step order`);
  }
  return indexes;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function stableSerialize(value) {
  if (Array.isArray(value)) return `[${value.map(stableSerialize).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => (
      `${JSON.stringify(key)}:${stableSerialize(value[key])}`
    )).join(",")}}`;
  }
  return JSON.stringify(value);
}

function hasExactOptionalProperty(value, property, expected) {
  return expected === null
    ? !Object.hasOwn(value, property)
    : Object.hasOwn(value, property) && value[property] === expected;
}

function requireExactRequiredJobContracts(workflowModel, failures) {
  const workflowEnvelope = Object.fromEntries(
    Object.entries(workflowModel).filter(([property]) => property !== "jobs"),
  );
  if (sha256(stableSerialize(workflowEnvelope)) !== workflowExecutionEnvelopeSha256) {
    failures.push("CI workflow must preserve its exact trigger, permission, and concurrency execution envelope");
  }
  const actualJobIds = Object.keys(workflowModel.jobs ?? {}).sort();
  if (!isDeepStrictEqual(actualJobIds, exactCiJobIds)) {
    failures.push("CI workflow must preserve its exact job-id set");
  }
  if (!isDeepStrictEqual(workflowModel.jobs?.["required-ci"], requiredCiAggregator)) {
    failures.push("Required / CI must preserve its exact ten-job success aggregation contract");
  }

  for (const [jobName, contracts] of Object.entries(requiredJobRunContracts)) {
    const job = workflowModel.jobs?.[jobName];
    if (!job || !Array.isArray(job.steps)) {
      failures.push(`${jobName} must define its complete ordered setup/proof/cleanup run contract`);
      continue;
    }

    const actionContracts = requiredJobActionContracts[jobName] ?? [];
    const jobMetadata = Object.fromEntries(
      Object.entries(job).filter(([property]) => property !== "steps"),
    );
    if (sha256(stableSerialize(jobMetadata)) !== requiredJobMetadataSha256[jobName]) {
      failures.push(`${jobName} must preserve its exact job execution envelope`);
    }
    if (job.steps.length !== contracts.length + actionContracts.length) {
      failures.push(
        `${jobName} must contain only its locked ordered action, setup, proof, and cleanup steps`,
      );
    }

    const actionSteps = job.steps.filter((step) => step && typeof step.uses === "string");
    if (actionSteps.length !== actionContracts.length) {
      failures.push(
        `${jobName} must preserve all ${actionContracts.length} ordered setup action steps; found ${actionSteps.length}`,
      );
    }
    for (const { index, step: expectedStep } of actionContracts) {
      if (!isDeepStrictEqual(job.steps[index], expectedStep)) {
        failures.push(
          `${jobName} setup action step ${index + 1} must preserve its exact name, identity, inputs, position, and execution semantics`,
        );
      }
    }

    const runSteps = job.steps.filter((step) => step && typeof step.run === "string");
    if (runSteps.length !== contracts.length) {
      failures.push(
        `${jobName} must preserve all ${contracts.length} ordered setup/proof run steps; found ${runSteps.length}`,
      );
    }

    for (let index = 0; index < Math.max(runSteps.length, contracts.length); index += 1) {
      const step = runSteps[index];
      const contract = contracts[index];
      if (!step || !contract) continue;

      const exactRun = Object.hasOwn(contract, "run")
        ? step.run === contract.run
        : sha256(step.run) === contract.runSha256;
      const exactExecutionMetadata = hasExactOptionalProperty(step, "if", contract.if)
        && hasExactOptionalProperty(step, "working-directory", contract.workingDirectory)
        && hasExactOptionalProperty(step, "shell", contract.shell)
        && !Object.hasOwn(step, "continue-on-error")
        && !Object.hasOwn(step, "timeout-minutes")
        && !Object.hasOwn(step, "uses")
        && !Object.hasOwn(step, "with");
      if (step.name !== contract.name || !exactRun || !exactExecutionMetadata) {
        failures.push(
          `${jobName} ${contract.kind} run step ${index + 1} must preserve its exact name, command, condition, and execution semantics`,
        );
      }
    }
  }
}

// The Swatinem/rust-cache step was REMOVED from this list on 2026-07-31 because Buck2
// never writes backend/target, so the job restored and saved a cache it could not use.
// The Buck2 app build itself was removed later: the only thing it fed was an app-boot
// comparison of the served /openapi/openapi.yaml against the file on disk, which is
// tautological because backend/app/src/lib.rs:214 include_str!s that exact file. What
// remains in this job is text-only, so it needs neither Rust nor a built binary.
const apiContractAllowedSteps = [
  "name: Checkout\n        uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7",
  "name: Set up Node.js\n        uses: actions/setup-node@48b55a011bda9f5d6aeb4c2d9c7362e8dae4041e # v6.4.0\n        with:\n          node-version: \"24\"\n          cache: npm",
  "name: Install Node tooling\n        run: npm ci",
  "name: Platform contract drift gate\n        if: ${{ !cancelled() }}\n        run: npm run check:platform-contract-drift",
  "name: Employee import replay contract\n        if: ${{ !cancelled() }}\n        run: npm run test:employee-import-contract",
  "name: Ontology write precondition contract\n        if: ${{ !cancelled() }}\n        run: npm run test:ontology-write-precondition",
];
function hasOnlyAllowedApiContractSteps(steps) {
  return steps.length === apiContractAllowedSteps.length
    && steps.every((step, index) => step.trimEnd() === apiContractAllowedSteps[index]);
}

function requireReindeerToolchainBefore(steps, command, failures) {
  const commandIndex = steps.findIndex((step) => runScalar(step) === command);
  const toolchainIndex = steps.findIndex((step) => multilineRunCommands(step).includes(reindeerToolchainInstall));
  if (toolchainIndex < 0) {
    failures.push("generated-face-authority must install the lock-pinned Reindeer Rust toolchain before full generated-face closure");
    return;
  }
  const commands = multilineRunCommands(steps[toolchainIndex]);
  const strictModeIndex = commands.indexOf(strictShellMode);
  const sourceIndex = commands.indexOf(reindeerToolchainSource);
  const installIndex = commands.indexOf(reindeerToolchainInstall);
  if (sourceIndex < 0) {
    failures.push(`generated-face-authority must source ${reindeerToolchainLock} before installing the Reindeer Rust toolchain`);
  } else if (strictModeIndex < 0 || strictModeIndex > sourceIndex) {
    failures.push(`generated-face-authority must enable strict shell mode before sourcing ${reindeerToolchainLock}`);
  } else if (sourceIndex > installIndex) {
    failures.push(`generated-face-authority must source ${reindeerToolchainLock} before installing the Reindeer Rust toolchain`);
  }
  if (commands.some((entry) => reindeerToolchainOverride.test(entry))) {
    failures.push(`generated-face-authority must not override REINDEER_TOOLCHAIN after sourcing ${reindeerToolchainLock}`);
  }
  if (!isUnconditional(steps[toolchainIndex])) {
    failures.push("generated-face-authority must install the Reindeer Rust toolchain unconditionally");
  }
  if (commandIndex >= 0 && toolchainIndex > commandIndex) {
    failures.push("generated-face-authority must install the lock-pinned Reindeer Rust toolchain before full generated-face closure");
  }
}

function requirePreflightRustToolchainBefore(steps, failures) {
  const setupIndex = steps.findIndex((step) =>
    step.startsWith("name: Install Rust toolchain for Cargo.lock consistency\n"),
  );
  if (setupIndex < 0) {
    failures.push("preflight must install the pinned Rust toolchain before Cargo-dependent CI preflight tests");
    return;
  }
  if (!isUnconditional(steps[setupIndex])) {
    failures.push("preflight must install the pinned Rust toolchain unconditionally");
    return;
  }
  for (const command of [
    ciPreflightTestCommand,
    "cargo metadata --manifest-path backend/Cargo.toml --locked --format-version=1 >/dev/null",
  ]) {
    const commandIndex = steps.findIndex((step) => runScalar(step) === command);
    if (commandIndex >= 0 && setupIndex > commandIndex) {
      failures.push(`preflight must install the pinned Rust toolchain before ${command}`);
    }
  }
}

function requireDotSlashBefore(steps, command, job, failures) {
  const commandIndex = steps.findIndex((step) => runScalar(step) === command);
  const dotSlashIndex = steps.findIndex((step) => runScalar(step) === dotSlashBootstrap);
  if (dotSlashIndex < 0) {
    failures.push(`${job} must install pinned DotSlash before Buck2`);
  } else if (!isUnconditional(steps[dotSlashIndex])) {
    failures.push(`${job} must install DotSlash unconditionally`);
  } else if (commandIndex >= 0 && dotSlashIndex > commandIndex) {
    failures.push(`${job} must install DotSlash before ${command}`);
  }
}

function requireEffectiveDotSlashBootstrap(block, job, failures) {
  const workingDirectory = block.match(/^    defaults:\n      run:\n        working-directory: ([^\n]+)$/m)?.[1]?.trim();
  const bootstrap = workingDirectory === "backend" ? `../${dotSlashBootstrap}` : dotSlashBootstrap;
  const steps = stepBlocks(block);
  const bootstrapIndex = steps.findIndex((step) => runScalar(step) === bootstrap);
  if (bootstrapIndex < 0) {
    failures.push(`${job} must install pinned DotSlash from ${bootstrap}`);
    return;
  }
  const firstBuckInvocation = steps.findIndex((step, index) => {
    const run = runScalar(step);
    const command = run === "|" ? multilineRunCommands(step).join("\n") : run ?? "";
    return index !== bootstrapIndex
      && (
        /(?:^|[^A-Za-z0-9_])tools\/buck(?:2|\/)/.test(command)
        || /\bdotslash\b/i.test(command)
        || /(?:^|[\s;&|])node\s+scripts\/dev-up\.mjs\s+bootstrap(?:\s|$)/m.test(command)
      );
  });
  if (firstBuckInvocation >= 0 && bootstrapIndex > firstBuckInvocation) {
    failures.push(`${job} must install pinned DotSlash before its first Buck invocation`);
  }
}

export function evaluateCiPreflight(
  workflow,
  buckBuildFile = postgresWrapperBuildFile,
  freeRunnerDiskAction = freeRunnerDiskActionFile,
) {
  const failures = [];
  let workflowModel;
  try {
    workflowModel = yaml.load(workflow);
  } catch (error) {
    return { failures: [`CI workflow must parse as YAML: ${error.message}`] };
  }
  if (!workflowModel || typeof workflowModel !== "object") {
    return { failures: ["CI workflow must parse as a YAML mapping"] };
  }
  try {
    const actionModel = yaml.load(freeRunnerDiskAction);
    if (sha256(stableSerialize(actionModel)) !== freeRunnerDiskActionSha256) {
      failures.push("free-runner-disk must preserve its exact composite action execution contract");
    }
  } catch (error) {
    failures.push(`free-runner-disk action must parse as YAML: ${error.message}`);
  }
  requireProtectedExecutionMetadata(workflowModel, failures);
  requireExactRequiredJobContracts(workflowModel, failures);

  for (const trigger of ["push", "pull_request"]) {
    const triggerContract = workflowModel.on?.[trigger];
    if (triggerContract && typeof triggerContract === "object"
      && ["paths", "paths-ignore"].some((filter) => Object.hasOwn(triggerContract, filter))) {
      failures.push(`${trigger} must create required CI contexts for every change without path filters`);
    }
  }

  const preflight = jobBlock(workflow, "preflight");
  if (!preflight) {
    failures.push("CI must define a preflight job before expensive jobs");
    return { failures };
  }

  const preflightSteps = stepBlocks(preflight);
  if (/^    if:/m.test(preflight)) {
    failures.push("preflight must not define job-level if");
  }
  if (/^    continue-on-error:/m.test(preflight)) {
    failures.push("preflight must not define job-level continue-on-error");
  }
  if (hasJobDefaultShell(preflight)) {
    failures.push("preflight must not override defaults.run.shell");
  }
  if (hasUnsafeStepShell(preflight)) {
    failures.push("preflight may use only the default shell or canonical shell: bash");
  }
  const checkout = preflightSteps.find((step) => step.startsWith("name: Checkout\n"));
  if (!checkout || !/^        with:\n(?:          [^\n]+\n)*          fetch-depth: 0$/m.test(checkout)) {
    failures.push("preflight checkout must fetch full history with fetch-depth: 0");
  }
  requireDotSlashBefore(preflightSteps, "tools/buck/preflight.sh", "preflight", failures);
  requirePreflightRustToolchainBefore(preflightSteps, failures);
  for (const command of requiredPreflightCommands) {
    requireUnconditionalRun(preflightSteps, command, "preflight", failures);
  }
  requireReasoningLensContracts(preflightSteps, failures);
  requireConsoleExactMergeProof(workflow, preflightSteps, failures);
  // One job, both crates. They share console-kernel-core, so two jobs recompiled
  // the same dependencies and paid two runner startups and two cache restores.
  const domainUnit = jobBlock(workflow, "domain-unit");
  if (domainUnit) {
    const steps = stepBlocks(domainUnit);
    const domainSteps = steps.filter((step) => stepName(step) === "Domain crate unit tests");
    const domainStep = domainSteps[0] ?? "";
    const parsedDomainCommands = shellCommandTokens(runScript(domainStep));
    const domainCommandsMatch = domainSteps.length === 1
      && parsedDomainCommands.length === domainUnitExpectedCommands.length
      && parsedDomainCommands.every((command, index) => (
        !command.malformed
        && JSON.stringify(command.tokens) === JSON.stringify(domainUnitExpectedCommands[index])
      ));
    if (!domainCommandsMatch || !isUnconditional(domainStep)) {
      failures.push("domain-unit must execute the locked Cargo test commands directly and unconditionally");
    }
    if (/^    (?:env|defaults):/m.test(domainUnit)
      || /^        (?:env|shell):/m.test(domainUnit)) {
      failures.push("domain-unit must use the default shell with no job or step env/defaults overrides");
    }
    const directTokens = parsedDomainCommands.flatMap((command) => command.tokens);
    for (const pkg of domainUnitPackages) {
      const present = directTokens.some((token, index) => token === "-p" && directTokens[index + 1] === pkg);
      if (!present) failures.push(`domain-unit must run -p ${pkg}`);
    }
    for (const t of domainUnitTestFiles) {
      const present = directTokens.some((token, index) => token === "--test" && directTokens[index + 1] === t);
      if (!present) failures.push(`domain-unit must run --test ${t}`);
    }
    if (parsedDomainCommands[0]?.tokens?.includes("--lib") !== true) {
      failures.push("domain-unit must pass --lib on its first cargo invocation");
    }

    const runStepNames = steps.filter((step) => runCommand(step) !== null).map(stepName);
    if (JSON.stringify(runStepNames) !== JSON.stringify(["Domain crate unit tests"])) {
      failures.push(`domain-unit must contain only the locked ordered run steps; found ${runStepNames.length} run steps`);
    }
  }

  requirePostgresWrapperContracts(buckBuildFile, failures);
  requireOntologyRestItestReachability(buckBuildFile, workflow, failures);

  const postgresDomainReachability = jobBlock(workflow, "postgres-domain-reachability");
  if (postgresDomainReachability) {
    const steps = stepBlocks(postgresDomainReachability);
    requireUnconditionalMultilineRun(
      steps,
      postgresDomainReachabilityCommands,
      "postgres-domain-reachability",
      failures,
    );
    requireOnlyLockedRuns(
      steps,
      [dotSlashBootstrap, postgresDomainReachabilityCommands.join("\n")],
      "postgres-domain-reachability",
      failures,
    );
  }

  const companyConformance = jobBlock(workflow, "company-conformance");
  if (companyConformance) {
    const steps = stepBlocks(companyConformance);
    requireOrderedStepContracts(
      steps,
      [{
        name: "Install pinned DotSlash runtime",
        run: dotSlashBootstrap,
        if: null,
      }, {
        name: "Company conformance against disposable PostgreSQL",
        run: companyConformanceCommands.join("\n"),
        if: null,
      }],
      "company-conformance",
      failures,
    );
    requireOnlyLockedRuns(
      steps,
      [dotSlashBootstrap, companyConformanceCommands.join("\n")],
      "company-conformance",
      failures,
    );
  }

  const backend = jobBlock(workflow, "backend");
  if (backend) {
    const steps = stepBlocks(backend);
    const failFastIf = null;
    const pr473ContractTestCommand = "python3 scripts/check-pr473-migration-operational.test.py -v";
    if (steps.some((step) => step.includes("if: ${{ !cancelled() }}"))) {
      failures.push("backend must not use !cancelled() on protected fail-fast steps");
    }
    const sourceGateContracts = [
      ["Layer-boundary gate", "cargo run -p console-gate-layer-boundary"],
      ["Audit-coverage gate", "cargo run -p console-gate-audit-coverage"],
      ["Migration-safety gate", "cargo run -p console-gate-migration-safety"],
      ["Tenant-isolation gate", "cargo run -p console-gate-tenant-isolation"],
      ["PII-no-logs gate", "cargo run -p console-gate-pii-no-logs"],
      ["RLS-arming gate", "cargo run -p console-gate-rls-arming"],
      ["Dev-auth-absence gate", "cargo run -p console-gate-dev-auth-absence"],
      ["IaC tier-discipline gate", "cargo run -p console-gate-iac-tier"],
      ["Fabricated-branch gate", "cargo run -p console-gate-fabricated-branch"],
    ];
    const gateIndexes = requireOrderedStepContracts(
      steps,
      [
        ["clippy -D warnings", "SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings"],
        ...sourceGateContracts,
        ["PR 473 migration operational contract tests", pr473ContractTestCommand],
        ["Reconcile portable PostgreSQL role topology", undefined],
        ["PR 473 migration operational gate", "npm run check:pr473-migration-operational"],
        ["Boot smoke — migrate + serve + /readyz", undefined],
      ].map(([name, run]) => ({ name, run, if: failFastIf })),
      "backend",
      failures,
    );
    if (gateIndexes[1] !== gateIndexes[0] + 1) {
      failures.push("backend must run source-only gates immediately after clippy");
    }
    requireOrderedStepContracts(
      steps,
      [
        {
          name: "Buck2 dev-auth feature PostgreSQL suites",
          run: [
            "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
            "//tools/buck:auth-rest-dev-auth-inline-postgres \\",
            "//tools/buck:auth-rest-dev-auth-session-postgres \\",
            "//tools/buck:auth-rest-dev-auth-group-admin-postgres \\",
            "//tools/buck:provisioning-dev-principal-upsert-race-postgres",
          ].join("\n"),
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          // Locked so the step cannot be deleted silently: the crate's residual
          // lowering decides row visibility, and its unit target executed in no
          // workflow at all until this contract existed.
          name: "Buck2 platform-authz unit suite",
          run: "env -u DATABASE_URL tools/buck2 test //backend/crates/platform/authz:console-platform-authz-unit",
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          name: "Buck2 console-app unit suite",
          run: "env -u DATABASE_URL tools/buck2 test //backend/app:console-app-unit",
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          // The suite H-1 is *about*. `openapi_drift` is the only thing that inventories every
          // mounted route against openapi.yaml, and it was unprotected: deleting this `run:` line
          // left check:ci-preflight, check:foundation-gates and check:doc-citations all exiting 0.
          // check:request-body-contract closed H-1's request-body half but reads no route
          // inventory, so nothing else in CI covers what this step covers — a gate one line from
          // silent removal is the meta-finding this file exists to refuse.
          name: "Buck2 console-app OpenAPI drift suite",
          run: "env -u DATABASE_URL tools/buck2 test //backend/app:console-app-itest-openapi_drift",
          workingDirectory: ".",
          if: failFastIf,
        },
        {
          name: "Buck2 console-app inline PostgreSQL suites",
          run: [
            "tools/buck/test_needs_postgres.sh --num-threads=1 \\",
            "//tools/buck:app-inline-postgres \\",
            "//tools/buck:app-dev-auth-persona-guard-postgres",
          ].join("\n"),
          workingDirectory: ".",
          if: failFastIf,
        },
      ],
      "backend",
      failures,
    );
    const directCargoTestAnalysis = steps.reduce(
      (analysis, step) => mergeCargoAnalysis(analysis, cargoTestAnalysisInStep(step)),
      emptyCargoAnalysis(),
    );
    if (directCargoTestAnalysis.malformed) {
      failures.push("backend must not contain a malformed executable shell surface");
    }
    for (const packageName of ["console-platform-auth-rest", "console-platform-provisioning"]) {
      if (directCargoTestAnalysis.packages.has(packageName)) {
        failures.push("backend must not run direct Cargo PostgreSQL tests for " + packageName);
      }
    }
  }

  const devUpSmoke = jobBlock(workflow, "dev-up-smoke");
  if (devUpSmoke) {
    const devUpSteps = stepBlocks(devUpSmoke);
    const devUpRunContracts = [
      { name: "dev-up compose contract unit test", run: "node --test scripts/dev-up-compose.test.mjs", if: null },
      { name: "Install pinned DotSlash runtime", run: dotSlashBootstrap, if: null },
      { name: "PostgreSQL topology integration regression", run: "ops/postgres-topology.integration.test.sh", if: null },
      { name: "dev-up bootstrap (compose deps + migrate + backend readyz)", run: "node scripts/dev-up.mjs bootstrap", if: null },
      { name: "Confirm /readyz reachable", run: 'curl -fsS "http://127.0.0.1:${CONSOLE_DEV_HTTP_PORT:-8090}/readyz"', if: null },
      { name: "dev-up down", run: "node scripts/dev-up.mjs down", if: "always()" },
    ];
    requireOrderedStepContracts(
      devUpSteps,
      [
        { name: "Checkout", run: null, if: null },
        { name: "dev-up compose contract unit test", run: "node --test scripts/dev-up-compose.test.mjs", if: null },
        { name: "Install pinned DotSlash runtime", run: dotSlashBootstrap, if: null },
        { name: "Free runner disk for Rust backend", run: null, if: null },
        { name: "Install Rust toolchain (pinned via rust-toolchain.toml)", run: null, if: null },
        { name: "Set up Node.js", run: null, if: null },
        ...devUpRunContracts.slice(2),
      ],
      "dev-up-smoke",
      failures,
    );
    const actualRunNames = devUpSteps.filter((step) => runCommand(step) !== null).map(stepName);
    const expectedRunNames = devUpRunContracts.map(({ name }) => name);
    if (JSON.stringify(actualRunNames) !== JSON.stringify(expectedRunNames)) {
      failures.push("dev-up-smoke must contain only the locked ordered proof and cleanup run steps");
    }
  }

  const fullGeneratedFaces = jobBlock(workflow, "generated-face-authority");
  if (fullGeneratedFaces) {
    const fullGeneratedFaceSteps = stepBlocks(fullGeneratedFaces);
    const fullGeneratedFaceCommand = "tools/buck/preflight.sh --full-generated-faces";
    const matchingFullGateSteps = fullGeneratedFaceSteps.filter((step) => runScalar(step) === fullGeneratedFaceCommand);
    if (matchingFullGateSteps.length === 0) {
      failures.push("generated-face-authority must run the complete generated-face closure");
    } else if (matchingFullGateSteps.some((step) => !isUnconditional(step))) {
      failures.push("generated-face-authority must run the complete generated-face closure unconditionally");
    }
    requireDotSlashBefore(
      fullGeneratedFaceSteps,
      fullGeneratedFaceCommand,
      "generated-face-authority",
      failures,
    );
    requireReindeerToolchainBefore(fullGeneratedFaceSteps, fullGeneratedFaceCommand, failures);
  }

  // repo-gates was entirely unlocked: deleting `run: npm run check:adrs` from it returned zero
  // preflight failures. A step wired into ci.yml is not thereby protected — it occupies a slot in
  // the job list and reads as coverage while being one line away from silent removal.
  const repoGates = jobBlock(workflow, "repo-gates");
  if (repoGates) {
    requireOrderedStepContracts(
      stepBlocks(repoGates),
      [{
        name: "Undeclared imports — every bare specifier must be declared",
        run: "npm run check:undeclared-imports",
        if: "${{ !cancelled() }}",
      }, {
        // Wired in 4e7da6b52 and unprotected until now: deleting this step returned zero
        // preflight failures, which is the same one-line-from-silent-removal state the
        // undeclared-imports step above was added to escape.
        name: "Request-body contract — spec fields must exist on the handler",
        run: "npm run check:request-body-contract",
        if: "${{ !cancelled() }}",
      }],
      "repo-gates",
      failures,
    );
  }

  const kubernetesManifests = jobBlock(workflow, "kubernetes-manifests");
  if (kubernetesManifests) {
    requireOrderedStepContracts(
      stepBlocks(kubernetesManifests),
      [{
        name: "Production hardening contract",
        run: "npm run check:production-hardening",
        if: "${{ !cancelled() }}",
      }],
      "kubernetes-manifests",
      failures,
    );
  }

  const apiContract = jobBlock(workflow, "api-contract");
  if (apiContract) {
    const apiContractSteps = stepBlocks(apiContract);
    if (!hasOnlyAllowedApiContractSteps(apiContractSteps)) {
      failures.push("api-contract must contain only the approved ordered steps");
    }
    if (/^    (?:services|env):/m.test(apiContract)) {
      failures.push("api-contract is text-only and must not provision services or job-level environment");
    }
    const driftGateCount = apiContractSteps
      .filter((step) => runScalar(step) === "npm run check:platform-contract-drift").length;
    if (driftGateCount !== 1) {
      failures.push("api-contract must run exactly one npm run check:platform-contract-drift");
    }
    // api-contract is now text-only: every step reads backend/openapi/openapi.yaml and
    // asserts against it. Nothing builds or boots the app, so nothing may hand a binary
    // path (or anything else) to a later step through the environment file.
    if (apiContract.includes("GITHUB_ENV")) {
      failures.push("api-contract must not hand state to later steps through GITHUB_ENV");
    }
    if (apiContract.includes("CONSOLE_APP_BIN")) {
      failures.push("api-contract must not reference CONSOLE_APP_BIN; the job builds no app");
    }
  }

  for (const job of ["backend", "dev-up-smoke"]) {
    const block = jobBlock(workflow, job);
    if (block) requireEffectiveDotSlashBootstrap(block, job, failures);
  }

  // The two jobs that actually run cargo must SHARE one rust-cache entry.
  //
  // MEASURED 2026-07-31: six jobs carried Swatinem/rust-cache, all with
  // `workspaces: backend` and none with a shared-key. rust-cache keys on job name by
  // default, so those were six separate near-duplicate caches of one workspace competing
  // for a 10GB repository LRU budget — they evicted each other. Four of the six never ran
  // cargo at all (Buck2 does not write backend/target) and were removed outright.
  //
  // Without this assertion, deleting `shared-key` silently refragments them: CI stays
  // green, nothing reports it, and the caches quietly stop being shared. Verified that a
  // deletion passed every gate before this block existed.
  for (const job of ["domain-unit", "backend"]) {
    const block = jobBlock(workflow, job);
    if (!block) continue;
    if (!/shared-key:\s*backend-cargo/.test(block)) {
      failures.push(`${job} must share the rust-cache entry via shared-key: backend-cargo`);
    }
    if (!/cache-all-crates:\s*"true"/.test(block)) {
      failures.push(`${job} must set cache-all-crates: "true" on rust-cache`);
    }
  }
  // Exactly ONE writer. Sharing a key without deciding who saves it means whichever job
  // finishes first publishes the cache — so domain-unit's three crates could become the
  // entry `backend` restores for a whole-workspace clippy. `backend` runs
  // clippy --all-targets, a strict superset, so it is the writer and every other cargo
  // job is restore-only.
  {
    const writers = ["domain-unit", "backend"].filter((job) => {
      const block = jobBlock(workflow, job);
      return block && !/save-if:\s*false/.test(block);
    });
    if (writers.length !== 1 || writers[0] !== "backend") {
      failures.push(
        `exactly one cargo job may write the shared rust-cache and it must be backend; found: ${writers.join(", ") || "none"}`,
      );
    }
  }
  // Buck2 never writes backend/target, so a rust-cache step on a Buck2-only job is pure
  // transfer cost and an LRU slot taken from the jobs that do use it.
  for (const job of ["postgres-domain-reachability", "company-conformance", "dev-up-smoke", "api-contract"]) {
    const block = jobBlock(workflow, job);
    if (block && /Swatinem\/rust-cache/.test(block)) {
      failures.push(`${job} runs no cargo and must not carry a rust-cache step`);
    }
  }

  for (const job of protectedJobs) {
    const block = jobBlock(workflow, job);
    if (!block) {
      failures.push(`CI must define protected job ${job}`);
      continue;
    }
    if (!needsPreflight(block)) failures.push(`${job} must need preflight`);
    if (/^    if:/m.test(block)) failures.push(`${job} must not define job-level if`);
    if (/^    continue-on-error:/m.test(block)) failures.push(`${job} must not define job-level continue-on-error`);
    if (hasJobDefaultShell(block)) failures.push(`${job} must not override defaults.run.shell`);
    if (hasUnsafeStepShell(block)) failures.push(`${job} may use only the default shell or canonical shell: bash`);
  }

  return { failures };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const { failures } = evaluateCiPreflight(readFileSync(resolve(root, ".github/workflows/ci.yml"), "utf8"));
  if (failures.length > 0) {
    console.error("CI preflight contract failed:");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exit(1);
  }
  console.log("CI preflight contract passed.");
}
