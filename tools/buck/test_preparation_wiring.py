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
}
TARGETS = (
    ("console-app", "backend/app", "account_migration", "postgres"),
    ("console-platform-authz", "backend/crates/platform/authz", "cedar_diagnostic_fail_closed", "none"),
    ("console-payroll-adapter-postgres", "backend/crates/payroll/adapter-postgres", "recovery", "postgres-recovery"),
)


def workflow_commands():
    script = """
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
const root = process.argv[1];
const {executableWorkflowCommands,directExecutable} = await import(pathToFileURL(root+'/scripts/lib/ci-workflow-executables.mjs'));
const commands = executableWorkflowCommands(readFileSync(root+'/.github/workflows/ci.yml','utf8'));
process.stdout.write(JSON.stringify(commands.filter(c=>!c.malformed&&!c.controlFlow).map(c=>({job:c.job,raw:c.tokens,tokens:directExecutable(c.tokens).tokens}))));
"""
    result = subprocess.run(["node", "--input-type=module", "-e", script, str(ROOT)], check=True, text=True, capture_output=True)
    return json.loads(result.stdout)


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
        self.assertFalse(any(entry.get("package") == "console-payroll-adapter-postgres" and entry.get("test") == "recovery" for entry in entries), "two-node recovery cannot be a one-node map entry, even an inactive one")

    def test_hermetic_cedar_binary_has_executable_cargo_workflow_path(self):
        found = []
        for command in workflow_commands():
            tokens = command["tokens"]
            if tokens[:2] != ["cargo", "test"]:
                continue
            if any(tokens[i:i+2] == ["-p", "console-platform-authz"] for i in range(len(tokens))):
                if any(tokens[i:i+2] == ["--test", "cedar_diagnostic_fail_closed"] for i in range(len(tokens))):
                    found.append(command)
        self.assertEqual(1, len(found), "Cedar must execute, not merely compile or appear in an echo")

    def test_both_recovery_cases_have_separate_supervised_exact_workflow_paths(self):
        # A filter is safe only when the exact selected union equals real cases.
        source = (ROOT / "backend/crates/payroll/adapter-postgres/tests/recovery.rs").read_text()
        discovered = set(re.findall(r"#\[sqlx::test[^\n]*\]\s*async fn ([A-Za-z0-9_]+)", source))
        self.assertEqual(set(CASES), discovered)
        found = {}
        for command in workflow_commands():
            tokens = command["tokens"]
            if tokens[:2] != ["python3", RECOVERY]:
                continue
            self.assertEqual("postgres-reachability-domain-b", command["job"], "reuse the existing required facet; no ungated recovery job")
            self.assertEqual("--", tokens[3])
            cargo = tokens[4:]
            self.assertEqual(["cargo", "test", "--locked", "--manifest-path", "backend/Cargo.toml", "-p", "console-payroll-adapter-postgres", "--test", "recovery"], cargo[:9])
            case = cargo[9]
            self.assertIn(case, CASES)
            self.assertEqual(["--", "--exact", "--test-threads=1", "--nocapture"], cargo[10:])
            self.assertNotIn(case, found, "duplicate recovery execution")
            self.assertIn("CONSOLE_RECOVERY_CUT=" + CASES[case], command["raw"], "each invocation binds its own exact fault mode")
            found[case] = command
        self.assertEqual(set(CASES), set(found), "both owner cases must execute through two-node supervision")

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
