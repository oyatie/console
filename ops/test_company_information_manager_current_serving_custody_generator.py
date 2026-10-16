"""Tests-first Manager PolicyV1 serving custody contract.

Only private trees execute the actual generator CLI. No database or build runs.
Observed finite capture pairs below are proposed under adopted Manager V3;
they do not freeze unreviewed finalizer/classifier postimages.
"""
from contextlib import contextmanager
import os
from pathlib import Path
import re
import subprocess
import sys
import unittest

ROOT = Path(os.environ.get('CONSOLE_MANAGER_TEST_SOURCE_ROOT',
                          Path(__file__).resolve().parents[1])).resolve()
sys.path.insert(0, str(ROOT / 'ops'))
import test_native_company_information_manager_current_capture_generator as capture
import test_native_org_unit_account_actor_staging_generator as prior

MODE = '--native-company-information-manager-current-serving-custody'
FINALIZER = 'ops/postgres-finalize-company-information-manager-current-policy-v1.sql'
CLASSIFIER = 'ops/postgres-company-information-manager-current-policy-v1-custody-state.sql'
APP_CLASSIFIER = 'backend/app/src/company_information_manager_current_policy_v1_custody_state.sql'
OUTPUTS = {FINALIZER, CLASSIFIER, APP_CLASSIFIER}
PAIRS = (
    ('3755792f52a4f78236a70e509f4f1546588049daa53f7545d8a101c74684d843',
     '20cac016d6cab101839b5689ecae0372dfb834d40184e445cd9ba73eabb85964',
     'c68aa085e01610c25db30ddc9c10ef5c49a73199173f17471dbbbc0e324c10ed'),
    ('9bde6410d51f8b665e5ca4400820e6d8a549a6c19b7b4d84feb26ef32c4bd604',
     '9c528d7d3bc708611b49cca19a4eab84c85b36da67959910a6b30d6b79dfd6f4',
     'eeef509f4e443a6ead4b8be28a5591f32b5ba33a097d2b4389b0482bfe806812'),
)


def relation_roster(sql):
    matches = re.findall(r'c\.relname IN \(([^)]+)\) ORDER BY c\.relname COLLATE "C"', sql)
    if len(matches) != 1:
        raise AssertionError('one immutable sorted relation-lock roster required')
    names = re.findall(r"'([a-z_0-9]+)'", matches[0])
    if len(names) != 61 or names != sorted(set(names)):
        raise AssertionError('exact PolicyV1 61-relation roster required')
    return names


class ManagerServingCustodyGenerator(unittest.TestCase):
    def setUp(self):
        capture.ROOT = ROOT
        self.h = capture.NativeCompanyInformationManagerCaptureGeneration()
        self.h.setUp()  # Frozen old definitions/defaults/modes, V4 files and decoder.

    @contextmanager
    def tree(self):
        with self.h.private_tree() as root:
            # Existing capture mode remains its own historical two-output mode.
            result = self.cli(root, capture.MODE)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.h.assert_exports({n: (root / n).read_text()
                                   for n in (self.h.owner, self.h.capture)})
            yield root

    def cli(self, root, *arguments):
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
        return subprocess.run([sys.executable, '-B', str(root / capture.SCRIPT), *arguments],
                              cwd=root, env=env, capture_output=True, text=True, timeout=30)

    def refuses(self, root, *arguments):
        before = capture.inventory(root, True)
        result = self.cli(root, *arguments)
        self.assertNotEqual(result.returncode, 0, 'refusal must be a failed actual CLI')
        self.assertEqual(capture.inventory(root, True), before,
                         'refusal changed source/output/root metadata')
        return result

    def generated(self, root):
        result = self.cli(root, MODE)
        if (result.returncode == 1 and result.stderr.startswith('usage: generate-account-custody.py ')
                and MODE not in result.stderr):
            self.fail('COMPANY_INFORMATION_MANAGER_CURRENT_SERVING_CUSTODY_MODE_MISSING\n'
                      + result.stderr)
        self.assertEqual(result.returncode, 0,
                         'PREREQUISITE: serving generator returned unrelated failure\n'
                         + result.stderr)
        files = {n: self.h.regular(root, n).read_text() for n in OUTPUTS}
        self.assertEqual(files[CLASSIFIER], files[APP_CLASSIFIER])
        return files

    def test_actual_cli_manager_serving_mode_is_additive_and_check_replays(self):
        with self.tree() as root:
            before = capture.inventory(root)
            files = self.generated(root)
            after = capture.inventory(root)
            self.assertEqual(set(after) - set(before), OUTPUTS)
            self.assertEqual({n: after[n] for n in before}, before,
                             'historical files were rewritten')
            self.generated(root)
            self.assertEqual(capture.inventory(root), after)
            readonly = capture.inventory(root, True)
            checked = self.cli(root, MODE, '--check')
            self.assertEqual(checked.returncode, 0, checked.stderr)
            self.assertEqual(capture.inventory(root, True), readonly)
            # Real historical CLI paths still execute, not only in-memory helpers.
            for args in (('--check',), (capture.MODE, '--check')):
                old = self.cli(root, *args)
                self.assertEqual(old.returncode, 0, old.stderr)
                self.assertEqual(capture.inventory(root, True), readonly)
            self.assertTrue(all(files.values()))

    def test_reviewed_v4_source_order_and_finite_pairs_lock_roster_remain_exact(self):
        with self.tree() as root:
            files = self.generated(root)
            finalizer = files[FINALIZER]
            positions = []
            for name in self.h.expected['source_order']:
                body = self.h.sources[name].decode()
                self.assertEqual(finalizer.count(body), 1, 'exact V4 body/ACL: ' + name)
                positions.append(finalizer.index(body))
            self.assertEqual(positions, sorted(positions), 'ACL must remain last')
            self.assertEqual(relation_roster(finalizer), relation_roster(
                (root / 'ops/postgres-finalize-native-company-policy.sql').read_text()))
            for text in (finalizer, files[CLASSIFIER]):
                for triple in PAIRS:
                    for digest in triple:
                        self.assertIn(digest, text, 'finite reviewed observation omitted')
            for suffix in ('finalized', 'install_required', 'absent', 'profile_mismatch'):
                self.assertIn('company_information_manager_current_policy_v1.' + suffix,
                              files[CLASSIFIER])
            self.assertIn("starts_with(p.proname,'identity_company_information_')",
                          files[CLASSIFIER], 'all-schema namespace scan required')

    def test_source_decoder_and_capture_faults_refuse_before_any_output_write(self):
        with self.tree() as root:
            self.generated(root)
            leaves = (*self.h.sources, self.h.expected['decoder_source'],
                      'ops/native-company-policy/closure-v1.sql',
                      'ops/native-company-policy/current-read-v2.sql')
            baseline = capture.inventory(root)
            for name in leaves:
                for fault in ('missing', 'changed', 'symlink', 'dangling', 'directory'):
                    with self.subTest(path=name, fault=fault), prior.leaf_fault(root / name, fault):
                        self.refuses(root, MODE)
                        self.refuses(root, MODE, '--check')
                    self.assertEqual(capture.inventory(root), baseline)

    def test_output_and_parent_faults_refuse_and_missing_check_never_generates(self):
        with self.tree() as root:
            self.refuses(root, MODE, '--check')
            self.assertFalse(any((root / n).exists() for n in OUTPUTS))
            self.generated(root)
            baseline = capture.inventory(root)
            for name in sorted(OUTPUTS):
                for fault in ('missing', 'changed', 'symlink', 'dangling', 'directory'):
                    with self.subTest(path=name, fault=fault), prior.leaf_fault(root / name, fault):
                        self.refuses(root, MODE, '--check')
                        if fault in ('symlink', 'dangling', 'directory'):
                            self.refuses(root, MODE)
                    self.assertEqual(capture.inventory(root), baseline)
            for name in ('ops', 'ops/native-company-information', 'backend/app/src'):
                for fault in ('missing', 'symlink', 'file'):
                    with self.subTest(path=name, fault=fault), prior.parent_fault(root / name, fault):
                        self.refuses(root, MODE)
                        self.refuses(root, MODE, '--check')
                    self.assertEqual(capture.inventory(root), baseline)

    def test_invalid_arguments_are_real_cli_refusal_without_dispatch_or_mutation(self):
        with self.tree() as root:
            # Adjacent valid dispatch must first work; a missing mode is not
            # evidence that any of the negative grammar cases passed.
            self.generated(root)
            for args in ((MODE, MODE), ('--check', MODE), (MODE, '--extra'),
                         (MODE, '--check', '--check'), (MODE, capture.MODE)):
                with self.subTest(arguments=args):
                    self.refuses(root, *args)


if __name__ == '__main__':
    unittest.main()
