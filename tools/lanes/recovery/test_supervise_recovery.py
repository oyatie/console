"""Owned subprocess deadline conformance; no Docker/product acceptance."""
import os
import json
import pathlib
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from supervise_recovery import supervise


class SupervisorTests(unittest.TestCase):
    def test_returns_child_nonzero_status(self):
        self.assertEqual(supervise([sys.executable, "-c", "raise SystemExit(7)"], 1, .1), 7)

    def test_timeout_kills_term_ignoring_process_group(self):
        with tempfile.TemporaryDirectory() as d:
            child_file = pathlib.Path(d) / "child.pid"
            code = """import os,signal,subprocess,sys,time
signal.signal(signal.SIGTERM, signal.SIG_IGN)
p=subprocess.Popen([sys.executable,'-c','import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); time.sleep(30)'])
open(sys.argv[1],'w').write(str(p.pid))
time.sleep(30)
"""
            start = time.monotonic()
            self.assertEqual(supervise([sys.executable, "-c", code, str(child_file)], .5, .2), 124)
            self.assertLess(time.monotonic() - start, 3)
            pid = int(child_file.read_text())
            # Killed grandchildren can briefly be unreaped zombies; neither a
            # zombie nor absence is an executing survivor.
            output = subprocess.run(["ps", "-o", "stat=", "-p", str(pid)], capture_output=True, text=True, check=False).stdout.strip()
            self.assertTrue(not output or output.startswith("Z"), "owned descendant survived killpg")


class RecoveryCleanupTests(unittest.TestCase):
    def run_cleanup(self, inventory_failure="", remaining="", removal_failure=False, original_exit=0, failed_output=False):
        repo = pathlib.Path(__file__).resolve().parents[3]
        source = (repo / "tools/lanes/recovery/recovery_postgres.sh").read_text()
        # Execute the actual initialization, cleanup function, and installed
        # EXIT/TERM/INT traps. Stop before any resource creation or credentials.
        boundary = "pw() { openssl rand -hex 32; }"
        self.assertEqual(source.count(boundary), 1)
        prefix = source.split(boundary)[0]
        self.assertIn("trap cleanup EXIT", prefix)
        with tempfile.TemporaryDirectory(prefix="recovery-cleanup-test-") as directory:
            root = pathlib.Path(directory)
            docker = root / "docker"
            docker.write_text(r"""#!/usr/bin/env python3
import json, os, sys
args = sys.argv[1:]
with open(os.environ['RECOVERY_TEST_CALLS'], 'a') as log:
    log.write(json.dumps(args) + '\n')
if args[:2] == ['image', 'inspect']:
    assert len(args) == 3 and args[2].startswith('postgres:18.6@sha256:')
    raise SystemExit(0)
if args[:2] in (['rm', '-fv'], ['volume', 'rm'], ['network', 'rm']):
    assert all(name.startswith('console-recovery-') for name in args[2:])
    raise SystemExit(42 if os.environ['RECOVERY_TEST_REMOVE_FAIL'] == '1' else 0)
kind = None
for candidate, command in [('container', ['ps', '-aq']), ('volume', ['volume', 'ls', '-q']), ('network', ['network', 'ls', '-q'])]:
    if args[:len(command)] == command:
        assert args[len(command):len(command)+1] == ['--filter']
        assert len(args) == len(command)+2
        assert args[-1].startswith('label=console.recovery.run=console-recovery-')
        kind = candidate
        break
if kind is None:
    raise SystemExit('unexpected Docker call; fixture cannot start services')
if kind == os.environ['RECOVERY_TEST_INVENTORY_FAIL']:
    if os.environ['RECOVERY_TEST_FAILED_OUTPUT'] == '1':
        print('partial-inventory-is-not-success')
    raise SystemExit(41)
if kind == os.environ['RECOVERY_TEST_REMAINING']:
    print('owned-resource-still-present')
""")
            docker.chmod(0o700)
            log = root / "calls.jsonl"
            env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ["PATH"],
                       TMPDIR=str(root), RECOVERY_TEST_CALLS=str(log),
                       RECOVERY_TEST_INVENTORY_FAIL=inventory_failure,
                       RECOVERY_TEST_REMAINING=remaining,
                       RECOVERY_TEST_REMOVE_FAIL=str(int(removal_failure)),
                       RECOVERY_TEST_FAILED_OUTPUT=str(int(failed_output)))
            result = subprocess.run(["bash", "-s", "--", str(repo), "--", "true"],
                                    input=prefix + "exit " + str(original_exit) + "\n",
                                    env=env, text=True, capture_output=True, timeout=5)
            self.assertTrue(log.exists(), result.stderr)
            calls = [json.loads(line) for line in log.read_text().splitlines()]
            inventories = [call for call in calls if "--filter" in call]
            self.assertEqual([call[:2] for call in inventories],
                             [["ps", "-aq"], ["volume", "ls"], ["network", "ls"]],
                             "every owned-resource inventory must be attempted")
            removals = [call for call in calls if call[:2] in
                        (["rm", "-fv"], ["volume", "rm"], ["network", "rm"])]
            run = result.stdout.split("recovery-fixture run=", 1)[1].split(" evidence=", 1)[0]
            self.assertEqual([call[-1] for call in inventories],
                             ["label=console.recovery.run=" + run] * 3)
            self.assertEqual(removals, [["rm", "-fv", run + "-seed", run + "-s", run + "-p"],
                                       ["volume", "rm", run + "-s-data", run + "-p-data"],
                                       ["network", "rm", run + "-net"]])
            self.assertEqual(calls, [["image", "inspect",
                                      "postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280"]]
                             + removals + inventories,
                             "no unobserved Docker command or service attempt may be swallowed")
            return result

    def test_successful_empty_inventory_preserves_success(self):
        self.assertEqual(self.run_cleanup().returncode, 0)

    def test_failed_container_inventory_is_not_successful_absence(self):
        self.assertNotEqual(self.run_cleanup(inventory_failure="container").returncode, 0)

    def test_failed_volume_inventory_is_not_successful_absence(self):
        self.assertNotEqual(self.run_cleanup(inventory_failure="volume").returncode, 0)

    def test_failed_network_inventory_is_not_successful_absence(self):
        self.assertNotEqual(self.run_cleanup(inventory_failure="network").returncode, 0)


    def failed_inventory_with_partial_output(self, kind):
        result = self.run_cleanup(inventory_failure=kind, failed_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("FAIL: owned fixture " + kind + " inventory unavailable", result.stderr)

    def test_failed_container_inventory_partial_output_is_still_unavailable(self):
        self.failed_inventory_with_partial_output("container")

    def test_failed_volume_inventory_partial_output_is_still_unavailable(self):
        self.failed_inventory_with_partial_output("volume")

    def test_failed_network_inventory_partial_output_is_still_unavailable(self):
        self.failed_inventory_with_partial_output("network")

    def test_remaining_container_refuses_success(self):
        self.assertNotEqual(self.run_cleanup(remaining="container").returncode, 0)

    def test_remaining_volume_refuses_success(self):
        self.assertNotEqual(self.run_cleanup(remaining="volume").returncode, 0)

    def test_remaining_network_refuses_success(self):
        self.assertNotEqual(self.run_cleanup(remaining="network").returncode, 0)

    def test_successful_absence_preserves_original_failure(self):
        self.assertEqual(self.run_cleanup(original_exit=7).returncode, 7)

    def test_successful_absence_preserves_timeout_classification(self):
        self.assertEqual(self.run_cleanup(original_exit=124).returncode, 124)

    def test_removal_error_can_reconcile_to_verified_absence(self):
        self.assertEqual(self.run_cleanup(removal_failure=True).returncode, 0)


if __name__ == "__main__":
    unittest.main()
