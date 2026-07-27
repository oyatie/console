import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";
import { evaluateIosUiTestFailClosedChecks } from "./check-ios-ui-test-fail-closed.mjs";

const validLauncher = readFileSync(new URL("./boot-ios-ui-backend.mjs", import.meta.url), "utf8");
const validBoot = '          CONSOLE_IOS_COLDSTART_OTP="$COLDSTART_OTP" "$CONSOLE_IOS_NODE_BIN" "$ROOT/scripts/boot-ios-ui-backend.mjs" "$ROOT" "$AUTH_DIR" "$BP"';
const validWorkflow = readFileSync(new URL("../.github/workflows/ios-ui-tests.yml", import.meta.url), "utf8");
const validFiles = {
  ".github/workflows/ios-ui-tests.yml": validWorkflow,
  "scripts/boot-ios-ui-backend.mjs": validLauncher,
  "ios/Sources/ConsoleApp/Info.plist": readFileSync(new URL("../ios/Sources/ConsoleApp/Info.plist", import.meta.url), "utf8"),
  "ios/Sources/ConsoleApp/ConsoleApp.swift": readFileSync(new URL("../ios/Sources/ConsoleApp/ConsoleApp.swift", import.meta.url), "utf8"),
  "ios/Sources/ConsoleCore/PersistenceStores.swift": readFileSync(new URL("../ios/Sources/ConsoleCore/PersistenceStores.swift", import.meta.url), "utf8"),
  "ios/Sources/ConsoleApp/ConsoleAccessibilityID.swift": readFileSync(new URL("../ios/Sources/ConsoleApp/ConsoleAccessibilityID.swift", import.meta.url), "utf8"),
  "ios/Sources/ConsoleApp/ConsoleViews.swift": readFileSync(new URL("../ios/Sources/ConsoleApp/ConsoleViews.swift", import.meta.url), "utf8"),
  "ios/Sources/ConsoleApp/CameraCaptureView.swift": readFileSync(new URL("../ios/Sources/ConsoleApp/CameraCaptureView.swift", import.meta.url), "utf8"),
  "ios/UITests/Support/ConsoleUITestCase.swift": readFileSync(new URL("../ios/UITests/Support/ConsoleUITestCase.swift", import.meta.url), "utf8"),
  "ios/UITests/AccessibilityAuditUITests.swift": readFileSync(new URL("../ios/UITests/AccessibilityAuditUITests.swift", import.meta.url), "utf8"),
  "ios/UITests/DynamicTypeRuntimeUITests.swift": readFileSync(new URL("../ios/UITests/DynamicTypeRuntimeUITests.swift", import.meta.url), "utf8"),
  "ios/UITests/Support/RealSessionSeed.swift": readFileSync(new URL("../ios/UITests/Support/RealSessionSeed.swift", import.meta.url), "utf8"),
  "ios/Sources/ConsoleUITestSeeder/UITestSeederApp.swift": readFileSync(new URL("../ios/Sources/ConsoleUITestSeeder/UITestSeederApp.swift", import.meta.url), "utf8"),
  "ios/Config/App.xcconfig": readFileSync(new URL("../ios/Config/App.xcconfig", import.meta.url), "utf8"),
  "ios/Config/ConsoleApp.entitlements": readFileSync(new URL("../ios/Config/ConsoleApp.entitlements", import.meta.url), "utf8"),
  "ios/Config/ConsoleUITestSeeder.entitlements": readFileSync(new URL("../ios/Config/ConsoleUITestSeeder.entitlements", import.meta.url), "utf8"),
  "ios/project.yml": readFileSync(new URL("../ios/project.yml", import.meta.url), "utf8"),
  "ios/UITests/ConsoleCriticalPathUITests.swift": readFileSync(new URL("../ios/UITests/ConsoleCriticalPathUITests.swift", import.meta.url), "utf8"),
  "ios/UITests/MessengerUITests.swift": readFileSync(new URL("../ios/UITests/MessengerUITests.swift", import.meta.url), "utf8"),
  "ios/UITests/CameraCaptureUITests.swift": readFileSync(new URL("../ios/UITests/CameraCaptureUITests.swift", import.meta.url), "utf8"),
  "ios/UITests/PreflightUITests.swift": readFileSync(new URL("../ios/UITests/PreflightUITests.swift", import.meta.url), "utf8"),
  "ios/UITests/XCTestPrewarmUITests.swift": "",
  "ios/UITests/LoginValidationUITests.swift": readFileSync(new URL("../ios/UITests/LoginValidationUITests.swift", import.meta.url), "utf8"),
  "e2e/harness/seed-mobile-ci.sql": readFileSync(new URL("../e2e/harness/seed-mobile-ci.sql", import.meta.url), "utf8"),
};
const evaluate = (overrides = {}) => evaluateIosUiTestFailClosedChecks({ ...validFiles, ...overrides });
const expectsFailure = (result, fragment) => assert.ok(result.failures.some((failure) => failure.includes(fragment)), `Expected ${fragment}: ${result.failures}`);
const mutateWorkflow = (search, replacement) => {
  const mutated = validWorkflow.replace(search, replacement);
  assert.notEqual(mutated, validWorkflow, `Workflow mutation source was not found: ${String(search)}`);
  return mutated;
};
const mutateWorkflowAll = (search, replacement) => {
  const mutated = validWorkflow.replaceAll(search, replacement);
  assert.notEqual(mutated, validWorkflow, `Workflow mutation source was not found: ${String(search)}`);
  return mutated;
};
const mutateFile = (source, search, replacement) => {
  const mutated = source.replace(search, replacement);
  assert.notEqual(mutated, source, `File mutation source was not found: ${String(search)}`);
  return mutated;
};
const mutateFileAll = (source, search, replacement) => {
  const mutated = source.replaceAll(search, replacement);
  assert.notEqual(mutated, source, `File mutation source was not found: ${String(search)}`);
  return mutated;
};
const moveReportFeedbackAfterCamera = (source) => {
  const feedbackStart = source.indexOf("                        if let messageKey = viewModel.messageKey {");
  const feedbackEnd = source.indexOf("\n                    }", feedbackStart);
  assert.ok(feedbackStart >= 0 && feedbackEnd > feedbackStart, "report feedback block must exist");
  const feedback = source.slice(feedbackStart, feedbackEnd);
  const withoutFeedback = source.slice(0, feedbackStart) + source.slice(feedbackEnd);
  const camera = withoutFeedback.indexOf("ConsoleAccessibilityID.detailCaptureEvidenceButton");
  const cameraSectionEnd = withoutFeedback.indexOf("\n                    }", camera);
  assert.ok(camera >= 0 && cameraSectionEnd > camera, "camera section must exist");
  return withoutFeedback.slice(0, cameraSectionEnd) + `\n\n${feedback}` + withoutFeedback.slice(cameraSectionEnd);
};

describe("iOS hermetic UI CI contract", () => {
  it("accepts the hosted, sharded hermetic workflow", () => assert.deepEqual(evaluate().failures, []));
  it("rejects public self-hosted or configurable runner exposure", () => {
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("runs-on: macos-26", "runs-on: ${{ vars.CONSOLE_IOS_CI_RUNNER }}") }), "untrusted PR code");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("runs-on: macos-26", "runs-on: [self-hosted, macos]") }), "untrusted PR code");
  });
  it("rejects non-exact checkout or cross-batch resource and artifact collisions", () => {
    const isolationGate = "batch-unique job-root";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("ref: ${{ github.sha }}", "ref: main") }), isolationGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'D="$RUNNER_TEMP/ios-ui-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}-${CONSOLE_IOS_BATCH_NAME}"',
      'D="$RUNNER_TEMP/ios-ui-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"',
    ) }), isolationGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '"Maintenance CI ${CONSOLE_IOS_BATCH_NAME}-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"',
      '"Maintenance CI ${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"',
    ) }), isolationGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('name: "ios-ui-test-results-${{ matrix.batch }}"', "name: ios-ui-test-results") }), isolationGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '${{ github.run_attempt }}-${{ matrix.batch }}/artifacts',
      '${{ github.run_attempt }}/artifacts',
    ) }), isolationGate);
  });
  it("rejects watchdog inflation, incomplete batches, or unbounded matrix fanout", () => {
    const matrixGate = "seven bounded isolated shard batches";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("timeout-minutes: 45", "timeout-minutes: 44") }), matrixGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("timeout-minutes: 45", "timeout-minutes: 90") }), matrixGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("fail-fast: false", "fail-fast: true") }), matrixGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("max-parallel: 5", "max-parallel: 15") }), matrixGate);
    for (const [shard, budget, invalidBudget] of [
      ["preflight-session", 60, 30],
      ["preflight-restore", 150, 120],
      ["critical-report", 360, 540],
            ["camera-capture", 150, 90],
      ["messenger-mutation", 240, 120],
    ]) {
      expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
        `${shard})
                SHARD_TIMEOUT_SECONDS=${budget}`,
        `${shard})
                SHARD_TIMEOUT_SECONDS=${invalidBudget}`,
      ) }), matrixGate);
    }
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'shards: "critical-today camera-capture critical-report"',
      'shards: "critical-today"',
    ) }), matrixGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'shards: "critical-today camera-capture critical-report"',
      'shards: "critical-today camera-capture critical-report preflight-session"',
    ) }), matrixGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("-parallel-testing-enabled NO", "-parallel-testing-enabled YES") }), matrixGate);
  });
  it("rejects a standalone XCTest prewarm, non-functional first shard, or functional-result substitution", () => {
    const coldStartGate = "start cold-sensitive workers with complete functional proofs";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "          TEST_STATUS=0",
      "          TEST_STATUS=0\n          timing_start xctest-prewarm",
    ) }), coldStartGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'shards: "authenticated-shell preflight-restore preflight-session preflight-fixtures login-validation accessibility-id-parity critical-location"',
      'shards: "preflight-restore authenticated-shell preflight-session preflight-fixtures login-validation accessibility-id-parity critical-location"',
    ) }), coldStartGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'shards: "messenger-mutation messenger-render audit-dynamic-today audit-dynamic-detail"',
      'shards: "messenger-render messenger-mutation audit-dynamic-today audit-dynamic-detail"',
    ) }), coldStartGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "SHARD_SELECTORS=(ConsoleUITests/PreflightUITests/testSeederRestoresThenClearsRealSession)",
      "SHARD_SELECTORS=(ConsoleUITests/XCTestPrewarmUITests/testRunnerAndHostLaunch)",
    ) }), coldStartGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "ConsoleUITests/MessengerUITests/testMessengerSendSurvivesBackendRefresh",
      "ConsoleUITests/MessengerUITests/testExactSeededMessengerThreadAndMessageRender",
    ) }), coldStartGate);
  });
  it("rejects toolchain and job-root drift", () => {
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("Build version 17F113", "Build version drift") }), "pin Xcode 26.6");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("Apple Swift version 6.3.3", "Apple Swift version drift") }), "pin Xcode 26.6");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("iOS-26-5", "iOS-26-4") }), "pin Xcode 26.6");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflowAll("SimDeviceType.iPhone-17-Pro", "SimDeviceType.iPhone-16") }), "pin Xcode 26.6");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("CARGO_TARGET_DIR=$D/cargo-target", "CARGO_TARGET_DIR=/tmp/cargo") }), "pin Xcode 26.6");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "      DEVELOPER_DIR: /Applications/Xcode_26.6.app/Contents/Developer",
      "      DEVELOPER_DIR: /Applications/Xcode_26.6.app/Contents/Developer\n      RUNNER_TOOL_CACHE: ${{ github.workspace }}/shadow-cache",
    ) }), "bind Node");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '        env: {CARGO_INCREMENTAL: "0", CARGO_PROFILE_DEV_DEBUG: "0", SQLX_OFFLINE: "true"}',
      '        env: {CARGO_INCREMENTAL: "0", SQLX_OFFLINE: "true", RUNNER_ARCH: X64, RUNNER_TOOL_CACHE: ${{ github.workspace }}/shadow-cache}',
    ) }), "bind Node");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "jobs:",
      "env:\n  RUNNER_TOOL_CACHE: ${{ github.workspace }}/shadow-cache\njobs:",
    ) }), "bind Node");
    for (const [name, value] of [
      ["BASH_ENV", "${{ github.workspace }}/scripts/bash-env-hook"],
      ["NODE_OPTIONS", "--require=${{ github.workspace }}/scripts/node-hook.cjs"],
    ]) {
      expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
        '        env: {CARGO_INCREMENTAL: "0", CARGO_PROFILE_DEV_DEBUG: "0", SQLX_OFFLINE: "true"}',
        `        env: {CARGO_INCREMENTAL: "0", SQLX_OFFLINE: "true", ${name}: ${value}}`,
      ) }), "bind Node");
    }
    for (const shell of [
      `"bash -c 'source $GITHUB_WORKSPACE/scripts/evil; source {0}'"`,
      `"BASH_ENV=$GITHUB_WORKSPACE/scripts/evil bash {0}"`,
    ]) {
      expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
        "        working-directory: ios\n        shell: bash",
        `        working-directory: ios\n        shell: ${shell}`,
      ) }), "bind Node");
    }
  });
  it("rejects any target or xcconfig that falls back from Swift 6 language mode", () => {
    expectsFailure(evaluate({ "ios/project.yml": mutateFile(
      validFiles["ios/project.yml"],
      '        GENERATE_INFOPLIST_FILE: "YES"\n        INFOPLIST_KEY_CFBundleDisplayName: "ConsoleUITests"',
      '        SWIFT_VERSION: "5.0"\n        GENERATE_INFOPLIST_FILE: "YES"\n        INFOPLIST_KEY_CFBundleDisplayName: "ConsoleUITests"',
    ) }), "Swift 6 language mode");
    expectsFailure(evaluate({ "ios/Config/App.xcconfig": mutateFile(
      validFiles["ios/Config/App.xcconfig"],
      "SWIFT_VERSION = 6.0",
      "SWIFT_VERSION = 5.0",
    ) }), "Swift 6 language mode");
  });
  it("rejects release-optimized or unstripped backend builds in behavioral E2E", () => {
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('CARGO_PROFILE_DEV_DEBUG: "0", ', "") }), "stripped-debug console-app");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("cargo build --locked -p console-app", "cargo build --locked --release -p console-app") }), "stripped-debug console-app");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("$CARGO_TARGET_DIR/debug/console-app", "$CARGO_TARGET_DIR/release/console-app") }), "stripped-debug console-app");
  });
  it("rejects missing pipeline phase or shard timing evidence", () => {
    const timingGate = "durable phase and per-shard timings";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('TIMINGS="$ARTIFACTS/pipeline-timings.tsv"', 'TIMINGS="$ARTIFACTS/missing.tsv"') }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("timing_start rust-debug-build", "true # missing rust timing") }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('TIMING_BUDGET_SECONDS="$SHARD_TIMEOUT_SECONDS"', 'TIMING_BUDGET_SECONDS="-"') }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('timing_start "test:$shard_name"', 'timing_start "test:unknown"') }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("trap on_exit EXIT", "trap 'clean_runtime || true' EXIT") }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('timing_finish "aborted(exit=$exit_status)"', "true # missing aborted timing") }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("status=124", "status=0 # missing timeout classification") }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("timing_finish timeout", "timing_finish failed") }), timingGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("### iOS UI pipeline timings", "### missing timings") }), timingGate);
  });
  it("rejects mutable XcodeGen and unsafe PGDATA", () => {
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("4d9e34b62172d645eed6457cac13fc222569974098ef4ee9c3368bedf0196806", "dynamic") }), "checksum-pinned XcodeGen");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('install -d -m 700 "$D" "$AUTH_DIR" "$PGDATA" "$RAW_RESULTS" "$ARTIFACTS"', 'mkdir -p /tmp/pg') }), "mode-0700 job-root PGDATA");
  });
  it("rejects PostgreSQL builds that omit or cannot load the complete extension set", () => {
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" --with-ssl=openssl", "") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("--with-ssl=openssl", "--without-ssl # --with-ssl=openssl") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(' export CPPFLAGS="-I$OPENSSL_PREFIX/include" LDFLAGS="-L$OPENSSL_PREFIX/lib" PKG_CONFIG_PATH="$OPENSSL_PREFIX/lib/pkgconfig"', "") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(' && make -C contrib/pgcrypto -j"$(sysctl -n hw.ncpu)"', "") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" && make -C contrib/pgcrypto install", "") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" -c 'CREATE EXTENSION pgcrypto;'", "") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" -c 'DROP EXTENSION pgcrypto;'", "") }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(' && make -C contrib/pg_trgm -j"$(sysctl -n hw.ncpu)"', "") }), "pg_trgm");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" && make -C contrib/pg_trgm install", "") }), "pg_trgm");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" -c 'CREATE EXTENSION pg_trgm;'", "") }), "pg_trgm");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(" -c 'DROP EXTENSION pg_trgm;'", "") }), "pg_trgm");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(' && make -C contrib/pgcrypto -j"$(sysctl -n hw.ncpu)" && make -C contrib/pgcrypto install', ' # make -C contrib/pgcrypto -j"$(sysctl -n hw.ncpu)" && make -C contrib/pgcrypto install') }), "pgcrypto");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(' && make -C contrib/pg_trgm -j"$(sysctl -n hw.ncpu)" && make -C contrib/pg_trgm install', ' # make -C contrib/pg_trgm -j"$(sysctl -n hw.ncpu)" && make -C contrib/pg_trgm install') }), "pg_trgm");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('          PGPASSWORD="$UP"', '          # PGPASSWORD="$UP"') }), "pg_trgm");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('"$PG_PREFIX/bin/pg_ctl" -D "$PGDATA" -o "-h 127.0.0.1 -p $PP" -w start', "true") }), "pg_trgm");
  });
  it("rejects invalid WebAuthn relying-party configuration", () => {
    expectsFailure(evaluate({ "scripts/boot-ios-ui-backend.mjs": validLauncher.replace("http://localhost:", "http://127.0.0.1:") }), "WebAuthn");
    expectsFailure(evaluate({ "scripts/boot-ios-ui-backend.mjs": validLauncher.replace('E2E_RP_ID: "localhost"', 'E2E_RP_ID: "localhost.evil"') }), "WebAuthn");
    expectsFailure(evaluate({ "scripts/boot-ios-ui-backend.mjs": validLauncher.replace("shell: false", "shell: true") }), "WebAuthn");
    expectsFailure(evaluate({ "scripts/boot-ios-ui-backend.mjs": validLauncher.replace("delete env.CONSOLE_IOS_COLDSTART_OTP;", "") }), "WebAuthn");
    expectsFailure(evaluate({ "scripts/boot-ios-ui-backend.mjs": "" }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, `${validBoot}\n          ${validBoot.trim()}`) }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, '          CONSOLE_IOS_COLDSTART_OTP="$COLDSTART_OTP" node "$ROOT/scripts/not-the-launcher.mjs" "$ROOT" "$AUTH_DIR" "$BP"') }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, '          E2E_AUTH_DIR="$AUTH_DIR" E2E_HTTP_ADDR="127.0.0.1:$BP" E2E_RP_ORIGIN="$URL" E2E_RP_ID=127.0.0.1 "$ROOT/e2e/harness/boot-backend.sh"') }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, `          cat <<'EOF'\n${validBoot}\n          EOF\n          E2E_AUTH_DIR="$AUTH_DIR" E2E_HTTP_ADDR="127.0.0.1:$BP" E2E_RP_ORIGIN="$URL" E2E_RP_ID=127.0.0.1 "$ROOT/e2e/harness/boot-backend.sh"`) }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, `          cat <<'EOF'\n${validBoot}\n          EOF`) }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, `          PAYLOAD='\n${validBoot}\n          '\n          node "$ROOT/scripts/alternate-backend-launcher.mjs"`) }), "WebAuthn");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(validBoot, `${validBoot}\n          node "$ROOT/scripts/alternate-backend-launcher.mjs"`) }), "WebAuthn");
    const shadowedNode = validWorkflow
      .replace(validBoot, validBoot.replace('"$CONSOLE_IOS_NODE_BIN"', "node"))
      .replace('          "$ROOT/e2e/harness/db.sh"', `          node() { command node "$ROOT/scripts/alternate-backend-launcher.mjs"; }\n          "$ROOT/e2e/harness/db.sh"`);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": shadowedNode }), "WebAuthn");
    const pathShadowedNode = validWorkflow
      .replace(validBoot, validBoot.replace('"$CONSOLE_IOS_NODE_BIN"', "/usr/bin/env node"))
      .replace('          "$ROOT/e2e/harness/db.sh"', `          PATH="$ROOT/scripts/shadow:$PATH"\n          "$ROOT/e2e/harness/db.sh"`);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": pathShadowedNode }), "WebAuthn");
    const reassignedTrustedNode = mutateWorkflow(
      '          readonly CONSOLE_IOS_NODE_BIN="$RUNNER_TOOL_CACHE/node/24.16.0/$NODE_ARCH/bin/node"',
      '          readonly CONSOLE_IOS_NODE_BIN="$RUNNER_TOOL_CACHE/node/24.16.0/$NODE_ARCH/bin/node"\n          CONSOLE_IOS_NODE_BIN="$GITHUB_WORKSPACE/scripts/shadow-node"',
    );
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": reassignedTrustedNode }), "bind Node");
    const capturePathPoisonedNode = mutateWorkflow(
      "          set -euo pipefail\n          unset BASH_ENV",
      "          set -euo pipefail\n          PATH=\"$GITHUB_WORKSPACE/scripts/shadow:$PATH\"\n          unset BASH_ENV",
    );
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": capturePathPoisonedNode }), "bind Node");
    const githubEnvironmentPoisonedNode = mutateWorkflow(
      '"CONSOLE_IOS_JOB_ROOT=$D" "CARGO_HOME=$D/cargo-home"',
      '"CONSOLE_IOS_NODE_BIN=$GITHUB_WORKSPACE/scripts/shadow-node" "CONSOLE_IOS_JOB_ROOT=$D" "CARGO_HOME=$D/cargo-home"',
    );
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": githubEnvironmentPoisonedNode }), "bind Node");
    const postPidAlternateLauncher = mutateWorkflow(
      '          BACKEND_PID="$(cat "$BACKEND_PID_FILE")"',
      '          BACKEND_PID="$(cat "$BACKEND_PID_FILE")"\n          "$CONSOLE_IOS_NODE_BIN" "$ROOT/scripts/alternate-backend-launcher.mjs"',
    );
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": postPidAlternateLauncher }), "WebAuthn");
    const postPidAlternateShellLauncher = mutateWorkflow(
      '          BACKEND_PID="$(cat "$BACKEND_PID_FILE")"',
      '          BACKEND_PID="$(cat "$BACKEND_PID_FILE")"\n          "$ROOT/scripts/alternate-backend-launcher.sh"',
    );
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": postPidAlternateShellLauncher }), "WebAuthn");
  });
  it("rejects missing or FK-unsafe per-class fixture restoration", () => {
    const seed = validFiles["e2e/harness/seed-mobile-ci.sql"];
    expectsFailure(evaluate({
      "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "DELETE FROM location_consent_ledger", "-- missing consent ledger reset"),
    }), "exact mutable mobile fixture baseline");
    expectsFailure(evaluate({
      "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "DELETE FROM messenger_read_receipts", "DELETE FROM messenger_messages"),
    }), "exact mutable mobile fixture baseline");
    expectsFailure(evaluate({
      "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "ELSE 'IN_PROGRESS'", "ELSE 'REPORT_SUBMITTED'"),
    }), "exact mutable mobile fixture baseline");
    expectsFailure(evaluate({
      "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "DELETE FROM work_order_approval_steps", "DELETE FROM audit_events"),
    }), "exact mutable mobile fixture baseline");
    expectsFailure(evaluate({
      "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "00000000-0000-0000-0000-000000c20008", "00000000-0000-0000-0000-000000c20001"),
    }), "isolated one-row Today and Messenger fixture");
  });

  it("rejects accessibility fixture-profile cross-contamination", () => {
    const seed = validFiles["e2e/harness/seed-mobile-ci.sql"];
    const profileGate = "isolated one-row Today and Messenger fixture";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      `audit-dynamic-today)
                SHARD_FIXTURE_PROFILE=accessibility-audit-one-row`,
      `audit-dynamic-today)
                SHARD_FIXTURE_PROFILE=full`,
    ) }), profileGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      `dynamic-type-ax5)
                SHARD_FIXTURE_PROFILE=accessibility-audit-one-row`,
      `dynamic-type-ax5)
                SHARD_FIXTURE_PROFILE=full`,
    ) }), profileGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("*) return 1 ;;", "*) SHARD_TIMEOUT_SECONDS=1 ;;") }), profileGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('mint_shard_session "$SHARD_FIXTURE_PROFILE"', 'mint_shard_session full') }), profileGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(' -v "fixture_profile=$fixture_profile"', "") }), profileGate);
    expectsFailure(evaluate({ "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "\\if :{?fixture_profile}", "\\if false") }), profileGate);
    expectsFailure(evaluate({ "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "IN ('full', 'accessibility-audit-one-row')", "IN ('full', 'accessibility-audit-one-row', 'anything')") }), profileGate);
    expectsFailure(evaluate({ "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "COUNT(*) = 5", "COUNT(*) = 3") }), profileGate);
    expectsFailure(evaluate({ "e2e/harness/seed-mobile-ci.sql": mutateFile(seed, "COUNT(*) = 8", "COUNT(*) = 7") }), profileGate);
  });
  it("rejects missing, mismatched, or wrongly wired app/seeder keychain configuration", () => {
    expectsFailure(evaluate({
      "ios/Config/ConsoleUITestSeeder.entitlements": validFiles["ios/Config/ConsoleUITestSeeder.entitlements"].replace("com.console.app.shared", "com.console.app.wrong"),
    }), "identically signed default keychain access group");
    expectsFailure(evaluate({
      "ios/Config/ConsoleUITestSeeder.entitlements": validFiles["ios/Config/ConsoleUITestSeeder.entitlements"].replace("<key>keychain-access-groups</key>", "<key>missing-keychain-access-groups</key>"),
    }), "identically signed default keychain access group");
    expectsFailure(evaluate({
      "ios/project.yml": validFiles["ios/project.yml"].replace("Sources/ConsoleUITestSeeder", "Sources/MissingSeeder"),
    }), "identically signed default keychain access group");
    expectsFailure(evaluate({
      "ios/project.yml": validFiles["ios/project.yml"].replace("- target: ConsoleUITestSeeder", "- target: MissingSeeder"),
    }), "identically signed default keychain access group");
    expectsFailure(evaluate({
      "ios/project.yml": validFiles["ios/project.yml"].replace("CODE_SIGN_ENTITLEMENTS: Config/ConsoleUITestSeeder.entitlements", "CODE_SIGN_ENTITLEMENTS: Config/Missing.entitlements"),
    }), "identically signed default keychain access group");
    expectsFailure(evaluate({
      "ios/Config/App.xcconfig": validFiles["ios/Config/App.xcconfig"].replace("CODE_SIGNING_ALLOWED = YES", "CODE_SIGNING_ALLOWED = NO"),
    }), "identically signed default keychain access group");
  });
  it("rejects a UI-test Keychain implementation instead of the dedicated helper", () => {
    expectsFailure(evaluate({
      "ios/UITests/Support/RealSessionSeed.swift": `${validFiles["ios/UITests/Support/RealSessionSeed.swift"]}\nimport Security\nlet forbidden = kSecAttrAccessGroup`,
    }), "system-granted default group");
    expectsFailure(evaluate({
      "ios/Sources/ConsoleUITestSeeder/UITestSeederApp.swift": validFiles["ios/Sources/ConsoleUITestSeeder/UITestSeederApp.swift"].replace("KeychainAccessGroup.resolveShared", "MissingAccessGroup.resolveShared"),
    }), "system-granted default group");
    expectsFailure(evaluate({
      "ios/Sources/ConsoleCore/PersistenceStores.swift": validFiles["ios/Sources/ConsoleCore/PersistenceStores.swift"].replace(
        "        let add: [String: Any] = [\n            kSecClass as String: kSecClassGenericPassword,\n            kSecAttrService as String: service,\n            kSecAttrAccount as String: account,",
        "        let add: [String: Any] = [\n            kSecClass as String: kSecClassGenericPassword,\n            kSecAttrService as String: service,\n            kSecAttrAccount as String: account,\n            kSecAttrAccessGroup as String: \"forbidden\",",
      ),
    }), "system-granted default group");
    expectsFailure(evaluate({
      "ios/Sources/ConsoleCore/PersistenceStores.swift": validFiles["ios/Sources/ConsoleCore/PersistenceStores.swift"].replace(
        "guard let result = try? probe.addProbe",
        "guard let result = try! probe.addProbe",
      ),
    }), "system-granted default group");
    expectsFailure(evaluate({
      "ios/Sources/ConsoleCore/PersistenceStores.swift": mutateFile(
        validFiles["ios/Sources/ConsoleCore/PersistenceStores.swift"],
        "try probe.deleteProbe",
        "try! probe.deleteProbe",
      ),
    }), "system-granted default group");
  });
  it("rejects UI automation that can launch outside the main actor", () => {
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    const fakeFieldContract = `@MainActor
class ConsoleUITestCase: XCTestCase {
  override func setUpWithError() throws {
    try super.setUpWithError()
    try RealSessionSeed.seed(tokens)
  }
  override func tearDownWithError() throws {
    try RealSessionSeed.clear()
    try super.tearDownWithError()
  }
}`;
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("@MainActor\nclass ConsoleUITestCase", "class ConsoleUITestCase"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("setUpWithError() throws", "setUp() async throws"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("tearDownWithError() throws", "tearDown() async throws"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/RealSessionSeed.swift": validFiles["ios/UITests/Support/RealSessionSeed.swift"].replace("@MainActor\nenum RealSessionSeed", "enum RealSessionSeed"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/PreflightUITests.swift": validFiles["ios/UITests/PreflightUITests.swift"].replace("@MainActor\n", ""),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/LoginValidationUITests.swift": validFiles["ios/UITests/LoginValidationUITests.swift"].replace("@MainActor\n", ""),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": `/* ${fakeFieldContract} */\n${fieldCase.replace("@MainActor\nclass ConsoleUITestCase", "class ConsoleUITestCase")}`,
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": `let fake = """\n${fakeFieldContract}\n"""\n${fieldCase.replace("@MainActor\nclass ConsoleUITestCase", "class ConsoleUITestCase")}`,
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": `let fake = """\n\\\"""\n${fakeFieldContract}\n"""\n${fieldCase.replace("@MainActor\nclass ConsoleUITestCase", "class ConsoleUITestCase")}`,
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": `let fake = #"""\n\\#"""#\n${fakeFieldContract}\n"""#\n${fieldCase.replace("@MainActor\nclass ConsoleUITestCase", "class ConsoleUITestCase")}`,
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("@MainActor\nclass ConsoleUITestCase", "// @MainActor\nclass ConsoleUITestCase"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/Support/RealSessionSeed.swift": validFiles["ios/UITests/Support/RealSessionSeed.swift"].replace("@MainActor\nenum RealSessionSeed", "// @MainActor\nenum RealSessionSeed"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/PreflightUITests.swift": validFiles["ios/UITests/PreflightUITests.swift"].replace("@MainActor\n", "// @MainActor\n"),
    }), "confine XCUIApplication");
    expectsFailure(evaluate({
      "ios/UITests/LoginValidationUITests.swift": validFiles["ios/UITests/LoginValidationUITests.swift"].replace("@MainActor\n", "// @MainActor\n"),
    }), "confine XCUIApplication");
  });
  it("rejects Runner mutation, stale Runner environment injection, and incomplete Mach-O entitlement proof", () => {
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '/usr/bin/codesign --verify --deep --strict "$SEEDER_APP"',
      "true",
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '/usr/bin/codesign --verify --deep --strict "$UITEST_RUNNER_APP"',
      '/usr/bin/codesign --force --sign - "$UITEST_RUNNER_APP"',
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'test "$APP_KEYCHAIN_GROUP" = "$SEEDER_KEYCHAIN_GROUP"',
      "true",
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'APP_KEYCHAIN_GROUP="$(mach_o_keychain_group "$BUILT_APP/ConsoleApp")"',
      'APP_KEYCHAIN_GROUP="missing"',
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '/usr/bin/lipo -archs "$executable"',
      'false # missing lipo architecture inspection',
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '/bin/cp "$executable" "$thin"',
      '/usr/bin/lipo "$executable" -thin "$MACH_O_ARCH" -output "$thin"',
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '_, _, segment, _, _, _, _, _, _, nsects, _ = struct.unpack_from("<II16sQQQQiiII", executable, offset)',
      '_, _, segment, _, _, _, _, _, nsects, _ = struct.unpack_from("<II16sQQQQiiII", executable, offset)',
    ) }), "preserve the Xcode-created Simulator Runner");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "--env CONSOLE_UITEST_BASE_URL",
      "--env CONSOLE_IOS_KEYCHAIN_GROUP --env CONSOLE_UITEST_BASE_URL",
    ) }), "preserve the Xcode-created Simulator Runner");
  });
  it("rejects xctestrun, ATS, and fail-slow xcresult regression", () => {
    const failSlow = "each iOS UI matrix worker";
    const xctestrunGate = "inject the UI host path and renewable session material only per functional shard";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      'chmod 600 "$XCTESTRUN"',
      'chmod 600 "$XCTESTRUN"\n          (cd "$ROOT" && python3 scripts/patch-ios-xctestrun.py "$XCTESTRUN" --target ConsoleUITests --ui-target-app-path \'__TESTROOT__/Debug-iphonesimulator/ConsoleApp.app\')',
    ) }), xctestrunGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "--ui-target-app-path '__TESTROOT__/Debug-iphonesimulator/ConsoleApp.app' --env CONSOLE_UITEST_BASE_URL",
      "--env CONSOLE_UITEST_BASE_URL",
    ) }), xctestrunGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('chmod 600 "$XCTESTRUN"', "") }), "mode-0600");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('CI_PLIST="$D/Info.ci.plist"', 'CI_PLIST="$RUNNER_TEMP/Info.plist"') }), "CI-only job-root");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('run_xcode_with_timeout "$shard_name" "$result" "$SHARD_TIMEOUT_SECONDS" "${SHARD_SELECTORS[@]}" || { shard_status=$?; TEST_STATUS=1; }', 'run_xcode_with_timeout "$shard_name" "$result" "$SHARD_TIMEOUT_SECONDS" "${SHARD_SELECTORS[@]}"') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("os.setsid(); ", "") }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflowAll('kill -TERM -- "-$test_pid"', 'kill -TERM "$test_pid"') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflowAll('kill -KILL -- "-$test_pid"', 'kill -KILL "$test_pid"') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if ! configure_shard "$shard_name"; then', 'if configure_shard "$shard_name"; then') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if ! set_simulator_presentation "$SHARD_APPEARANCE" "$SHARD_CONTENT_SIZE"; then', 'if set_simulator_presentation "$SHARD_APPEARANCE" "$SHARD_CONTENT_SIZE"; then') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if ! mint_shard_session "$SHARD_FIXTURE_PROFILE"; then', 'if mint_shard_session "$SHARD_FIXTURE_PROFILE"; then') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if [[ "$shard_name" == camera-capture ]] && ! xcrun simctl privacy "$UUID" reset camera; then', 'if [[ "$shard_name" == camera-capture ]] && xcrun simctl privacy "$UUID" reset camera; then') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(`timing_finish setup-failed
              continue`, `timing_finish setup-failed
              :`) }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('read -r -a SHARD_MANIFEST <<< "$CONSOLE_IOS_SHARD_BATCH"', "SHARD_MANIFEST=(preflight)") }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('printf \'%s\\n\' "$CONSOLE_IOS_BATCH_NAME" > "$ARTIFACTS/batch-name.txt"', "true") }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if ! clean_runtime; then', 'if clean_runtime; then') }), failSlow);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('exit "$TEST_STATUS"', 'exit 0') }), failSlow);
  });
  it("rejects incomplete or fail-open cross-worker result aggregation", () => {
    const aggregateGate = "exactly one structured summary and tests JSON";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      '" = 21',
      '" = 20',
    ) }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("needs: ios-ui-tests", "needs: []") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      `ios-ui-results:
    name: iOS — aggregate structured results
    needs: ios-ui-tests
    if: always()`,
      `ios-ui-results:
    name: iOS — aggregate structured results
    needs: ios-ui-tests`,
    ) }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("pattern: ios-ui-test-results-*", "pattern: ios-ui-test-results-core") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('name: "ios-ui-test-results-${{ matrix.batch }}"', "name: ios-ui-test-results") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("if-no-files-found: error", "if-no-files-found: warn") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "EXPECTED_BATCHES=(core critical-core messenger-dynamic audit-standard audit-adaptive)",
      "EXPECTED_BATCHES=(core critical-core messenger-dynamic audit-standard)",
    ) }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "EXPECTED_SHARDS=(authenticated-shell preflight-restore preflight-session preflight-fixtures login-validation accessibility-id-parity",
      "EXPECTED_SHARDS=(login-validation accessibility-id-parity",
    ) }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "core) expected_manifest='authenticated-shell preflight-restore preflight-session preflight-fixtures login-validation accessibility-id-parity critical-location'",
      "core) expected_manifest='preflight-restore authenticated-shell preflight-session preflight-fixtures login-validation accessibility-id-parity critical-location'",
    ) }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(
      "EXPECTED_SHARDS=(authenticated-shell preflight-restore preflight-session preflight-fixtures login-validation accessibility-id-parity",
      "EXPECTED_SHARDS=(preflight-restore authenticated-shell preflight-session preflight-fixtures login-validation accessibility-id-parity",
    ) }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("((${#summaries[@]} == 1))", "true") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('node scripts/verify-xcresult-test-results.mjs "${VERIFY_ARGS[@]}" --swift-tests ios/UITests', "true") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if [[ "$WORKER_RESULT" != success ]]; then', "if false; then") }), aggregateGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflowAll('test "$(git rev-parse HEAD)" = "$GITHUB_SHA"', "true") }), aggregateGate);
  });
  it("rejects raw artifact session material and cleanup proof regression", () => {
    const artifactGate = "scan-clean derived diagnostics";
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(`id: artifact-scan
        if: always()`, "id: artifact-scan") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('install -d -m 700 "$D" "$D/auth" "$D/raw-xcresults" "$D/artifacts"', 'install -d -m 700 "$D"') }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow(': > "$SESSION_MINTED_MARKER"', "true") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('if [[ -e "$SESSION_MINTED_MARKER" ]]; then', "if true; then") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("raw-session scan source exists without its session-minted marker", "unmarked raw-session source accepted") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("[[ -s \"$SECRETS_FILE\" ]] ||", "true ||") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflowAll("test artifact contains raw session material", "ignored") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('result="$RAW_RESULTS/$shard_name.xcresult"', 'result="$ARTIFACTS/$shard_name.xcresult"') }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("raw xcresult bundle entered upload tree", "raw xcresult bundle allowed") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("symlink entered upload tree", "symlink allowed") }), artifactGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow("steps.artifact-scan.outcome == 'success'", "true") }), "upload before final");
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflowAll("backend PID identity changed; refusing cross-process cleanup", "backend cleanup ignored identity") }), "identity-aware backend");
  });
  it("rejects fail-open support and accessibility parity drift", () => {
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    const auditTests = validFiles["ios/UITests/AccessibilityAuditUITests.swift"];
    const runtimeTests = validFiles["ios/UITests/DynamicTypeRuntimeUITests.swift"];
    const strictGate = "strict accessibility auditing";
    const presentationGate = "precondition supported Simulator appearance";
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": "throw XCTSkip()" }), "must not include skip-testing");
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "continueAfterFailure = true", "") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "issue.auditType == .dynamicType", "issue.auditType != .dynamicType") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "Dynamic Type font sizes are partially unsupported", "unexpected diagnostic") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "User will not be able to change the font size of this SwiftUI.AccessibilityNode", "unexpected detailed diagnostic") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "issue.auditType == .dynamicType,", "issue.auditType != .dynamicType,") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, `XCTAssertEqual(
            observed.sorted(),
            expectedCompatibilityIssues.sorted(),`, "true") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "try app.performAccessibilityAudit(for: .all.subtracting(.dynamicType))", "try app.performAccessibilityAudit(for: .all)") }), strictGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('xcrun simctl ui "$UUID" appearance "$expected_appearance"', "true") }), presentationGate);
    expectsFailure(evaluate({ ".github/workflows/ios-ui-tests.yml": mutateWorkflow('actual_content_size="$(xcrun simctl ui "$UUID" content_size)"', 'actual_content_size="$expected_content_size"') }), presentationGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": `${fieldCase}
app.launchArguments += ["-UIPreferredContentSizeCategoryName", "UICTContentSizeCategoryL"]` }), presentationGate);
    expectsFailure(evaluate({ "ios/UITests/Support/ConsoleUITestCase.swift": `${fieldCase}
XCUIDevice.shared.appearance = .dark` }), presentationGate);
    expectsFailure(evaluate({ "ios/UITests/AccessibilityAuditUITests.swift": mutateFile(auditTests, "testTodayScreenPassesDynamicTypeAudit", "testTodayScreenPassesNonDynamicAuditStandard") }), presentationGate);
    expectsFailure(evaluate({ "ios/UITests/AccessibilityAuditUITests.swift": mutateFile(auditTests, "AID.locationConsentGrantButton", "AID.todayRefreshButton") }), strictGate);
    expectsFailure(evaluate({ "ios/UITests/DynamicTypeRuntimeUITests.swift": mutateFile(runtimeTests, "sameHorizontalBand(body.frame, timestamp.frame)", "body.frame.intersects(timestamp.frame)") }), presentationGate);
    expectsFailure(evaluate({ "ios/UITests/DynamicTypeRuntimeUITests.swift": mutateFile(runtimeTests, "XCTAssertGreaterThan(timestamp.frame.minY, body.frame.maxY", "XCTAssertLessThan(timestamp.frame.minY, body.frame.maxY") }), presentationGate);
    expectsFailure(evaluate({ "ios/UITests/DynamicTypeRuntimeUITests.swift": mutateFile(runtimeTests, "XCTAssertGreaterThanOrEqual(app.buttons[AID.locationConsentGrantButton].frame.height, 44)", "XCTAssertGreaterThanOrEqual(app.buttons[AID.locationConsentGrantButton].frame.height, 1)") }), presentationGate);
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleAccessibilityID.swift": `public enum ConsoleAccessibilityID { public static let onlyProduction = "x" }` }), "mirror every ConsoleAccessibilityID");
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "if dynamicTypeSize.isAccessibilitySize == false", "if dynamicTypeSize.isAccessibilitySize") }), "Today must retain inline location consent outside accessibility Dynamic Type");
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "ConsoleAccessibilityID.todayLocationConsentButton", "ConsoleAccessibilityID.todayRefreshButton") }), "Today must retain inline location consent outside accessibility Dynamic Type");
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": `${fieldViews}
Text("fixed").font(.system(size: 17))` }), presentationGate);
  });
  it("rejects a tab whose NavigationStack is not wrapped by the unobscured content host", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "TabBarContentLayoutGuideHost(content: content)", "content") }), "every authenticated iOS tab must use the direct UIKit content-layout-guide host");
  });
  it("rejects tab content without formal UIHostingController containment", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "addChild(hostingController)", "// containment removed") }), "every authenticated iOS tab must use the direct UIKit content-layout-guide host");
  });
  it("rejects a tab host without direct guide constraints or lifecycle teardown", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const guideGate = "every authenticated iOS tab must use the direct UIKit content-layout-guide host";
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "hostingController.view.bottomAnchor.constraint(equalTo: guide.bottomAnchor)", "hostingController.view.bottomAnchor.constraint(equalTo: tabBarController.view.bottomAnchor)") }), guideGate);
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "NSLayoutConstraint.activate(contentLayoutConstraints)", "// activation removed") }), guideGate);
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "hostingController.removeFromParent()", "// teardown removed") }), guideGate);
  });
  it("rejects private tab hierarchy workarounds and fixed bottom clearance", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    for (const forbidden of [
      "tabBarController.selectedViewController = self",
      "let privateHierarchy = view.subviews",
      "view.safeAreaInset(edge: .bottom) { EmptyView() }",
      "view.frame = CGRect(x: 0, y: 0, width: 1, height: 84)",
      "view.traitOverrides.horizontalSizeClass = .compact",
    ]) {
      expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": `${fieldViews}
${forbidden}` }), "every authenticated iOS tab must use the direct UIKit content-layout-guide host");
    }
  });
  it("rejects suppressing or unsanitized accessibility audit issue handlers", () => {
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    const strictGate = "strict accessibility auditing";
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "return false\n            }\n        } catch",
        "return true\n            }\n        } catch",
      ),
    }), strictGate);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "identifier=\\(identifier.debugDescription)",
        "identifier=\\(element?.label.debugDescription ?? \"none\")",
      ),
    }), strictGate);
    const normalDynamicIdentifier = "messenger.threadRow.123e4567-e89b-12d3-a456-426614174000";
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        '"<redacted>"',
        `identifier ?? "${normalDynamicIdentifier}"`,
      ),
    }), strictGate);
    expectsFailure(evaluate({
      "ios/UITests/AccessibilityAuditUITests.swift": `${validFiles["ios/UITests/AccessibilityAuditUITests.swift"]}\nlet issueHandler = { _ in }`,
    }), strictGate);
  });
  it("rejects Messenger section headers that are not scalable semantic in-row content", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const headerGate = "scalable semantic in-row headers";
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `Section {
                Text("messenger_threads")`,
        `Section {
            } header: {
                Text("messenger_threads")`,
      ),
    }), headerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `Text("messenger_threads")
                    .font(.headline)
                    .fixedSize(horizontal: false, vertical: true)`,
        `Text("messenger_threads")
                    .font(.headline)`,
      ),
    }), headerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".accessibilityAddTraits(.isHeader)", ""),
    }), headerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".listRowBackground(Color.clear)", ""),
    }), headerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".listRowSeparator(.hidden)", ""),
    }), headerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `Text("messenger_threads")
                    .font(.headline)`,
        `Text("messenger_threads")
                    .font(.headline)
                    .headerProminence(.increased)`,
      ),
    }), headerGate);
  });
  it("rejects a Messenger composer placeholder with native attenuation or duplicate accessibility", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const composerGate = "primary-foreground semantic placeholder without native placeholder opacity";
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `ZStack(alignment: .leading) {
                    if viewModel.messengerDraft.isEmpty {
                        Text("messenger_composer")
                            .foregroundStyle(.primary)
                            .accessibilityHidden(true)
                            .allowsHitTesting(false)
                    }
                    TextField("", text: $viewModel.messengerDraft, axis: .vertical)`,
        `TextField(
                    "",
                    text: $viewModel.messengerDraft,
                    prompt: Text("messenger_composer").foregroundStyle(.primary),
                    axis: .vertical
                        )`,
      ),
    }), composerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `Text("messenger_composer")
                            .foregroundStyle(.primary)
                            .accessibilityHidden(true)`,
        `Text("messenger_composer")
                            .foregroundStyle(.secondary)
                            .accessibilityHidden(true)`,
      ),
    }), composerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `Text("messenger_composer")
                            .foregroundStyle(.primary)
                            .accessibilityHidden(true)`,
        `Text("messenger_composer")
                            .foregroundStyle(.primary)`,
      ),
    }), composerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "if viewModel.messengerDraft.isEmpty {",
        "if viewModel.messengerDraft.isEmpty == false {",
      ),
    }), composerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `.accessibilityHidden(true)\n                            .allowsHitTesting(false)`,
        `.accessibilityHidden(true)\n                            .allowsHitTesting(true)`,
      ),
    }), composerGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `.accessibilityHidden(true)\n                            .allowsHitTesting(false)`,
        `.accessibilityHidden(true)`,
      ),
    }), composerGate);
  });
  it("rejects a Messenger composer that is not a persistent gated sibling of the List", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const messengerTests = validFiles["ios/UITests/MessengerUITests.swift"];
    const persistentGate = "selected-thread-gated opaque VStack sibling of the List";

    // The send action returns to a row inside the lazy List — the exact defect.
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "VStack(spacing: 0) {\n        List {",
        "VStack(spacing: 0) {\n        List {\n            Button(\"x\") {}\n                .accessibilityIdentifier(ConsoleAccessibilityID.messengerSendButton)",
      ),
    }), persistentGate);

    // Selected-thread gate removed: the bar would render with no thread open.
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "if viewModel.messengerState.selectedThreadID != nil {\n                messengerComposerBar\n            }",
        "messengerComposerBar",
      ),
    }), persistentGate);

    // The List is no longer wrapped, so the bar is not a sibling of it.
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "VStack(spacing: 0) {\n        List {",
        "Group {\n        List {",
      ),
    }), persistentGate);

    // The opaque surface keeping the primary action readable is removed.
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        ".background(Color.opaqueConsoleNavigationBackground)",
        ".background(Color.clear)",
      ),
    }), persistentGate);

    // The List's modifiers drift onto the VStack: safeAreaInset(edge: .top)
    // stops insetting a scroll view and silently becomes layout padding.
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "        .safeAreaInset(edge: .top, spacing: 0) {\n            if dynamicTypeSize == .accessibility5 {\n                Color.clear\n                    .frame(height: 56)\n                    .accessibilityHidden(true)\n            }\n        }\n            if viewModel.messengerState.selectedThreadID != nil {\n                messengerComposerBar\n            }",
        "            if viewModel.messengerState.selectedThreadID != nil {\n                messengerComposerBar\n            }\n        .safeAreaInset(edge: .top, spacing: 0) {\n            if dynamicTypeSize == .accessibility5 {\n                Color.clear\n                    .frame(height: 56)\n                    .accessibilityHidden(true)\n            }\n        }",
      ),
    }), persistentGate);

    // Tests scroll to reach the composer again, re-masking any regression.
    expectsFailure(evaluate({
      "ios/UITests/MessengerUITests.swift": mutateFile(
        messengerTests,
        "let composer = persistentComposer()\n        guard composer.waitForExistence(timeout: 15), waitUntilHittable(composer) else {",
        "let composer = app.descendants(matching: .any)[AID.messengerComposerField]\n        guard scrollToMessengerElement(composer, topSentinel: thread) != nil else {",
      ),
    }), persistentGate);
  });
  it("rejects Messenger AX5 content suppression, missing vertical geometry, and translucent navigation chrome", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const runtimeTests = validFiles["ios/UITests/DynamicTypeRuntimeUITests.swift"];
    const ax5Gate = "complete adaptive thread content and an opaque visible navigation surface";
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "struct MessengerThreadRow: View {\n    let thread: MessengerThread\n    let isSelected: Bool\n    @Environment(\\.dynamicTypeSize) private var dynamicTypeSize\n\n    var body: some View {\n        if dynamicTypeSize.isAccessibilitySize {", "struct MessengerThreadRow: View {\n    let thread: MessengerThread\n    let isSelected: Bool\n    @Environment(\\.dynamicTypeSize) private var dynamicTypeSize\n\n    var body: some View {\n        if dynamicTypeSize.isAccessibilitySize == false {"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".toolbarBackground(Color.opaqueConsoleNavigationBackground, for: .navigationBar)", ".toolbarBackground(Color.clear, for: .navigationBar)"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".toolbarBackground(.visible, for: .navigationBar)", ".toolbarBackground(.automatic, for: .navigationBar)"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".messengerNavigationBarBackground()", ".inlineNavigationTitle()"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "#if os(iOS)\n        self\n            .toolbarBackground(Color.opaqueConsoleNavigationBackground",
        "#if os(macOS)\n        self\n            .toolbarBackground(Color.opaqueConsoleNavigationBackground",
      ),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".font(.headline)\n                ConsoleChip(key: thread.kind.fieldLabelKey)", ".font(.headline)\n                    .lineLimit(1)\n                ConsoleChip(key: thread.kind.fieldLabelKey)"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/UITests/DynamicTypeRuntimeUITests.swift": mutateFile(runtimeTests, "member.frame.minY", "member.frame.maxY"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".frame(height: 56)", ".frame(height: 0)"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "if dynamicTypeSize == .accessibility5 {", "if dynamicTypeSize == .accessibility4 {"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, "if dynamicTypeSize == .accessibility5 {", "if dynamicTypeSize != .accessibility5 {"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/UITests/DynamicTypeRuntimeUITests.swift": mutateFile(runtimeTests, "thread.frame.minY", "thread.frame.maxY"),
    }), ax5Gate);
    expectsFailure(evaluate({
      "ios/UITests/DynamicTypeRuntimeUITests.swift": mutateFile(runtimeTests, "navigationBar.waitForExistence", "thread.waitForExistence"),
    }), ax5Gate);
  });
  it("rejects translucent or implicit-foreground status capsules", () => {
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const contrastGate = "contrast-stable adaptive backgrounds";
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        ".background(Color.opaqueConsoleCapsuleBackground, in: Capsule())",
        ".background(.thinMaterial, in: Capsule())",
      ),
    }), contrastGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        "Color(uiColor: .systemGray5)",
        "Color(uiColor: .tertiarySystemFill)",
      ),
    }), contrastGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `.font(.caption)
                    .foregroundStyle(.primary)
                    .padding(.horizontal, 8)`,
        `.font(.caption)
                    .padding(.horizontal, 8)`,
      ),
    }), contrastGate);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(
        fieldViews,
        `.background(Color.opaqueConsoleDetailBackground)
                .tint(.primary)`,
        `.background(Color.opaqueConsoleDetailBackground)`,
      ),
    }), contrastGate);
  });
  it("rejects a preflight that proves only an authenticated shell", () => {
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    expectsFailure(evaluate({
      "ios/UITests/PreflightUITests.swift": validFiles["ios/UITests/PreflightUITests.swift"].replace(
        "scrollToWorkOrderRow(in: restoredApp, id: detailWorkOrderID, timeout: 20) != nil",
        "restoredApp.buttons[AID.workOrderRow(detailWorkOrderID)].exists",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("dragStart.press(forDuration: 0.1, thenDragTo: dragEnd)", ""),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("let topSentinel = app.staticTexts[KO.locationConsentTitle]", "let topSentinel = app.staticTexts[KO.todayTitle]"),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "if workOrderRowActivationPoint(in: app, row: row, list: list) != nil {\n            return row\n        }",
        "if row.waitForExistence(timeout: 0.5), row.isHittable {\n            return row\n        }",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "if tabBar.exists {",
        "if false {",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "let tabChromeTop = tabBar.frame.minY - tabBar.frame.height",
        "let tabChromeTop = tabBar.frame.minY",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "guard viewport.contains(center) else { return nil }",
        "guard viewport.intersects(row.frame) else { return nil }",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5)",
        "withNormalizedOffset: CGVector(dx: 0.5, dy: 0.28)",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "timeout: TimeInterval = 60,\n    maxSwipes: Int = 48",
        "timeout: TimeInterval = 30,\n    maxSwipes: Int = 24",
      ),
    }), "decodes and renders the exact deterministic Today work order");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("back.isHittable", "detail.isHittable"),
    }), "actionable back control");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace(
        "activationPoint.tap()",
        "row.tap()",
      ),
    }), "actionable back control");
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "fixtureKey: String,\n        timeout: TimeInterval = 60",
        "fixtureKey: String,\n        timeout: TimeInterval = 30",
      ),
    }), "actionable back control");
  });
  it("rejects full-fixture Today traversal or tab-bar geometry drift", () => {
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    const criticalPath = validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"];
    const geometryGate = "traverse all five deterministic Today rows";
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "tabBar.frame.minY + 1", "list.frame.maxY + 1"),
    }), geometryGate);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(fieldCase, "tabBar.frame.minY - 1", "tabBar.frame.minY - tabBar.frame.height"),
    }), geometryGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(criticalPath, "UITestFixture.reportSuccessWorkOrderID", "UITestFixture.reportWorkOrderID"),
    }), geometryGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(criticalPath, "UITestFixture.adminRejectWorkOrderID", "UITestFixture.adminApproveWorkOrderID"),
    }), geometryGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(criticalPath, "workOrderRowActivationPoint(in: app, row: row, list: list)", "row.isHittable"),
    }), geometryGate);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": `${fieldCase}\nprint("TODAY_DIAGNOSTIC")`,
    }), geometryGate);
  });
  it("rejects lazy detail scrolling that can time out early or target the wrong surface", () => {
    const lazyScroll = "deadline-bounded exact-element scroll";
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    const fieldViews = validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"];
    const auditTests = validFiles["ios/UITests/AccessibilityAuditUITests.swift"];
    const messengerTests = validFiles["ios/UITests/MessengerUITests.swift"];
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replaceAll("let deadline = Date().addingTimeInterval(timeout)", "let deadline = Date()"),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("guard container.waitForExistence", "guard element.waitForExistence"),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace("container.swipeDown()", "container.swipeUp()"),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replaceAll("dragStart.press(forDuration: 0.1, thenDragTo: dragEnd)", "container.swipeUp()"),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace(
        "let origin = container.coordinate(withNormalizedOffset: .zero)",
        "let origin = container.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace(
        "CGVector(dx: trailingGutterX, dy: container.frame.height * 0.50)",
        "CGVector(dx: container.frame.width * 0.5, dy: container.frame.height * 0.50)",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace(
        "let trailingGutterX = max(container.frame.width * 0.9, 8)",
        "let trailingGutterX = 8",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace(
        "let trailingGutterX = max(container.frame.width * 0.9, 8)",
        "let trailingGutterX = 8",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/AccessibilityAuditUITests.swift": mutateFile(
        auditTests,
        "let trailingGutterX = max(container.frame.width - 8, 8)",
        "let trailingGutterX = 8",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/MessengerUITests.swift": mutateFile(
        messengerTests,
        "let trailingGutterX = max(list.frame.width * 0.9, 8)",
        "let trailingGutterX = max(list.frame.width - 8, 8)",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "if element.exists, element.isHittable {\n            return element\n        }",
        "if element.waitForExistence(timeout: 0.5), element.isHittable {\n            return element\n        }",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": mutateFile(fieldViews, ".scrollDismissesKeyboard(.immediately)", ""),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "in: app.descendants(matching: .any)[AID.detailView]",
        "in: app.collectionViews[AID.detailView]",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": fieldCase.replace(
        "topSentinel: app.staticTexts[KO.locationConsentTitle]",
        "topSentinel: app.buttons[AID.detailBackButton]",
      ),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"].replace("scrollToDetailElement(app.buttons[AID.detailStartWorkButton])", "app.buttons[AID.detailStartWorkButton]"),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"].replace("scrollToDetailElement(app.buttons[AID.detailSubmitReportButton])", "app.buttons[AID.detailSubmitReportButton]"),
    }), lazyScroll);
    expectsFailure(evaluate({
      "ios/UITests/CameraCaptureUITests.swift": validFiles["ios/UITests/CameraCaptureUITests.swift"].replace("scrollToDetailElement(app.buttons[AID.detailCaptureEvidenceButton])", "app.buttons[AID.detailCaptureEvidenceButton]"),
    }), lazyScroll);
  });
  it("rejects report feedback placed after the unrelated camera controls", () => {
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/ConsoleViews.swift": moveReportFeedbackAfterCamera(validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"]),
    }), "live report response feedback adjacent to submit-report controls");
  });
  it("rejects messenger rows that share a cross-section message identifier", () => {
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"].replace("messengerSearchResultRow", "messengerMessageRow") }), "section-scoped dynamic accessibility IDs");
    expectsFailure(evaluate({ "ios/Sources/ConsoleApp/ConsoleViews.swift": validFiles["ios/Sources/ConsoleApp/ConsoleViews.swift"].replace("messengerMessageRow", "messengerSearchResultRow") }), "section-scoped dynamic accessibility IDs");
  });
  it("rejects camera authorization state that cannot refresh after returning from Settings", () => {
    expectsFailure(evaluate({
      "ios/Sources/ConsoleApp/CameraCaptureView.swift": mutateFile(
        validFiles["ios/Sources/ConsoleApp/CameraCaptureView.swift"],
        "@Environment(\\.scenePhase) private var scenePhase",
        "",
      ),
    }), "refresh authorization when the app becomes active");
  });

  it("rejects local-state-only critical-path evidence", () => {
    expectsFailure(evaluate({ "ios/UITests/ConsoleCriticalPathUITests.swift": validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"].replace("AID.detailStatus", "KO.inProgress") }), "scoped mutations");
    expectsFailure(evaluate({ "ios/UITests/ConsoleCriticalPathUITests.swift": validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"].replaceAll("app.terminate()", "") }), "scoped mutations");
    expectsFailure(evaluate({ "ios/UITests/MessengerUITests.swift": validFiles["ios/UITests/MessengerUITests.swift"].replace("app.terminate()", "") }), "scoped mutations");
    expectsFailure(evaluate({ "ios/UITests/CameraCaptureUITests.swift": "if previewIsUsable { return }\ncancel.tap()" }), "scoped mutations");
    expectsFailure(evaluate({
      "ios/UITests/CameraCaptureUITests.swift": validFiles["ios/UITests/CameraCaptureUITests.swift"].replace(
        "XCTAssertTrue(\n            cancel.waitForNonExistence(timeout: 5),",
        "XCTAssertFalse(\n            cancel.waitForExistence(timeout: 5),",
      ),
    }), "scoped mutations");
    expectsFailure(evaluate({ "ios/UITests/LoginValidationUITests.swift": "XCTAssertTrue(loginError.exists)" }), "scoped mutations");
  });
  it("rejects globally or copy-scoped report terminal evidence", () => {
    const criticalPath = validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"];
    const reportEvidenceGate = "detail-owned accessibility identifiers";
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "detail.descendants(matching: .any)[AID.detailMessage]",
        "detail.staticTexts[KO.reportSuccessMessage]",
      ),
    }), reportEvidenceGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "detail.descendants(matching: .any)[AID.detailStatus]",
        "detail.staticTexts[KO.reportSubmitted]",
      ),
    }), reportEvidenceGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "detailMessage,\n            in: detail,",
        "detailMessage,\n            in: app,",
      ),
    }), reportEvidenceGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "topSentinel: detail.buttons[AID.detailSubmitReportButton],",
        "topSentinel: detail.staticTexts[KO.locationConsentTitle],",
      ),
    }), reportEvidenceGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "resolvedMessage.label,\n            KO.reportSuccessMessage,",
        "resolvedMessage.exists,\n            true,",
      ),
    }), reportEvidenceGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "resolvedStatus.label,\n            KO.reportSubmitted,",
        "resolvedStatus.exists,\n            true,",
      ),
    }), reportEvidenceGate);
  });
  it("rejects cached or globally scoped location-consent elements across SwiftUI state replacement", () => {
    const fieldCase = validFiles["ios/UITests/Support/ConsoleUITestCase.swift"];
    const criticalPath = validFiles["ios/UITests/ConsoleCriticalPathUITests.swift"];
    const freshQueryGate = "reacquire dynamic SwiftUI elements";
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "while Date() < deadline {\n            let detail = app.descendants(matching: .any)[AID.detailView]",
        "while Date() < deadline {\n            let detail = app",
      ),
    }), freshQueryGate);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "let element = detail.descendants(matching: .any)[identifier]",
        "let element = cachedElement",
      ),
    }), freshQueryGate);
    expectsFailure(evaluate({
      "ios/UITests/Support/ConsoleUITestCase.swift": mutateFile(
        fieldCase,
        "return app.descendants(matching: .any)[AID.detailView].buttons[identifier]",
        "return app.buttons[identifier]",
      ),
    }), freshQueryGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "detailButton(AID.locationConsentGrantButton).tap()",
        "let grant = detailButton(AID.locationConsentGrantButton)\n        grant.tap()",
      ),
    }), freshQueryGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "detailButton(AID.locationConsentGrantButton).tap()",
        "app.buttons[AID.locationConsentGrantButton].tap()",
      ),
    }), freshQueryGate);
    expectsFailure(evaluate({
      "ios/UITests/ConsoleCriticalPathUITests.swift": mutateFile(
        criticalPath,
        "waitForLabel(AID.locationConsentCollectionValue, containing: KO.yes)",
        "waitForLabel(cachedCollectionValue, containing: KO.yes)",
      ),
    }), freshQueryGate);
  });
});
