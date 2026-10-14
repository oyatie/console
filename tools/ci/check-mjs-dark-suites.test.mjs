import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, mkdirSync, readFileSync, rmSync, existsSync, realpathSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { resolveDarkSuites } from "./check-mjs-dark-suites.mjs";
import { gitFixtureEnvironment } from "../../scripts/lib/git-fixture-environment.mjs";

test("resolveDarkSuites classifies wired vs dark", () => {
  const dir = mkdtempSync(join(tmpdir(), "dark-suite-"));
  mkdirSync(join(dir, "scripts"), { recursive: true });
  mkdirSync(join(dir, ".github/workflows"), { recursive: true });
  mkdirSync(join(dir, "tools/ci"), { recursive: true });

  writeFileSync(
    join(dir, "package.json"),
    JSON.stringify({
      scripts: {
        "test:wired": "node --test scripts/wired.test.mjs",
        "test:orphan": "node --test scripts/orphan.test.mjs",
        "test:ci-tools": "node --test tools/ci/local.test.mjs",
      },
    }),
  );
  writeFileSync(
    join(dir, ".github/workflows/ci.yml"),
    "jobs:\n  t:\n    steps:\n      - run: npm run test:wired\n",
  );
  writeFileSync(join(dir, "scripts/wired.test.mjs"), "import test from 'node:test';\n");
  writeFileSync(join(dir, "scripts/orphan.test.mjs"), "import test from 'node:test';\n");
  writeFileSync(join(dir, "scripts/nowhere.test.mjs"), "import test from 'node:test';\n");
  writeFileSync(join(dir, "tools/ci/local.test.mjs"), "import test from 'node:test';\n");

  const report = resolveDarkSuites(dir);
  assert.equal(report.class_id, "js-test-reachability.dark-suite");
  assert.ok(report.wired.includes("scripts/wired.test.mjs"));
  assert.ok(report.dark.some((d) => d.suite === "scripts/nowhere.test.mjs" && d.npm_script === null));
  assert.ok(report.dark.some((d) => d.suite === "scripts/orphan.test.mjs" && d.npm_script === "test:orphan"));
  // tools/ci suite with npm owner is still dark (not CI-invoked) but not orphan
  assert.ok(report.dark.some((d) => d.suite === "tools/ci/local.test.mjs" && d.npm_script === "test:ci-tools"));
  assert.equal(report.tools_ci_orphan.length, 0);
});

// TEST_ONLY finite executable fixtures: no compiler, product service or real HR run.
const repo = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const ordinary = "fixture/new.test.mjs";
const supervised = "scripts/check-native-hr-policy-parser-contract.test.mjs";
const shapes = [
  { name: "direct Node", run: `node --test ${ordinary}` },
  { name: "comment-only", run: `# node --test ${ordinary}\ntrue` },
  { name: "inert echo", run: `echo node --test ${ordinary}` },
  { name: "disabled step", run: `node --test ${ordinary}`, guard: "        if: false\n" },
  { name: "uninvoked sweep", run: "true", sweep: true },
  { name: "invoked sweep", run: "node tools/ci/gate-sweep.mjs", sweep: true },
  { name: "basename collision", run: "node --test first/shared.test.mjs", suites: ["first/shared.test.mjs", "second/shared.test.mjs"] },
  { name: "supervised HR child", run: "scripts/run-native-hr-policy-parser-contract.sh", suites: [supervised], child: true },
];
function finiteFixture(shape) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "TEST_ONLY-secondary-projection-")));
  try {
    const write = (name, value, mode = 0o644) => {
      const file = join(root, name); mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, value, { mode });
    };
    const suites = shape.suites ?? [ordinary];
    const marker = join(root, "TEST_ONLY-executed-suites");
    write("package.json", JSON.stringify({ type: "module", scripts: {} }));
    write(".github/workflows/fixture.yml", "jobs:\n  finite:\n    steps:\n      - name: TEST_ONLY execution\n" +
      (shape.guard ?? "") + "        run: |\n" + shape.run.split("\n").map(line => "          " + line).join("\n") + "\n");
    for (const suite of suites)
      write(suite, "import test from 'node:test'; import fs from 'node:fs'; test('TEST_ONLY finite marker', () => fs.appendFileSync(process.env.TEST_ONLY_CHILD_MARKER, " + JSON.stringify(suite + "\n") + "));\n");
    if (shape.sweep) {
      write("tools/ci/gate-sweep.mjs", readFileSync(join(repo, "tools/ci/gate-sweep.mjs")));
      write("tools/ci/gate-sweep.json", JSON.stringify({ gates: [{ id: "TEST_ONLY-finite", run: `node --test ${ordinary}` }] }));
    }
    if (shape.child) {
      write("scripts/run-native-hr-policy-parser-contract.sh", readFileSync(join(repo, "scripts/run-native-hr-policy-parser-contract.sh")), 0o755);
      write("scripts/check-native-hr-policy-parser-contract.mjs", "import {spawn} from 'node:child_process';\nconst binding = {node:{path:process.execPath}};\n" +
        "const child = spawn(binding.node.path, ['--test', 'scripts/check-native-hr-policy-parser-contract.test.mjs'], {stdio:'inherit', env:{...process.env, NODE_OPTIONS:'--test-reporter=tap', NODE_PATH:''}});\n" +
        "child.on('close', code => {process.exitCode = code ?? 1;});\n");
    }
    const env = { ...gitFixtureEnvironment(), PATH: dirname(realpathSync(process.execPath)) + ":" + process.env.PATH,
      NODE_OPTIONS: "--test-reporter=tap", NODE_PATH: "", TEST_ONLY_CHILD_MARKER: marker,
      CONSOLE_HR_POLICY_NODE: realpathSync(process.execPath), CONSOLE_HR_POLICY_BINDING: "/TEST_ONLY/unused",
      CONSOLE_HR_POLICY_BINDING_SHA256: "1".repeat(64) };
    delete env.NODE_TEST_CONTEXT;
    const execute = (file, args) => spawnSync(file, args, { cwd: root, env, encoding: "utf8", timeout: 10_000, maxBuffer: 1024 * 1024 });
    for (const args of [["init", "--quiet"], ["add", "."]]) {
      const result = execute("git", args); assert.equal(result.status, 0, result.stderr); assert.equal(result.signal, null); assert(!result.error);
    }
    assert(!existsSync(marker));
    // Literal-disabled workflow steps have the same finite non-execution control.
    const actual = execute("/bin/sh", ["-c", shape.guard ? `if false; then\n${shape.run}\nfi` : shape.run]);
    assert.equal(actual.signal, null, actual.stderr); assert(!actual.error); assert.equal(actual.status, 0, actual.stderr);
    const executed = existsSync(marker) ? readFileSync(marker, "utf8").trim().split("\n") : [];
    const shouldExecute = ["direct Node", "invoked sweep", "basename collision", "supervised HR child"].includes(shape.name);
    assert.deepEqual(executed, shouldExecute ? [suites[0]] : [], "finite positive/negative execution prerequisite");
    if (executed.length) {
      assert.equal(executed.length, 1);
      assert.match(actual.stdout, /# tests 1\n[\s\S]*# pass 1\n# fail 0\n[\s\S]*# skipped 0\n/);
    } else assert.doesNotMatch(actual.stdout, /# tests [1-9]/);
    return { root, suites, executed, close: () => rmSync(root, { recursive: true }) };
  } catch (error) { rmSync(root, { recursive: true }); throw error; }
}
for (const shape of shapes)
  test(`finite secondary projection equals actual execution: ${shape.name}`, () => {
    const fixture = finiteFixture(shape);
    try {
      const report = resolveDarkSuites(fixture.root);
      assert.equal(report.class_id, "js-test-reachability.dark-suite");
      assert.equal(report.suite_count, fixture.suites.length);
      assert.deepEqual([...report.wired].sort(), [...fixture.executed].sort(), "text cannot wire an inert suite; an executing exact child cannot remain dark");
      assert.equal(report.wired_count, fixture.executed.length);
      assert.equal(report.dark_count, fixture.suites.length - fixture.executed.length);
      assert.deepEqual(report.dark.map(entry => entry.suite).sort(), fixture.suites.filter(suite => !fixture.executed.includes(suite)).sort());
      assert.deepEqual(report.npm_scripts_reachable_from_workflow, []);
    } finally { fixture.close(); }
  });
