import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";

import { evaluateCiPreflight } from "./check-ci-preflight.mjs";

const workflow = readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
const postgresWrapperBuildFile = readFileSync(new URL("../tools/buck/BUCK", import.meta.url), "utf8");
const cargoLockGate = "cargo metadata --manifest-path backend/Cargo.toml --locked --format-version=1 >/dev/null";
const ciPreflightTests = "node --test scripts/check-ci-preflight.test.mjs";
const reachabilityPreflightCommands = [
  "node --test scripts/console/route-inventory.test.mjs",
  "tools/buck/run_test_with_postgres_env.test.sh",
  "tools/buck/test_needs_postgres.test.sh",
];
const preflightRustToolchainSetup = `      - name: Install Rust toolchain for Cargo.lock consistency
        uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8 # stable
        with:
          toolchain: "1.97.1"

`;

function expectFailure(source, message, buckBuildFile = postgresWrapperBuildFile) {
  const { failures } = evaluateCiPreflight(source, buckBuildFile);
  assert.ok(failures.some((failure) => failure.includes(message)), failures.join("\n"));
}

describe("CI preflight contract", () => {
  it("accepts the workflow's cheap preflight and protected expensive jobs", () => {
    assert.deepEqual(evaluateCiPreflight(workflow).failures, []);
  });

  it("rejects CI path filters that omit toolchain changes", () => {
    expectFailure(
      workflow.replace('      - "toolchains/**"\n', ""),
      "push must include toolchains/** in CI path filters",
    );
    const pullRequest = workflow.indexOf("  pull_request:\n");
    const pullWithoutToolchains = workflow.slice(0, pullRequest) + workflow.slice(pullRequest).replace(
      '      - "toolchains/**"\n',
      "",
    );
    expectFailure(pullWithoutToolchains, "pull_request must include toolchains/** in CI path filters");
  });

  it("rejects docs/program path removal independently for push and pull_request", () => {
    expectFailure(workflow.replace('      - "docs/program/**"\n', ""), "push must include docs/program/** in CI path filters");
    const pullRequest = workflow.indexOf("  pull_request:\n");
    const withoutPullDocs = workflow.slice(0, pullRequest) + workflow.slice(pullRequest).replace('      - "docs/program/**"\n', "");
    expectFailure(withoutPullDocs, "pull_request must include docs/program/** in CI path filters");
  });

  it("rejects toolchain entries placed outside each trigger's paths mapping", () => {
    const pushWithoutToolchains = workflow.replace('      - "toolchains/**"\n', "");
    expectFailure(
      pushWithoutToolchains.replace(
        "  pull_request:\n",
        "    paths-ignore:\n      - \"toolchains/**\"\n  pull_request:\n",
      ),
      "push must include toolchains/** in CI path filters",
    );
    expectFailure(
      pushWithoutToolchains.replace(
        "    paths:\n",
        "    branches-ignore:\n      - \"toolchains/**\"\n    paths:\n",
      ),
      "push must include toolchains/** in CI path filters",
    );

    const pullRequest = workflow.indexOf("  pull_request:\n");
    const pullWithoutToolchains = workflow.slice(0, pullRequest) + workflow.slice(pullRequest).replace(
      '      - "toolchains/**"\n',
      "",
    );
    expectFailure(
      pullWithoutToolchains.replace(
        "  workflow_dispatch:\n",
        "    paths-ignore:\n      - \"toolchains/**\"\n  workflow_dispatch:\n",
      ),
      "pull_request must include toolchains/** in CI path filters",
    );
    expectFailure(
      pullWithoutToolchains.replace(
        "  workflow_dispatch:\n",
        "    branches-ignore:\n      - \"toolchains/**\"\n  workflow_dispatch:\n",
      ),
      "pull_request must include toolchains/** in CI path filters",
    );
  });

  it("rejects Buck2 jobs that do not bootstrap pinned DotSlash before invocation", () => {
    expectFailure(
      workflow.replace(
        "      - name: Install pinned DotSlash runtime\n        run: tools/buck/install_dotslash.sh\n",
        "",
      ),
      "preflight must install pinned DotSlash before Buck2",
    );
    const apiContract = workflow.indexOf("  api-contract:\n");
    const apiWithoutDotSlash = workflow.slice(0, apiContract) + workflow.slice(apiContract).replace(
      "      - name: Install pinned DotSlash runtime\n        run: tools/buck/install_dotslash.sh\n",
      "",
    );
    expectFailure(apiWithoutDotSlash, "api-contract must install pinned DotSlash before Buck2");
  });

  it("resolves the backend DotSlash bootstrap from its effective working directory", () => {
    expectFailure(
      workflow.replace(
        "        run: ../tools/buck/install_dotslash.sh",
        "        run: tools/buck/install_dotslash.sh",
      ),
      "backend must install pinned DotSlash from ../tools/buck/install_dotslash.sh",
    );
  });

  it("requires backend Buck2 commands to run from the repository root", () => {
    for (const stepName of [
      "Buck2 dev-auth feature PostgreSQL suites",
      "Buck2 mnt-app unit suite",
      "Buck2 mnt-app inline PostgreSQL suites",
    ]) {
      expectFailure(
        workflow.replace(
          `      - name: ${stepName}\n        working-directory: .\n`,
          `      - name: ${stepName}\n`,
        ),
        "backend must preserve the locked fail-fast step multiset and failure semantics",
      );
    }
  });

  it("requires dev-up smoke to install pinned DotSlash before its indirect Buck2 build", () => {
    const devUp = workflow.indexOf("  dev-up-smoke:\n");
    const installStep =
      "      - name: Install pinned DotSlash runtime\n        run: tools/buck/install_dotslash.sh\n\n";
    const devUpWorkflow = workflow.slice(devUp);
    const withoutDotSlash = workflow.slice(0, devUp) + devUpWorkflow.replace(installStep, "");
    expectFailure(
      withoutDotSlash,
      "dev-up-smoke must install pinned DotSlash from tools/buck/install_dotslash.sh",
    );

    const firstBootstrap = "        run: node scripts/dev-up.mjs bootstrap\n";
    const afterFirstBootstrap = devUpWorkflow
      .replace(installStep, "")
      .replace(firstBootstrap, `${firstBootstrap}\n${installStep}`);
    expectFailure(
      workflow.slice(0, devUp) + afterFirstBootstrap,
      "dev-up-smoke must install pinned DotSlash before its first Buck invocation",
    );
  });

  it("rejects API contract tests that point MNT_APP_BIN at a Cargo target", () => {
    for (const path of [
      "${{ github.workspace }}/backend/target/debug/mnt-app",
      "${CARGO_TARGET_DIR}/debug/mnt-app",
    ]) {
      expectFailure(
        workflow.replace(
          "      CONTRACT_DATABASE_URL: postgres://postgres:postgres@localhost:5432/mnt_contract\n",
          `      CONTRACT_DATABASE_URL: postgres://postgres:postgres@localhost:5432/mnt_contract\n      MNT_APP_BIN: ${path}\n`,
        ),
        "api-contract must not use a Cargo target path for MNT_APP_BIN",
      );
    }
  });

  it("rejects duplicate API contract app producers and later binary overrides", () => {
    expectFailure(
      workflow.replace(
        "      - name: Capture Buck2-built app for contract test\n",
        "      - name: Duplicate OpenAPI app-served drift gate\n        run: npm run check:openapi-app\n\n      - name: Capture Buck2-built app for contract test\n",
      ),
      "api-contract must run exactly one npm run check:openapi-app producer",
    );
    expectFailure(
      workflow.replace(
        "      - name: Capture Buck2-built app for contract test\n",
        "      - name: Duplicate direct Buck2 app build\n        run: tools/buck2 build //backend/app:mnt-app\n\n      - name: Capture Buck2-built app for contract test\n",
      ),
      "api-contract must contain only the approved ordered steps",
    );
    expectFailure(
      workflow.replace(
        "      - name: Employee import replay contract\n",
        "      - name: Override contract app\n        run: echo \"MNT_APP_BIN=${CARGO_TARGET_DIR}/debug/mnt-app\" >> \"$GITHUB_ENV\"\n\n      - name: Employee import replay contract\n",
      ),
      "api-contract may reference GITHUB_ENV only in the designated capture step",
    );
    expectFailure(
      workflow.replace(
        '          printf \'MNT_APP_BIN=%s\\n\' "${mnt_app_bin}" >> "${GITHUB_ENV}"\n',
        '          printf \'MNT_APP_BIN=%s\\n\' "${mnt_app_bin}" >> "${GITHUB_ENV}"\n          export MNT_APP_BIN=/tmp/other-mnt-app\n',
      ),
      "api-contract capture must use the designated verified command grammar",
    );
    expectFailure(
      workflow.replace(
        "      - name: Employee import replay contract\n",
        "      - name: Employee import replay contract\n        env:\n          MNT_APP_BIN: /tmp/other-mnt-app\n",
      ),
      "api-contract must not override the captured MNT_APP_BIN",
    );
  });

  it("accepts the designated verified Buck2 app handoff", () => {
    assert.match(workflow, /# check:openapi-app is the sole Buck2 producer for this handoff\./);
    assert.deepEqual(evaluateCiPreflight(workflow).failures, []);
  });

  it("rejects shell-spelling bypasses for API contract producers and environment writes", () => {
    expectFailure(
      workflow.replace(
        "      - name: Capture Buck2-built app for contract test\n",
        "      - name: Duplicate OpenAPI producer\n        run: |\n          # This still produces the Buck app.\n          CI=1 npm \\\n            run check:openapi-app; :\n\n      - name: Capture Buck2-built app for contract test\n",
      ),
      "api-contract must contain only the approved ordered steps",
    );
    expectFailure(
      workflow.replace(
        "      - name: Capture Buck2-built app for contract test\n",
        "      - name: Duplicate direct Buck2 app build\n        run: |\n          command ./tools/buck2 --isolation-dir .tmp \\\n            build --out .tmp/duplicate //backend/app:mnt-app # direct producer\n\n      - name: Capture Buck2-built app for contract test\n",
      ),
      "api-contract must contain only the approved ordered steps",
    );
    expectFailure(
      workflow.replace(
        "      - name: Employee import replay contract\n",
        "      - name: Late redirected override\n        run: |\n          echo \"MNT_APP_BIN=/tmp/other-mnt-app\" >> \"$GITHUB_ENV\" # still a write\n          :\n\n      - name: Employee import replay contract\n",
      ),
      "api-contract may reference GITHUB_ENV only in the designated capture step",
    );
    expectFailure(
      workflow.replace(
        "      - name: Employee import replay contract\n",
        "      - name: Late tee override\n        run: printf 'MNT_APP_BIN=/tmp/other-mnt-app\\n' | tee -a \"$GITHUB_ENV\"\n\n      - name: Employee import replay contract\n",
      ),
      "api-contract may reference GITHUB_ENV only in the designated capture step",
    );
  });

  it("fails closed on indirect producers and every non-capture GITHUB_ENV surface", () => {
    for (const command of [
      'bash -c "npm run check:openapi-app"',
      'sh -c "npm run check:openapi-app"',
      'zsh -c "npm run check:openapi-app"',
      'eval "npm run check:openapi-app"',
      '"$OPENAPI_PRODUCER"',
      'command "$OPENAPI_PRODUCER"',
      "node scripts/check-openapi-app.mjs",
    ]) {
      expectFailure(
        workflow.replace(
          "      - name: Capture Buck2-built app for contract test\n",
          `      - name: Indirect OpenAPI producer\n        run: ${command}\n\n      - name: Capture Buck2-built app for contract test\n`,
        ),
        "api-contract must contain only the approved ordered steps",
      );
    }

    expectFailure(
      workflow.replace(
        "      - name: Employee import replay contract\n",
        "      - name: Programmatic environment override\n        run: node -e 'require(\"node:fs\").appendFileSync(process.env.GITHUB_ENV, \"MNT_APP_BIN=/tmp/other\\n\")'\n\n      - name: Employee import replay contract\n",
      ),
      "api-contract may reference GITHUB_ENV only in the designated capture step",
    );
    expectFailure(
      workflow.replace(
        '          printf \'MNT_APP_BIN=%s\\n\' "${mnt_app_bin}" >> "${GITHUB_ENV}"',
        '          printf \'MNT_APP_BIN=%s\\n\' "${mnt_app_bin}" > "${GITHUB_ENV:?}"',
      ),
      "api-contract capture must use the designated verified command grammar",
    );
  });

  it("allows only the ordered API contract execution surface", () => {
    for (const command of [
      "$(printf ./tools/buck2) build //backend/app:mnt-app",
      "node ./scripts/check-openapi-app.mjs",
      "node --enable-source-maps scripts/check-openapi-app.mjs",
      "cargo build -p mnt-app",
      'env_name=GITHUB_$(printf ENV); key=MNT_APP_$(printf BIN); printf "$key=/tmp/other\\n" >> "${!env_name}"',
    ]) {
      expectFailure(
        workflow.replace(
          "      - name: Capture Buck2-built app for contract test\n",
          `      - name: Unexpected executable surface\n        run: ${command}\n\n      - name: Capture Buck2-built app for contract test\n`,
        ),
        "api-contract must contain only the approved ordered steps",
      );
    }
  });

  it("requires backend DotSlash bootstrap before any Buck or DotSlash invocation", () => {
    for (const command of ["tools/buck2 --version", "dotslash run //backend/app:mnt-app"]) {
      expectFailure(
        workflow.replace(
          "      - name: Install pinned DotSlash runtime\n        run: ../tools/buck/install_dotslash.sh\n",
          `      - name: First Buck invocation\n        run: ${command}\n\n      - name: Install pinned DotSlash runtime\n        run: ../tools/buck/install_dotslash.sh\n`,
        ),
        "backend must install pinned DotSlash before its first Buck invocation",
      );
    }
  });

  it("rejects a generated-face authority job without the complete closure", () => {
    expectFailure(
      workflow.replace(
        "tools/buck/preflight.sh --full-generated-faces",
        "tools/buck/preflight.sh --unexpected",
      ),
      "generated-face-authority must run the complete generated-face closure",
    );
  });

  it("requires the lock-sourced Reindeer toolchain before the full generated-face closure", () => {
    const toolchainSetup = `      - name: Install lock-pinned Reindeer Rust toolchain
        shell: bash
        run: |
          set -euo pipefail
          # shellcheck source=third-party/rust/reindeer/upstream.lock
          source third-party/rust/reindeer/upstream.lock
          rustup toolchain install "$REINDEER_TOOLCHAIN" --profile minimal

`;
    const fullGate = `      - name: Full generated-face closure
        run: tools/buck/preflight.sh --full-generated-faces
`;

    expectFailure(
      workflow.replace(toolchainSetup, ""),
      "must install the lock-pinned Reindeer Rust toolchain before full generated-face closure",
    );
    expectFailure(
      workflow.replace(
        "source third-party/rust/reindeer/upstream.lock",
        "REINDEER_TOOLCHAIN=hardcoded-not-lock-sourced",
      ),
      "must source third-party/rust/reindeer/upstream.lock",
    );
    expectFailure(
      workflow.replace(toolchainSetup, "").replace(fullGate, `${fullGate}${toolchainSetup}`),
      "must install the lock-pinned Reindeer Rust toolchain before full generated-face closure",
    );
    expectFailure(
      workflow.replace(
        "          set -euo pipefail\n          # shellcheck source=third-party/rust/reindeer/upstream.lock\n          source third-party/rust/reindeer/upstream.lock",
        "          source third-party/rust/reindeer/upstream.lock\n          set -euo pipefail",
      ),
      "must enable strict shell mode before sourcing third-party/rust/reindeer/upstream.lock",
    );
    expectFailure(
      workflow.replace(
        "          source third-party/rust/reindeer/upstream.lock\n          rustup toolchain install \"$REINDEER_TOOLCHAIN\" --profile minimal",
        "          rustup toolchain install \"$REINDEER_TOOLCHAIN\" --profile minimal\n          source third-party/rust/reindeer/upstream.lock",
      ),
      "must source third-party/rust/reindeer/upstream.lock before installing the Reindeer Rust toolchain",
    );
    expectFailure(
      workflow.replace(
        "          rustup toolchain install \"$REINDEER_TOOLCHAIN\" --profile minimal",
        "          export REINDEER_TOOLCHAIN=untrusted\n          rustup toolchain install \"$REINDEER_TOOLCHAIN\" --profile minimal",
      ),
      "must not override REINDEER_TOOLCHAIN after sourcing third-party/rust/reindeer/upstream.lock",
    );
  });

  it("requires the pinned Rust toolchain before Cargo-dependent preflight tests", () => {
    expectFailure(
      workflow.replace(preflightRustToolchainSetup, "").replace(
        `      - name: CI preflight contract tests\n        run: ${ciPreflightTests}`,
        `      - name: CI preflight contract tests
        run: ${ciPreflightTests}

${preflightRustToolchainSetup.trimEnd()}`,
      ),
      `preflight must install the pinned Rust toolchain before ${ciPreflightTests}`,
    );
  });

  it("requires a full-history checkout for merge-tip console validation", () => {
    expectFailure(
      workflow.replace("          fetch-depth: 0\n", ""),
      "preflight checkout must fetch full history with fetch-depth: 0",
    );
  });

  it("requires explicit exact-M C/T derivation before every normal-PR console admission", () => {
    expectFailure(
      workflow.replace('          CONSOLE_AUTHORITY_TIP_SHA="$(git rev-parse "$CONSOLE_SYNTHETIC_MERGE_SHA^2")"\n', ''),
      "derive exact C/T/M",
    );
    expectFailure(
      workflow.replace('          CONSOLE_CANDIDATE_SHA="$(git rev-parse "$CONSOLE_AUTHORITY_TIP_SHA^")"\n', '          CONSOLE_CANDIDATE_SHA="$GITHUB_SHA"\n'),
      "derive exact C/T/M",
    );
    expectFailure(
      workflow.replace('        if: ${{ github.event_name == \'pull_request\' }}\n        run: npm run check:console-truth-ledger', '        run: npm run check:console-truth-ledger'),
      "exact C/T/M derivation",
    );
    expectFailure(
      workflow.replace('          CONSOLE_SYNTHETIC_MERGE_SHA="$(git rev-parse "$GITHUB_SHA^{commit}")"\n', ''),
      "derive exact C/T/M",
    );
    expectFailure(
      workflow.replace('          CONSOLE_CANDIDATE_SHA="$(git rev-parse "$CONSOLE_AUTHORITY_TIP_SHA^")"\n', '          test "$(git rev-parse HEAD)" = "$CONSOLE_SYNTHETIC_MERGE_SHA"\n          CONSOLE_CANDIDATE_SHA="$(git rev-parse "$CONSOLE_AUTHORITY_TIP_SHA^")"\n'),
      "derive exact C/T/M",
    );
    expectFailure(
      workflow.replace('        run: node --test scripts/console/validate-console-truth-ledger.test.mjs', '        if: ${{ github.event_name == \'pull_request\' }}\n        run: node --test scripts/console/validate-console-truth-ledger.test.mjs'),
      "validate-console-truth-ledger.test.mjs",
    );
    expectFailure(
      workflow.replace('        run: node --test scripts/console/plan-fanout.test.mjs', '        if: ${{ github.event_name == \'pull_request\' }}\n        run: node --test scripts/console/plan-fanout.test.mjs'),
      "plan-fanout.test.mjs",
    );
    expectFailure(
      workflow.replace('      - name: Console authority-train regression\n        run: node --test scripts/console/verify-console-authority-train.test.mjs\n\n', ''),
      "verify-console-authority-train.test.mjs",
    );
    expectFailure(
      workflow.replace('        if: ${{ github.event_name == \'pull_request\' }}\n        run: node scripts/console/plan-fanout.mjs', '        run: node scripts/console/plan-fanout.mjs'),
      "plan-fanout.mjs",
    );
    expectFailure(
      workflow.replace('        run: npm run check:console-truth-ledger', '        run: CONSOLE_INTEGRATION_TIP_SHA="$CONSOLE_AUTHORITY_TIP_SHA" npm run check:console-truth-ledger'),
      "CONSOLE_INTEGRATION_TIP_SHA",
    );
    expectFailure(
      workflow.replace('        run: node scripts/console/plan-fanout.mjs --candidate "$CONSOLE_CANDIDATE_SHA" --authority-tip "$CONSOLE_AUTHORITY_TIP_SHA" --synthetic-merge "$CONSOLE_SYNTHETIC_MERGE_SHA"', '        run: node scripts/console/plan-fanout.mjs'),
      "plan-fanout.mjs",
    );
  });

  it("rejects omission and comment-only reachability regressions", () => {
    for (const command of reachabilityPreflightCommands) {
      expectFailure(workflow.replace(`        run: ${command}\n`, ""), command);
      expectFailure(workflow.replace(`        run: ${command}\n`, `        # ${command}\n`), command);
    }
  });

  it("rejects conditional and continue-on-error reachability regressions", () => {
    for (const command of reachabilityPreflightCommands) {
      expectFailure(
        workflow.replace(`        run: ${command}\n`, `        if: \${{ false }}\n        run: ${command}\n`),
        "unconditionally",
      );
      expectFailure(
        workflow.replace(`        run: ${command}\n`, `        continue-on-error: true\n        run: ${command}\n`),
        "unconditionally",
      );
    }
  });

  it("rejects a preflight that does not run npm and Cargo lock consistency gates", () => {
    expectFailure(workflow.replace("npm run check:package-lock", "npm run check:root-workspaces"), "check:package-lock");
    expectFailure(
      workflow.replace(
        cargoLockGate,
        "cargo metadata --manifest-path backend/Cargo.toml --format-version=1 >/dev/null",
      ),
      cargoLockGate,
    );
  });

  it("rejects a dependency missing from Cargo.lock while the clean lock passes", () => {
    const root = mkdtempSync(join(tmpdir(), "maintenance-cargo-lock-"));
    const app = join(root, "app");
    const dependency = join(root, "dependency");
    const extra = join(root, "extra");
    try {
      for (const directory of [app, dependency, extra]) {
        mkdirSync(join(directory, "src"), { recursive: true });
      }
      writeFileSync(join(root, "Cargo.toml"), "[workspace]\nmembers = [\"app\", \"dependency\"]\nresolver = \"2\"\n");
      for (const [directory, name] of [[app, "fixture-app"], [dependency, "fixture-dependency"]]) {
        writeFileSync(join(directory, "Cargo.toml"), `[package]\nname = \"${name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n`);
        writeFileSync(join(directory, "src/lib.rs"), "pub fn fixture() {}\n");
      }
      writeFileSync(join(app, "Cargo.toml"), "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nfixture-dependency = { path = \"../dependency\" }\n");
      assert.equal(spawnSync("cargo", ["generate-lockfile"], { cwd: root }).status, 0);
      assert.equal(spawnSync("cargo", ["metadata", "--manifest-path", join(app, "Cargo.toml"), "--locked", "--format-version=1"], { cwd: root }).status, 0);

      writeFileSync(join(extra, "Cargo.toml"), "[package]\nname = \"fixture-extra\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
      writeFileSync(join(extra, "src/lib.rs"), "pub fn extra() {}\n");
      writeFileSync(join(app, "Cargo.toml"), "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nfixture-dependency = { path = \"../dependency\" }\nfixture-extra = { path = \"../extra\" }\n");
      assert.equal(spawnSync("cargo", ["metadata", "--manifest-path", join(app, "Cargo.toml"), "--locked", "--no-deps", "--format-version=1"], { cwd: root }).status, 0);
      assert.notEqual(spawnSync("cargo", ["metadata", "--manifest-path", join(app, "Cargo.toml"), "--locked", "--format-version=1"], { cwd: root }).status, 0);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects a preflight command that appears only in a comment", () => {
    expectFailure(
      workflow.replace(
        "      - name: Canonical npm lockfile\n        run: npm run check:package-lock",
        "      - name: Canonical npm lockfile\n        # npm run check:package-lock",
      ),
      "check:package-lock",
    );
  });

  it("rejects a required preflight step guarded by a condition", () => {
    expectFailure(
      workflow.replace(
        "      - name: Canonical npm lockfile\n        run: npm run check:package-lock",
        "      - name: Canonical npm lockfile\n        if: ${{ false }}\n        run: npm run check:package-lock",
      ),
      "unconditionally",
    );
  });

  it("rejects a required preflight step allowed to continue on error", () => {
    expectFailure(
      workflow.replace(
        "      - name: Canonical npm lockfile\n        run: npm run check:package-lock",
        "      - name: Canonical npm lockfile\n        continue-on-error: true\n        run: npm run check:package-lock",
      ),
      "unconditionally",
    );
  });

  it("rejects any expensive backend, mobile, or browser job without the preflight dependency", () => {
    expectFailure(workflow.replace("  backend:\n", "  backend:\n    needs: []\n"), "backend must need preflight");
    expectFailure(workflow.replace("    needs: [preflight, mobile-parity]\n", "    needs: []\n"), "android-app must need preflight");
    expectFailure(workflow.replace("  browser-e2e:\n", "  browser-e2e:\n    needs: []\n"), "browser-e2e must need preflight");
    expectFailure(workflow.replace("  kubernetes-manifests:\n", "  kubernetes-manifests:\n    needs: []\n"), "kubernetes-manifests must need preflight");
  });

  it("rejects failure-insensitive job-level conditions on protected jobs", () => {
    expectFailure(workflow.replace("  backend:\n", "  backend:\n    if: always()\n"), "backend must not define job-level if");
    expectFailure(workflow.replace("  browser-e2e:\n", "  browser-e2e:\n    if: ${{ !cancelled() }}\n"), "browser-e2e must not define job-level if");
  });

  it("rejects job-level preflight failure bypasses", () => {
    expectFailure(
      workflow.replace("  preflight:\n", "  preflight:\n    if: always()\n"),
      "preflight must not define job-level if",
    );
    expectFailure(
      workflow.replace("  preflight:\n", "  preflight:\n    continue-on-error: true\n"),
      "preflight must not define job-level continue-on-error",
    );
  });

  it("locks post-preflight Buck2 reachability targets and disallows added run surfaces", () => {
    expectFailure(
      workflow.replace(
        "tools/buck2 test //backend/crates/support/domain:mnt-support-domain-unit",
        "cargo test -p mnt-support-domain",
      ),
      "support-domain-unit must run tools/buck2 test //backend/crates/support/domain:mnt-support-domain-unit",
    );
    expectFailure(
      workflow.replace(
        "tools/buck/test_needs_postgres.sh --num-threads=1",
        "tools/buck/test_needs_postgres.sh",
      ),
      "postgres-domain-reachability must run the locked PostgreSQL reachability targets",
    );
    expectFailure(
      workflow.replace(
        "//tools/buck:attendance-concurrency-postgres",
        "//backend/crates/attendance/adapter-postgres:mnt-attendance-adapter-postgres-itest-concurrency",
      ),
      "postgres-domain-reachability must run the locked PostgreSQL reachability targets",
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper dispatch-p1-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'name = "dispatch-p1-postgres",\n    test = "run_test_with_postgres_env.sh",',
        'name = "dispatch-p1-postgres",\n    test = "unexpected_loader.sh",',
      ),
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper attendance-concurrency-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'args = ["$(location //backend/crates/attendance/adapter-postgres:mnt-attendance-adapter-postgres-itest-concurrency)"]',
        'args = ["$(location //backend/crates/attendance/adapter-postgres:mnt-attendance-adapter-postgres-itest-cancel_substitution)"]',
      ),
    );
    expectFailure(
      workflow.replace(
        "      - name: Support domain unit target\n",
        "      - name: Unexpected Cargo test\n        run: cargo test -p mnt-support-domain\n\n      - name: Support domain unit target\n",
      ),
      "support-domain-unit must contain only the locked ordered Buck2 run steps",
    );
    expectFailure(
      workflow.replace(
        "//tools/buck:app-inline-postgres",
        "//backend/app:mnt-app-itest-inline-postgres",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "//tools/buck:app-dev-auth-persona-guard-postgres",
        "//backend/app:mnt-app-itest-dev_auth_persona_guard_feature",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "//tools/buck:auth-rest-dev-auth-group-admin-postgres",
        "//backend/crates/platform/auth-rest:mnt-platform-auth-rest-itest-group_admin_tenant_context",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "//tools/buck:provisioning-dev-principal-upsert-race-postgres",
        "//backend/crates/platform/provisioning:mnt-platform-provisioning-itest-dev_principal_upsert_race",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper auth-rest-dev-auth-inline-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'name = "auth-rest-dev-auth-inline-postgres",\n    test = "run_test_with_postgres_env.sh",',
        'name = "auth-rest-dev-auth-inline-postgres",\n    test = "unexpected_loader.sh",',
      ),
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper provisioning-dev-principal-upsert-race-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'deps = ["//backend/crates/platform/provisioning:mnt-platform-provisioning-itest-dev_principal_upsert_race"],',
        'deps = ["//backend/crates/platform/auth-rest:mnt-platform-auth-rest-itest-dev_auth_session"],',
      ),
    );
    expectFailure(
      workflow.replace(
        "      - name: Buck2 dev-auth feature PostgreSQL suites\n",
        "      - name: Direct Cargo dev-auth suite\n        run: cargo test -p mnt-platform-auth-rest --features dev-auth\n\n      - name: Buck2 dev-auth feature PostgreSQL suites\n",
      ),
      "backend must not run direct Cargo PostgreSQL tests for mnt-platform-auth-rest",
    );
    expectFailure(
      workflow.replace(
        "      - name: Buck2 dev-auth feature PostgreSQL suites\n",
        "      - name: Direct Cargo provisioning race\n        run: cargo test -p mnt-platform-provisioning --test dev_principal_upsert_race\n\n      - name: Buck2 dev-auth feature PostgreSQL suites\n",
      ),
      "backend must not run direct Cargo PostgreSQL tests for mnt-platform-provisioning",
    );
    const cargo = ["car", "go"].join("");
    const test = ["te", "st"].join("");
    const backendMarker = "      - name: Buck2 dev-auth feature PostgreSQL suites\n";
    const insertBackendRun = (command) => workflow.replace(
      backendMarker,
      "      - name: Adversarial direct Cargo target\n        run: |\n          "
        + command
        + "\n\n"
        + backendMarker,
    );
    for (const packageName of ["mnt-platform-auth-rest", "mnt-platform-provisioning"]) {
      for (const runner of [cargo + " " + test, cargo + " nextest run"]) {
        for (const packageArgument of [
          "-p " + packageName,
          "-p=" + packageName,
          "--package " + packageName,
          "--package=" + packageName,
        ]) {
          expectFailure(
            insertBackendRun(runner + " " + packageArgument),
            "backend must not run direct Cargo PostgreSQL tests for " + packageName,
          );
          for (const prefix of [
            "command env SQLX_OFFLINE=true ",
            "command -- env -- ",
            "command -p env SQLX_OFFLINE=true ",
            "command -p -- env -- ",
            "env -i command -- ",
          ]) {
            expectFailure(
              insertBackendRun(prefix + runner + " " + packageArgument),
              "backend must not run direct Cargo PostgreSQL tests for " + packageName,
            );
          }
          expectFailure(
            insertBackendRun("env -S 'command env " + runner + " " + packageArgument + "'"),
            "backend must not run direct Cargo PostgreSQL tests for " + packageName,
          );
          expectFailure(
            insertBackendRun("env -S 'command -p env " + runner + " " + packageArgument + "'"),
            "backend must not run direct Cargo PostgreSQL tests for " + packageName,
          );
          expectFailure(
            insertBackendRun("env -S 'command -p -- env -- " + runner + " " + packageArgument + "'"),
            "backend must not run direct Cargo PostgreSQL tests for " + packageName,
          );
        }
      }
    }
    for (const [packageName, command] of [
      ["mnt-platform-provisioning", cargo + " \\\n          " + test + " \\\n          --package \\\n          mnt-platform-provisioning"],
      ["mnt-platform-auth-rest", "env SQLX_OFFLINE=true " + cargo + " nextest run \\\n          -p=mnt-platform-auth-rest"],
      ["mnt-platform-auth-rest", "env -u DATABASE_URL -- " + cargo + " nextest \\\n          run --package=mnt-platform-auth-rest"],
      ["mnt-platform-provisioning", "command " + cargo + " " + test + " --package mnt-platform-provisioning"],
    ]) {
      expectFailure(
        insertBackendRun(command),
        "backend must not run direct Cargo PostgreSQL tests for " + packageName,
      );
    }
    for (const command of [
      "# " + cargo + " " + test + " -p mnt-platform-auth-rest",
      cargo + " run -p mnt-platform-auth-rest",
      "echo " + cargo + " " + test + " -p mnt-platform-provisioning",
      "command -v " + cargo + " " + test + " -p mnt-platform-auth-rest",
      "command -V " + cargo + " " + test + " -p mnt-platform-provisioning",
    ]) {
      const failures = evaluateCiPreflight(insertBackendRun(command)).failures;
      assert.ok(
        !failures.some((failure) => failure.startsWith("backend must not run direct Cargo PostgreSQL tests")),
        failures.join("\n"),
      );
    }
    for (const command of ["command --", "env -S", "env -S 'command --'"]) {
      expectFailure(
        insertBackendRun(command),
        "backend must not contain a malformed executable shell surface",
      );
    }
    expectFailure(
      insertBackendRun(
        "command -p " + cargo + " " + test + " -p mnt-platform-auth-rest \\",
      ),
      "backend must not contain a malformed executable shell surface",
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper app-inline-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'name = "app-inline-postgres",\n    test = "run_test_with_postgres_env.sh",\n    args = ["$(location //backend/app:mnt-app-itest-inline-postgres)"],\n    deps = ["//backend/app:mnt-app-itest-inline-postgres"],\n    labels = ["test.integration", "resource.postgres", "needs-postgres"],',
        'name = "app-inline-postgres",\n    test = "run_test_with_postgres_env.sh",\n    args = ["$(location //backend/app:mnt-app-itest-inline-postgres)"],\n    deps = ["//backend/app:mnt-app-itest-inline-postgres"],\n    labels = ["owner.backend.app", "domain.app", "test.integration", "resource.postgres", "needs-postgres"],',
      ),
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper app-dev-auth-persona-guard-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'name = "app-dev-auth-persona-guard-postgres",\n    test = "run_test_with_postgres_env.sh",',
        'name = "app-dev-auth-persona-guard-postgres",\n    test = "run_test_with_postgres_env.sh",\n    args = ["$(location //backend/app:mnt-app-itest-inline-postgres)"],',
      ),
    );
    expectFailure(
      workflow,
      "tools/buck/BUCK must bind PostgreSQL wrapper app-dev-auth-persona-guard-postgres to the loader and exact Rust binary",
      postgresWrapperBuildFile.replace(
        'deps = ["//backend/app:mnt-app-itest-dev_auth_persona_guard_feature"],',
        'deps = ["//backend/app:mnt-app-itest-inline-postgres"],',
      ),
    );
  });

  it("preserves fail-fast backend and dev-up ordering", () => {
    const sourceGateDisplaced = workflow
      .replace("      - name: Layer-boundary gate\n", "      - name: Displaced source gate\n")
      .replace("      - name: Reconcile portable PostgreSQL role topology\n", "      - name: Layer-boundary gate\n");
    expectFailure(sourceGateDisplaced, "backend must run source-only gates immediately after clippy");

    const unitAfterPostgres = workflow
      .replace("      - name: Buck2 mnt-app unit suite\n", "      - name: Temporary Buck2 step\n")
      .replace("      - name: Buck2 mnt-app inline PostgreSQL suites\n", "      - name: Buck2 mnt-app unit suite\n");
    expectFailure(unitAfterPostgres, "backend must preserve the locked fail-fast step order");

    const devUpContractAfterDiskPurge = workflow
      .replace("      - name: dev-up compose contract unit test\n", "      - name: Temporary dev-up step\n")
      .replace("      - name: Free runner disk for Rust backend\n", "      - name: dev-up compose contract unit test\n");
    expectFailure(devUpContractAfterDiskPurge, "dev-up-smoke must preserve the locked fail-fast step order");
  });

  it("fails closed when optimized gates or targets are commented, weakened, or duplicated", () => {
    expectFailure(
      workflow.replace(
        "        run: cargo run -p mnt-gate-layer-boundary",
        "        # cargo run -p mnt-gate-layer-boundary",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "      - name: Audit-coverage gate\n",
        "      - name: Audit-coverage gate\n        if: ${{ false }}\n",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "      - name: Migration-safety gate\n",
        "      - name: Migration-safety gate\n        continue-on-error: true\n",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "      - name: Audit-coverage gate\n",
        "      - name: Layer-boundary gate\n        if: ${{ !cancelled() }}\n        run: cargo run -p mnt-gate-layer-boundary\n\n      - name: Audit-coverage gate\n",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "        run: env -u DATABASE_URL tools/buck2 test //backend/app:mnt-app-unit",
        "        # env -u DATABASE_URL tools/buck2 test //backend/app:mnt-app-unit",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    expectFailure(
      workflow.replace(
        "      - name: dev-up compose contract unit test\n        run: node --test scripts/dev-up-compose.test.mjs",
        "      - name: dev-up compose contract unit test\n        continue-on-error: true\n        run: node --test scripts/dev-up-compose.test.mjs",
      ),
      "dev-up-smoke must preserve the locked fail-fast step multiset and failure semantics",
    );
  });

  it("keeps protected backend steps fail-fast and runs PR 473 contract tests before topology", () => {
    expectFailure(
      workflow.replace(
        "      - name: rustfmt check\n",
        "      - name: rustfmt check\n        if: ${{ !cancelled() }}\n",
      ),
      "backend must not use !cancelled() on protected fail-fast steps",
    );
    expectFailure(
      workflow.replace(
        "        run: python3 scripts/check-pr473-migration-operational.test.py -v",
        "        # python3 scripts/check-pr473-migration-operational.test.py -v",
      ),
      "backend must preserve the locked fail-fast step multiset and failure semantics",
    );
    const pr473ContractAfterTopology = workflow
      .replace("      - name: PR 473 migration operational contract tests\n", "      - name: Deferred PR 473 contract tests\n")
      .replace("      - name: Reconcile portable PostgreSQL role topology\n", "      - name: PR 473 migration operational contract tests\n");
    expectFailure(pr473ContractAfterTopology, "backend must preserve the locked fail-fast step order");
  });
});
