"""Real Git regression checks for the lane's changed-path custody boundary."""

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


spec = importlib.util.spec_from_file_location("fanout_status", Path(__file__).with_name("fanout.py"))
fanout = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = fanout
spec.loader.exec_module(fanout)


class StatusCustody(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.git("init", "--quiet")
        self.git("config", "user.name", "Lane fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "core.hooksPath", "/dev/null")
        self.git("config", "commit.gpgsign", "false")

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.root), *args], stderr=subprocess.PIPE)

    def write(self, name, content="before\n"):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def baseline(self, *names):
        for name in names:
            self.write(name)
        self.git("add", "--all")
        self.git("commit", "--quiet", "-m", "fixture")

    def test_clean_is_empty(self):
        self.baseline("backend/file.rs")
        self.assertEqual(fanout._changed(self.root), [])

    def test_first_unstaged_path_keeps_first_character(self):
        self.baseline("backend/file.rs")
        self.write("backend/file.rs", "after\n")
        self.assertEqual(fanout._changed(self.root), ["backend/file.rs"])

    def test_mixed_staged_unstaged_and_nested_untracked(self):
        self.baseline("a.rs", "b.rs")
        self.write("a.rs", "after\n")
        self.write("b.rs", "after\n")
        self.git("add", "b.rs")
        self.write("new/nested/c.rs")
        self.assertEqual(set(fanout._changed(self.root)), {"a.rs", "b.rs", "new/nested/c.rs"})

    def test_filenames_are_not_stripped_quoted_or_split(self):
        names = [' leading.rs', 'trailing.rs ', 'quote".rs', 'line\nbreak.rs', '급여.rs']
        self.baseline(*names)
        for name in names:
            self.write(name, "after\n")
        self.assertEqual(set(fanout._changed(self.root)), set(names))

    def test_rename_checks_both_source_and_destination(self):
        self.baseline("outside/source.rs")
        (self.root / "allowed").mkdir()
        self.git("mv", "outside/source.rs", "allowed/destination.rs")
        changed = fanout._changed(self.root)
        self.assertEqual(set(changed), {"outside/source.rs", "allowed/destination.rs"})
        self.assertEqual([p for p in changed if not fanout._in_slice(p, ["allowed"])], ["outside/source.rs"])

    def test_deleted_path_is_retained(self):
        self.baseline("backend/deleted.rs")
        (self.root / "backend/deleted.rs").unlink()
        self.assertEqual(fanout._changed(self.root), ["backend/deleted.rs"])

    def test_untracked_outside_allowlist_is_rejected(self):
        self.baseline("allowed/kept.rs")
        self.write("outside/new.rs")
        changed = fanout._changed(self.root)
        self.assertEqual(changed, ["outside/new.rs"])
        self.assertFalse(fanout._in_slice(changed[0], ["allowed"]))

    def test_git_failure_is_not_a_clean_tree(self):
        with tempfile.TemporaryDirectory() as nonrepo:
            with self.assertRaises(subprocess.CalledProcessError):
                fanout._changed(Path(nonrepo))


if __name__ == "__main__":
    unittest.main(verbosity=2)
