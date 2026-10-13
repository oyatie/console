// Miniature static repositories exercise the real gate; no business data or compiler runs.
import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {cpSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync} from "node:fs";
import {tmpdir} from "node:os";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";
import {it} from "node:test";
import {executableWorkflowCommands, directExecutable} from "./lib/ci-workflow-executables.mjs";
import {countDeclaredTestAttributes} from "./lib/executed-tests-baseline.mjs";

const ROOT = fileURLToPath(new URL("../", import.meta.url));
const PEOPLE = "backend/crates/payroll/ui/tests/react_people.rs";
const PEOPLE_TARGET = "//backend/crates/payroll/ui:console-payroll-ui-itest-react_people";
const DEFAULT_KEY = `${PEOPLE} --features islands,ssr`;
const ANCHORS = [
  ["backend/crates/ontology/rest/tests/object_policy_attach_as_runtime_role.rs", []],
  ["backend/crates/attendance/adapter-postgres/tests/concurrency.rs", []],
  ["backend/ci/gates/tenant-isolation/tests/owner_only_acl_postgres18.rs", []],
  ["backend/app/src/lib.rs", ["test-postgres"]],
];
function fixture({mode = "cargo", explicit = [], literalSsr = false, pinnedSsr = false, options = [], omitDeclaration = false, metadataFault = null, defaults = ["ssr", "islands"], guard = "ssr"} = {}) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "console-static-default-feature-test-")));
  const write = (path, value, mode) => {
    const absolute = join(root, path);
    mkdirSync(dirname(absolute), {recursive: true});
    writeFileSync(absolute, value, mode ? {mode} : undefined);
  };
  try {
    cpSync(join(ROOT, "scripts"), join(root, "scripts"), {recursive: true});
    write("tools/buck/BUCK", "");
    const targets = [], counts = {}, workflow = [];
    const declare = (source, features, index) => {
      const directory = dirname(source).replace(/\/(?:src|tests)$/, "");
      const target = `//${directory}:case_${index}`;
      write(`${directory}/BUCK`, `${readExisting(`${directory}/BUCK`)}rust_test(\n    name = "case_${index}",\n    crate_root = "${source}",\n    features = ${JSON.stringify(features)},\n)\n`);
      targets.push(target);
    };
    const readExisting = path => {
      try { return readFileSync(join(root, path), "utf8"); } catch (error) {
        if (error.code !== "ENOENT") throw error;
        return "";
      }
    };
    for (const [index, [source, features]] of ANCHORS.entries()) {
      write(source, `${source.endsWith("/src/lib.rs") ? "#[cfg(test)]\nmod tests {\n" : ""}#[test]\nfn anchor() {}\n${source.endsWith("/src/lib.rs") ? "}\n" : ""}`);
      counts[source] = 1;
      declare(source, features, index);
    }
    write(PEOPLE, `#![cfg(feature = "${guard}")]\n` + Array.from({length: 8}, (_, i) => `#[test]\nfn case_${i}() {}\n`).join(""));
    counts[PEOPLE] = 8;
    if (!omitDeclaration) declare(PEOPLE, guard === "dev-auth" ? ["dev-auth"] : ["islands", "ssr"], "people");
    if (literalSsr) declare(PEOPLE, ["ssr"], "people_ssr");
    if (explicit.length) declare(PEOPLE, [...defaults, ...explicit], "people_extra");
    const anchorTargets = targets.slice(0, ANCHORS.length);
    workflow.push(`tools/buck2 test ${anchorTargets.join(" ")}`);
    const argv = ["cargo", "test", "-p", "console-payroll-ui", "--test", "react_people", ...options];
    if (mode === "cargo") workflow.push(argv.join(" "));
    if (mode === "postgres") {
      workflow.push("tools/ci/cargo_needs_postgres.sh --workflow-only");
      write("tools/ci/postgres-cargo-map.json", JSON.stringify({entries: [{name: "people", in_workflow_postgres_job: true, cargo_argv: argv}]}));
    }
    if (explicit.length) workflow.push(`${argv.join(" ")} --features ${explicit.join(",")}`);
    if (mode === "buck") workflow.push(`tools/buck2 test ${targets.slice(ANCHORS.length).join(" ")}`);
    if (mode === "missing") delete counts[PEOPLE];
    write(".github/workflows/ci.yml", `jobs:\n  static-fixture:\n    steps:\n      - name: Run\n        run: |\n${workflow.map(line => `          ${line}`).join("\n")}\n`);
    write("docs/program/executed-tests-baseline.json", JSON.stringify({dark_baseline: 0, deferred_fixture: [], defined_feature_variants: pinnedSsr ? [`${PEOPLE} --features ssr`] : [], test_attribute_baseline: counts}));
    const metadata = {packages: [{name: "console-payroll-ui", features: {default: defaults, ssr: [], islands: [], extra: [], "dev-auth": []}, targets: [{name: "react_people", kind: ["test"], test: true, src_path: join(root, PEOPLE)}]}]};
    if (metadataFault === "missing-owner") metadata.packages[0].targets = [];
    if (metadataFault === "ambiguous-owner") metadata.packages.push({...metadata.packages[0], name: "other-owner"});
    if (metadataFault === "malformed-default") metadata.packages[0].features.default = "ssr";
    write("fake-bin/cargo", `#!${process.execPath}\nprocess.stdout.write(${JSON.stringify(JSON.stringify(metadata))});\n`, 0o700);
    const result = spawnSync(process.execPath, ["scripts/check-executed-tests.mjs", "--json"], {cwd: root, env: {...process.env, PATH: `${join(root, "fake-bin")}:${process.env.PATH}`}, encoding: "utf8", timeout: 20000});
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    return {...result, report: JSON.parse(result.stdout)};
  } finally { rmSync(root, {recursive: true, force: true}); }
}

it("source-derived cfg minimum includes package defaults without inventing SSR-only work", () => {
  const result = fixture({mode: "missing"});
  assert.equal(result.status, 1);
  assert.deepEqual(result.report.dark, [DEFAULT_KEY]);
  assert.deepEqual(result.report.unresolved, []);
});
it("guarded default definition survives deletion of both declaration and workflow", () => {
  const result = fixture({mode: "missing", omitDeclaration: true});
  assert.equal(result.status, 1);
  assert.deepEqual(result.report.dark, [DEFAULT_KEY]);
  assert.deepEqual(result.report.unresolved, []);
});
it("no-default package retains its nonempty required guard identity", () => {
  const result = fixture({defaults: [], guard: "dev-auth", explicit: ["dev-auth"], omitDeclaration: true});
  // Only the file-level definition exists; explicit Cargo must reach that exact guard.
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.report.dark, []);
});
it("unresolved or malformed metadata and unknown required feature fail closed", () => {
  for (const metadataFault of ["missing-owner", "ambiguous-owner", "malformed-default"]) {
    const result = fixture({mode: "buck", metadataFault});
    assert.equal(result.status, 1);
    assert.ok(result.report.unresolved.length > 0, metadataFault);
  }
  const result = fixture({mode: "buck", guard: "not-declared"});
  assert.equal(result.status, 1);
  assert.ok(result.report.unresolved.length > 0);
});
it("ordinary Cargo invocation uses its manifest defaults", () => {
  const result = fixture();
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.report.dark, []);
});
it("explicit Cargo features add to defaults and preserve both defined variants", () => {
  const result = fixture({explicit: ["extra"]});
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.report.defined, ANCHORS.length + 2);
  assert.deepEqual(result.report.dark, []);
});
it("PostgreSQL map Cargo invocation applies the same defaults", () => {
  const result = fixture({mode: "postgres"});
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.report.dark, []);
});
it("unsupported feature-mode overrides fail closed in both Cargo sources", () => {
  for (const mode of ["cargo", "postgres"]) for (const option of ["--no-default-features", "--all-features"]) {
    const result = fixture({mode, options: [option]});
    assert.equal(result.status, 1);
    assert.equal(result.report.unresolved.length, 1, `${mode} ${option} must be unresolved`);
  }
});
it("literal Buck SSR-only targets keep their independent identity", () => {
  const result = fixture({mode: "buck", literalSsr: true});
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.report.defined, ANCHORS.length + 2);
});
it("pinned SSR-only binary remains dark when only the default variant runs", () => {
  const result = fixture({mode: "buck", pinnedSsr: true});
  assert.equal(result.status, 1);
  assert.deepEqual(result.report.dark, [`${PEOPLE} --features ssr`]);
});
function nativePeopleCommands(workflow) {
  return executableWorkflowCommands(workflow).filter(command => {
    if (command.job !== "backend" || command.malformed || command.controlFlow) return false;
    const {tokens, malformed} = directExecutable(command.tokens);
    return !malformed && tokens[0] === "tools/buck2" && tokens[1] === "test"
      && tokens.includes(PEOPLE_TARGET) && !tokens.includes("--") && !tokens.includes("--filter");
  });
}
it("protected backend workflow executes the complete existing native People UI binary once", () => {
  const workflow = readFileSync(join(ROOT, ".github/workflows/ci.yml"), "utf8");
  assert.equal(nativePeopleCommands(workflow).length, 1);
});
it("workflow oracle rejects disabled, echoed, non-test and filtered claims", () => {
  const workflow = (line, fields = "", job = "backend") => `jobs:\n  ${job}:\n    steps:\n      - name: Native People\n${fields}        run: ${line}\n`;
  const valid = `tools/buck2 test ${PEOPLE_TARGET}`;
  assert.equal(nativePeopleCommands(workflow(valid)).length, 1);
  for (const invalid of [workflow(`echo ${valid}`), workflow(`tools/buck2 build ${PEOPLE_TARGET}`), workflow(valid, "        if: false\n"), workflow(valid, "        continue-on-error: true\n"), workflow(valid, "", "unprotected"), workflow(`${valid} -- --list`), workflow(`${valid} --filter first`)]) {
    assert.equal(nativePeopleCommands(invalid).length, 0);
  }
});
it("static source-attribute floor records all eight real People UI tests", () => {
  const baseline = JSON.parse(readFileSync(join(ROOT, "docs/program/executed-tests-baseline.json"), "utf8"));
  assert.equal(countDeclaredTestAttributes(readFileSync(join(ROOT, PEOPLE), "utf8")), 8);
  assert.equal(baseline.test_attribute_baseline[PEOPLE], 8);
});
