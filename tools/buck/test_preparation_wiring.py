#!/usr/bin/env python3
"""Admission regression for three preparation roots and their execution topology.

Copy to tools/buck/test_preparation_wiring.py in the reviewed TEST candidate.
Scratch invocation uses CONSOLE_PREPARATION_WIRING_ROOT; no file generation.
"""
import importlib.util
import json
import os
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(os.environ.get("CONSOLE_PREPARATION_WIRING_ROOT", Path(__file__).resolve().parents[2])).resolve()
GENERATOR_PATH = ROOT / "tools/buck/gen_first_party.py"
SPEC = importlib.util.spec_from_file_location("gen_first_party", GENERATOR_PATH)
assert SPEC is not None and SPEC.loader is not None
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
RECOVERY = "tools/lanes/recovery/supervise_recovery.py"
CASES = {
    "replay_cannot_acknowledge_a_locally_visible_receipt_before_remote_apply": "",
    "lost_commit_transport_reply_replays_one_real_payroll_effect": "after-commit-response",
    "fresh_commit_wait_release_requires_explicit_remote_confirmation": "",
    "staging_success_and_idempotent_restage_wait_for_remote_confirmation": "",
    "required_remote_unknown_is_bounded_and_never_local_fallback": "",
    "finite_remote_bound_and_repeated_replay_preserve_exact_rows": "",
}
OBSERVER_CASE = "real_observer_installer_is_atomic_replay_exact_and_drift_refusing"
SUPERVISED_CASES = {**CASES, OBSERVER_CASE: ""}
LIVE_IF = "${{ needs.preflight.outputs.run_live_postgres == 'true' }}"
CEDAR_IF = "${{ needs.preflight.outputs.run_heavy == 'true' }}"
RECOVERY_IF = "${{ !cancelled() && needs.preflight.outputs.run_live_postgres == 'true' && steps.recovery-checkout.outcome == 'success' && steps.recovery-toolchain.outcome == 'success' && steps.recovery-image.outcome == 'success' }}"
IMAGE = "postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280"
CEDAR_ARGV = ["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-platform-authz", "--test", "cedar_pbac_readiness_cases", "--test", "cedar_pbac_legacy_only_observe_and_record", "--test", "cedar_diagnostic_fail_closed"]
RECOVERY_CARGO = ["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-payroll-adapter-postgres", "--test", "recovery"]
TARGETS = (
    ("console-app", "backend/app", "account_migration", "postgres"),
    ("console-platform-authz", "backend/crates/platform/authz", "cedar_diagnostic_fail_closed", "none"),
    ("console-payroll-adapter-postgres", "backend/crates/payroll/adapter-postgres", "recovery", "postgres-recovery"),
    ("console-payroll-adapter-postgres", "backend/crates/payroll/adapter-postgres", "durability_observer", "postgres-recovery"),
)


def workflow_commands(workflow=None):
    script = """
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
const root = process.argv[1];
const {executableWorkflowCommands,directExecutable} = await import(pathToFileURL(root+'/scripts/lib/ci-workflow-executables.mjs'));
const workflow = process.argv[2] === 'stdin' ? readFileSync(0,'utf8') : readFileSync(root+'/.github/workflows/ci.yml','utf8');
const commands = executableWorkflowCommands(workflow);
process.stdout.write(JSON.stringify(commands.filter(c=>!c.malformed&&!c.controlFlow).map(c=>({...c,raw:c.tokens,tokens:directExecutable(c.tokens).tokens}))));
"""
    result = subprocess.run(["node", "--input-type=module", "-e", script, str(ROOT), "stdin" if workflow is not None else "file"], input=workflow, check=True, text=True, capture_output=True)
    return json.loads(result.stdout)


def step_scalar(step, key):
    # First YAML key may follow '- ' rather than eight spaces (including shell).
    matches = re.findall(r"^(?:        )?" + re.escape(key) + r": ([^\n]+)", step, re.M)
    assert len(matches) <= 1, "duplicate step scalar is not execution authority"
    return matches[0].split(" #", 1)[0].strip() if matches else None


def fixture_workflow():
    """Positive wiring control only; never supplies product responses or DB truth."""
    steps = f"""      - name: Checkout
        id: recovery-checkout
        if: {LIVE_IF}
        uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0
        with:
          persist-credentials: false
      - name: Install Rust toolchain (pinned via rust-toolchain.toml)
        id: recovery-toolchain
        if: {LIVE_IF}
        uses: dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8
        with:
          toolchain: "1.98.1"
      - name: Prepare pinned recovery PostgreSQL image
        id: recovery-image
        if: {LIVE_IF}
        run: docker pull {IMAGE}
      - name: Run disposable PostgreSQL integration targets
        if: {LIVE_IF}
        run: tools/ci/cargo_needs_postgres.sh --workflow-only --shard-id domain-b --num-threads=1 --runner nextest
"""
    for case, mode in SUPERVISED_CASES.items():
        prefix = RECOVERY_CARGO[:-1] + ["durability_observer" if case == OBSERVER_CASE else "recovery"]
        command = " ".join(prefix + [case, "--", "--exact", "--test-threads=1", "--nocapture"])
        steps += f"""      - name: Recovery {case}
        if: {RECOVERY_IF}
        run: CONSOLE_RECOVERY_CUT={mode} python3 {RECOVERY} "$GITHUB_WORKSPACE" -- {command}
"""
    return "jobs:\n  domain-unit:\n    steps:\n      - name: Cedar\n        if: " + CEDAR_IF + "\n        run: SQLX_OFFLINE=true " + " ".join(CEDAR_ARGV) + "\n  postgres-reachability-domain-b:\n    needs: preflight\n    runs-on: ubuntu-latest\n    steps:\n" + steps


class PreparationWiringTests(unittest.TestCase):
    def test_all_new_roots_have_reviewed_resource_requirements(self):
        for package, directory, stem, resource in TARGETS:
            with self.subTest(package=package, test=stem):
                source = f"tests/{stem}.rs"
                self.assertIn((package, "test.integration", source), GENERATOR.discovered_test_resource_keys(str(ROOT / directory), package))
                self.assertEqual(resource, GENERATOR.resource_requirement(package, "test.integration", source))

    def test_publication_nested_module_is_materialized_without_duplicate_root(self):
        crate = ROOT / "backend/app"
        config = GENERATOR.integration_resource_config("console-app", "tests/auth_rest.rs")
        materialized = {file.relative_to(crate).as_posix() for pattern in config["srcs"] for file in crate.glob(pattern) if file.is_file()}
        source = "tests/auth_rest/publication_privileges.rs"
        self.assertTrue((crate / source).is_file())
        self.assertIn(source, materialized, "Cargo-readable publication module absent from Buck materialization")
        self.assertNotIn(("console-app", "test.integration", source), GENERATOR.discovered_test_resource_keys(str(crate), "console-app"))

    def test_generated_new_targets_preserve_exact_resource_taxonomy(self):
        for package, directory, stem, resource in TARGETS:
            with self.subTest(package=package, test=stem):
                text = (ROOT / directory / "BUCK").read_text()
                name = f"{package}-itest-{stem}"
                blocks = [block for block in re.findall(r"rust_test\(\n([\s\S]*?)\n\)", text) if f'name = "{name}"' in block]
                self.assertEqual(1, len(blocks), "new test must have exactly one generated target")
                labels_match = re.search(r"labels\s*=\s*(\[[^\]]*\])", blocks[0])
                self.assertIsNotNone(labels_match)
                labels = json.loads(labels_match.group(1))
                self.assertEqual([f"resource.{resource}"], [label for label in labels if label.startswith("resource.")])
                self.assertEqual("needs-postgres" in labels, resource == "postgres", "recovery must never enter the one-node PostgreSQL dispatcher")

    def test_ordinary_postgres_map_executes_migration_once_and_excludes_recovery(self):
        entries = json.loads((ROOT / "tools/ci/postgres-cargo-map.json").read_text())["entries"]
        selected = [entry for entry in entries if entry.get("package") == "console-app" and entry.get("test") == "account_migration"]
        self.assertEqual(1, len(selected), "migration binary needs one actual app-facet mapping")
        self.assertTrue(selected[0].get("in_workflow_postgres_job"))
        self.assertEqual(["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-app", "--test", "account_migration", "--", "--test-threads=1"], selected[0]["cargo_argv"])
        self.assertFalse(any(entry.get("package") == "console-payroll-adapter-postgres" and entry.get("test") in ("recovery", "durability_observer") for entry in entries), "two-node recovery cannot be a one-node map entry, even an inactive one")

    def assert_cedar_workflow(self, workflow=None):
        found = []
        for command in workflow_commands(workflow):
            tokens = command["tokens"]
            if tokens[:2] != ["cargo", "test"]:
                continue
            if any(tokens[i:i+2] == ["-p", "console-platform-authz"] for i in range(len(tokens))):
                if any(tokens[i:i+2] == ["--test", "cedar_diagnostic_fail_closed"] for i in range(len(tokens))):
                    self.assertEqual(CEDAR_ARGV, tokens, "build/list/filter/skip commands cannot prove test execution")
                    self.assertEqual(["SQLX_OFFLINE=true"] + CEDAR_ARGV, command["raw"], "Cedar cannot hide a cwd-changing env wrapper")
                    self.assertEqual("domain-unit", command["job"])
                    self.assertEqual(CEDAR_IF, step_scalar(command["step"], "if"), "Cedar runs on the intended heavy-work condition, not failure-only")
                    self.assertIsNone(step_scalar(command["step"], "working-directory"))
                    self.assertIn(command["shell"], (None, "bash"), "effective shell must execute the command")
                    self.assertIn(step_scalar(command["step"], "shell"), (None, "bash"))
                    found.append(command)
        self.assertEqual(1, len(found), "Cedar must execute, not merely compile or appear in an echo")

    def test_hermetic_cedar_binary_has_executable_cargo_workflow_path(self):
        self.assert_cedar_workflow()

    def assert_recovery_workflow(self, workflow=None):
        if workflow is None:
            workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        # A filter is safe only when the exact selected union equals real cases.
        source = (ROOT / "backend/crates/payroll/adapter-postgres/tests/recovery.rs").read_text()
        discovered = set(re.findall(r"#\[sqlx::test[^\n]*\]\s*async fn ([A-Za-z0-9_]+)", source))
        self.assertEqual(set(CASES), discovered)
        observer_source = (ROOT / "backend/crates/payroll/adapter-postgres/tests/durability_observer.rs").read_text()
        self.assertEqual([OBSERVER_CASE], re.findall(r"#\[sqlx::test[^\n]*\]\s*async fn ([A-Za-z0-9_]+)", observer_source))
        job_match = re.search(r"^  postgres-reachability-domain-b:\n([\s\S]*?)(?=^  [A-Za-z0-9_-]+:|\Z)", workflow, re.M)
        self.assertIsNotNone(job_match)
        job = job_match.group(1)
        self.assertNotRegex(job.split("    steps:")[0], r"(?m)^    (if|continue-on-error):", "existing required facet must remain unconditionally available after preflight")
        steps = re.split(r"^      - ", job, flags=re.M)[1:]
        scalar = step_scalar
        def by_id(identity):
            matches = [(index, step) for index, step in enumerate(steps) if scalar(step, "id") == identity]
            self.assertEqual(1, len(matches), "setup guard must identify one real setup step")
            index, step = matches[0]
            self.assertIsNone(scalar(step, "continue-on-error"))
            self.assertEqual(LIVE_IF, scalar(step, "if"))
            return index, step
        checkout_index, checkout = by_id("recovery-checkout")
        toolchain_index, toolchain = by_id("recovery-toolchain")
        image_index, image = by_id("recovery-image")
        self.assertEqual("actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", scalar(checkout, "uses"))
        self.assertRegex(checkout, r"(?m)^          persist-credentials: false$")
        self.assertNotRegex(checkout, r"(?m)^          (ref|repository|path):", "checkout cannot select another candidate/repository/path")
        self.assertEqual("dtolnay/rust-toolchain@29eef336d9b2848a0b548edc03f92a220660cdb8", scalar(toolchain, "uses"))
        self.assertRegex(toolchain, r'(?m)^          toolchain: "1\.98\.1"$')
        self.assertEqual("docker pull " + IMAGE, scalar(image, "run"))
        self.assertLess(checkout_index, toolchain_index)
        self.assertLess(toolchain_index, image_index)
        ordinary = [i for i, step in enumerate(steps) if step.splitlines()[0] == "name: Run disposable PostgreSQL integration targets"]
        self.assertEqual(1, len(ordinary))
        self.assertLess(image_index, ordinary[0], "image preparation must not be skipped after an ordinary test failure")
        found = {}
        scenario_steps = []
        for command in workflow_commands(workflow):
            tokens = command["tokens"]
            if tokens[:2] != ["python3", RECOVERY]:
                continue
            self.assertEqual("postgres-reachability-domain-b", command["job"], "reuse the existing required facet; no ungated recovery job")
            self.assertEqual(18, len(tokens), "exact supervisor/Cargo argv required")
            self.assertEqual("$GITHUB_WORKSPACE", tokens[2], "supervisor must run the checked-out candidate")
            self.assertEqual("--", tokens[3])
            cargo = tokens[4:]
            case = cargo[9]
            self.assertIn(case, SUPERVISED_CASES)
            self.assertEqual(RECOVERY_CARGO[:-1] + ["durability_observer" if case == OBSERVER_CASE else "recovery"], cargo[:9])
            self.assertEqual(["--", "--exact", "--test-threads=1", "--nocapture"], cargo[10:])
            self.assertNotIn(case, found, "duplicate recovery execution")
            self.assertEqual(["CONSOLE_RECOVERY_CUT=" + SUPERVISED_CASES[case]] + tokens, command["raw"], "only the exact fault-mode assignment may wrap the executor")
            self.assertEqual(RECOVERY_IF, scalar(command["step"], "if"), "both cases must run despite earlier test failure, only after real setup success")
            self.assertIsNone(scalar(command["step"], "continue-on-error"))
            self.assertIsNone(scalar(command["step"], "working-directory"))
            self.assertIn(command["shell"], (None, "bash"), "effective shell must execute the supervisor")
            self.assertIn(scalar(command["step"], "shell"), (None, "bash"))
            index = steps.index(command["step"])
            self.assertGreater(index, ordinary[0])
            self.assertNotIn(index, scenario_steps, "separate steps preserve second-case execution after first failure")
            scenario_steps.append(index)
            found[case] = command
        self.assertEqual(set(SUPERVISED_CASES), set(found), "both owner cases must execute through two-node supervision")

    def test_both_recovery_cases_have_separate_supervised_exact_workflow_paths(self):
        self.assert_recovery_workflow()

    def test_cedar_execution_oracle_rejects_nonexecuting_mutations(self):
        valid = fixture_workflow()
        self.assert_cedar_workflow(valid)
        mutations = [
            valid.replace("--test cedar_diagnostic_fail_closed", "--test cedar_diagnostic_fail_closed " + suffix, 1)
            for suffix in ("--no-run", "-- --list", "nonexistent_filter", "-- --skip row_visibility_fails_closed_when_allow_has_diagnostics", "-- --ignored")
        ]
        mutations += [valid.replace("run: SQLX_OFFLINE=true cargo", "run: echo cargo", 1), valid.replace("      - name: Cedar\n", "      - name: Cedar\n        if: false\n", 1)]
        mutations += [
            valid.replace("run: SQLX_OFFLINE=true", "run: env -C /tmp/unbound-checkout SQLX_OFFLINE=true", 1),
            valid.replace("        if: " + CEDAR_IF, "        if: failure()", 1),
            valid.replace("      - name: Cedar\n", "      - name: Cedar\n        working-directory: /tmp/unbound-checkout\n", 1),
            valid.replace("      - name: Cedar\n", "      - shell: echo {0}\n        name: Cedar\n", 1),
        ]
        for number, mutated in enumerate(mutations):
            with self.subTest(mutation=number), self.assertRaises(AssertionError):
                self.assert_cedar_workflow(mutated)

    def test_recovery_execution_oracle_rejects_unbound_or_success_only_mutations(self):
        valid = fixture_workflow()
        self.assert_recovery_workflow(valid)
        mutations = [
            valid.replace('"$GITHUB_WORKSPACE"', '/tmp/unbound-checkout', 1),
            valid.replace('        if: ' + RECOVERY_IF + '\n', '', 1),
            valid.replace('!cancelled()', 'success()', 1),
            valid.replace(" && steps.recovery-image.outcome == 'success'", '', 1),
            valid.replace('steps.recovery-image.outcome', 'steps.unrelated-success.outcome', 1),
            valid.replace('        id: recovery-image', '        id: unrelated-success', 1),
            valid.replace('run: docker pull', 'run: echo docker pull', 1),
            valid.replace('        uses: actions/checkout@', '        continue-on-error: true\n        uses: actions/checkout@', 1),
            valid.replace('          persist-credentials: false', '          persist-credentials: false\n          ref: stale-branch', 1),
            valid.replace('-- --exact --test-threads=1 --nocapture', '-- --list', 1),
            valid.replace(' --test recovery ', ' --no-run --test recovery ', 1),
            valid.replace('CONSOLE_RECOVERY_CUT=after-commit-response', 'CONSOLE_RECOVERY_CUT=before-send', 1),
            valid.replace('        if: ' + RECOVERY_IF, '        continue-on-error: true\n        if: ' + RECOVERY_IF, 1),
        ]
        image_start = valid.index('      - name: Prepare pinned recovery PostgreSQL image')
        ordinary_start = valid.index('      - name: Run disposable PostgreSQL integration targets')
        recovery_start = valid.index('      - name: Recovery ')
        mutations.append(valid[:image_start] + valid[ordinary_start:recovery_start] + valid[image_start:ordinary_start] + valid[recovery_start:])
        mutations.append(valid.replace("      - name: Recovery ", "      - shell: echo {0}\n        name: Recovery ", 1))
        for number, mutated in enumerate(mutations):
            with self.subTest(mutation=number), self.assertRaises(AssertionError):
                self.assert_recovery_workflow(mutated)

    def test_generic_queue_refuses_recovery_resource_instead_of_running_without_topology(self):
        script = """
import {pathToFileURL} from 'node:url';
const {partitionTargetsByMetadata} = await import(pathToFileURL(process.argv[1]+'/scripts/console/run-verification-queue.mjs'));
const target='//backend/crates/payroll/adapter-postgres:console-payroll-adapter-postgres-itest-recovery';
try { partitionTargetsByMetadata([target],{['root'+target]:{labels:['test.integration','resource.postgres-recovery']}}); process.exit(3); }
catch(error) { if(!/recovery|topology/i.test(error.message)) throw error; }
"""
        result = subprocess.run(["node", "--input-type=module", "-e", script, str(ROOT)], text=True, capture_output=True)
        self.assertEqual(0, result.returncode, "generic queue must fail closed with explicit topology guidance: " + result.stderr)


if __name__ == "__main__":
    unittest.main()
