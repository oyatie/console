"""Explicit V2 capture export; defaults/history and installation stay unchanged.

Accepted finite-column design c0c96a5d6c69fdb8b26a2d4c23666d46106ccb0cb54b8b171b995812b53ef1fc.
Uses actual generator and retained private-tree refusal controls; no SQL is executed.
"""
from contextlib import contextmanager
import hashlib
import json
import shutil
import unittest
from unittest.mock import patch

import test_native_org_unit_account_actor_capture_generator as prior

ENTRY = 'native_org_unit_account_actor_custody_capture_v2_files'
MODE = '--native-org-unit-account-actor-capture-v2'
SOURCE = 'ops/native-org-unit/account-actor-custody-capture-v2.sql'
OUTPUT = 'ops/postgres-capture-native-org-unit-account-actor-v2-custody.sql'
ORACLE = 'ops/fixtures/native-org-unit-account-actor-capture-export-contract-v2.json'


class NativeOrgUnitCaptureV2Generation(unittest.TestCase):
    def setUp(self):
        self.prior = prior.NativeOrgUnitAccountActorCaptureGeneration()
        self.prior.setUp()
        self.generator = self.prior.generator
        self.query = self.prior.oracle.regular(prior.ROOT, SOURCE).read_bytes()
        oracle = json.loads(self.prior.oracle.regular(prior.ROOT, ORACLE).read_bytes())
        self.assertEqual(oracle['source'], SOURCE)
        self.assertEqual(oracle['output'], OUTPUT)
        self.assertEqual(oracle['source_bytes'], len(self.query))
        self.assertEqual(oracle['source_sha256'], hashlib.sha256(self.query).hexdigest())
        for field in ('phase_hashes_registered', 'database_owner_accepted',
                      'source_release_accepted', 'MVP_accepted'):
            self.assertIs(oracle[field], False)

    def entry(self):
        function = getattr(self.generator, ENTRY, None)
        self.assertTrue(callable(function), 'ORG_V2_OPT_IN_CAPTURE_EXPORT_MISSING')
        return function

    @contextmanager
    def private_tree(self):
        with self.prior.private_tree() as root:
            shutil.copyfile(self.prior.oracle.regular(prior.ROOT, SOURCE), root / SOURCE)
            self.prior.cli(root, prior.MODE)
            yield root

    def generate(self, root):
        with patch.object(self.generator, 'ROOT', root):
            return self.entry()()

    def cli(self, root, *arguments):
        with patch.object(self.generator, 'ROOT', root), patch.object(
                self.generator.sys, 'argv', [str(root / prior.SCRIPT), *arguments]):
            self.generator.main()

    def test_00_existing_default_and_V1_capture_are_preserved(self):
        self.assertEqual(prior.historical.definitions(
            self.prior.oracle.regular(prior.ROOT, prior.SCRIPT).read_text())['function:' + prior.ENTRY],
            'de39edd8f67e8ecc37532c446a7f92621a6504e62ccf9004ab5f35c61aa09292', 'historical V1 capture definition changed')
        self.prior.assert_exports(self.prior.entry()())
        self.assertNotIn(OUTPUT, self.generator.generated_files())
        self.assertNotIn(OUTPUT, self.prior.entry()())

    def test_10_exact_successor_bytes_read_only_and_no_default_activation(self):
        with self.private_tree() as root:
            before = prior.inventory(root, strict=True)
            self.assertEqual(self.generate(root), {OUTPUT: self.query.decode('utf8')})
            self.assertEqual(self.generate(root), {OUTPUT: self.query.decode('utf8')})
            self.assertEqual(prior.inventory(root, strict=True), before)
            self.assertEqual((root / prior.SOURCE).read_bytes(), self.prior.query)
            self.assertEqual((root / prior.OUTPUT).read_bytes(), self.prior.query)

    def test_20_cli_writes_one_output_and_check_replay_refusals_preserve_history(self):
        self.entry()
        with self.private_tree() as root:
            before = prior.inventory(root)
            self.prior.refuses(root, lambda: self.cli(root, MODE, '--check'))
            result = self.prior.oracle.process(root, MODE)
            self.assertEqual(result.returncode, 0, result.stderr)
            after = prior.inventory(root)
            self.assertEqual(set(after) - set(before), {OUTPUT})
            self.assertEqual({name: after[name] for name in before}, before)
            self.assertEqual((root / OUTPUT).read_bytes(), self.query)
            self.cli(root, MODE)
            self.assertEqual(prior.inventory(root), after)
            unchanged = prior.inventory(root, strict=True)
            self.cli(root, MODE, '--check')
            self.assertEqual(prior.inventory(root, strict=True), unchanged)
            self.prior.old_cli(root)
            self.prior.cli(root, prior.MODE, '--check')
            self.assertEqual(prior.inventory(root), after)
            for args in ((MODE, '--extra'), (MODE, MODE), ('--check', MODE),
                         (MODE, '--check', '--check'), (MODE, prior.MODE)):
                with self.subTest(arguments=args):
                    self.prior.refuses(root, lambda: self.cli(root, *args))
            for fault in ('missing', 'changed', 'symlink', 'dangling', 'directory'):
                with self.subTest(output=fault), prior.historical.leaf_fault(root / OUTPUT, fault):
                    self.prior.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    if fault in ('symlink', 'dangling', 'directory'):
                        self.prior.refuses(root, lambda: self.cli(root, MODE))
                self.assertEqual(prior.inventory(root), after)

    def test_30_bad_sources_and_parents_refuse_without_writes(self):
        self.entry()
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = prior.inventory(root)
            for fault in ('missing', 'changed', 'symlink', 'dangling', 'directory'):
                with self.subTest(source=fault), prior.historical.leaf_fault(root / SOURCE, fault):
                    for action in (lambda: self.generate(root), lambda: self.cli(root, MODE),
                                   lambda: self.cli(root, MODE, '--check')):
                        self.prior.refuses(root, action)
                self.assertEqual(prior.inventory(root), original)
            for name in ('ops/native-org-unit', 'ops'):
                for fault in ('missing', 'symlink', 'file'):
                    with self.subTest(parent=name, fault=fault), prior.historical.parent_fault(root / name, fault):
                        for action in (lambda: self.generate(root), lambda: self.cli(root, MODE),
                                       lambda: self.cli(root, MODE, '--check')):
                            self.prior.refuses(root, action)
                    self.assertEqual(prior.inventory(root), original)


if __name__ == '__main__':
    unittest.main()
