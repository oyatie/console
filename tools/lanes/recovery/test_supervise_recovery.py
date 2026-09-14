"""Owned subprocess deadline conformance; no Docker/product acceptance."""
import os
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


if __name__ == "__main__":
    unittest.main()
