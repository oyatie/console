"""Exact opt-in query export; native SQL/reader/workflow proof is separate.

V5 design 8e39f70157e5a7b32ccbca8ab288657f5084599c1a48d639174686f107400511.
Reuse historical private-tree controls. JSON SQL is a byte oracle only.
"""
from contextlib import contextmanager
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import test_native_org_unit_closed_bounded_reader_generator as bounded

historical = bounded.historical
ROOT, SCRIPT = historical.ROOT, historical.SCRIPT
FIXTURE = 'ops/fixtures/native-org-unit-account-actor-capture-export-contract-v1.json'
FIXTURE_SHA256 = '2e9a86873bd920c40c33f8ddeae8763deda19d51c83a384265326f2c4991fd40'
ENTRY = 'native_org_unit_account_actor_custody_capture_files'
MODE = '--native-org-unit-account-actor-capture'
SOURCE = 'ops/native-org-unit/account-actor-custody-capture-v1.sql'
OUTPUT = 'ops/postgres-capture-native-org-unit-account-actor-v1-custody.sql'
SOURCE_SHA256 = '724a777fb7cac58a4adf987cc9d22f56ce3a5029dcd43e0e8a68ed0f97c63410'


def inventory(root, *, strict=False):
    rows = historical.inventory(root, strict=strict)
    if strict:
        meta = root.lstat()
        rows['.'] = ('root', meta.st_mode, meta.st_size, meta.st_ino,
                     meta.st_mtime_ns, meta.st_ctime_ns)
    return rows


class NativeOrgUnitAccountActorCaptureGeneration(unittest.TestCase):
    def setUp(self):
        # Execute genuine default/old9/staging prerequisites before new entry.
        self.prior = bounded.NativeOrgUnitClosedBoundedReaderGeneration()
        self.prior.setUp()
        self.oracle, self.generator = self.prior.oracle, self.prior.generator
        raw = self.oracle.regular(ROOT, FIXTURE).read_bytes()
        self.assertEqual(historical.sha(raw), FIXTURE_SHA256)
        self.expected = json.loads(raw)
        self.assertEqual(self.expected['schema'],
                         'console.native_org_unit_account_actor_capture_export_contract.v1')
        self.assertEqual(self.expected['source'], SOURCE)
        self.assertEqual(self.expected['output'], OUTPUT)
        self.assertEqual(self.expected['reviewed_source_sha256'], SOURCE_SHA256)
        self.query = self.expected['query_utf8'].encode('utf8')
        self.assertEqual(historical.sha(self.query), SOURCE_SHA256)
        current = historical.definitions(self.oracle.regular(ROOT, SCRIPT).read_text())
        for name, digest in self.expected['old_definitions_except_main'].items():
            self.assertEqual(current.get(name), digest, 'old definition changed: ' + name)
        # The inherited fixture covers ten prior modes, so prove bounded too.
        files = self.generator.native_org_unit_closed_bounded_reader_files()
        self.prior.assert_exports(files)
        self.assertEqual(set(files), set(self.expected['bounded_outputs']))
        for name, digest in self.expected['bounded_outputs'].items():
            self.assertEqual(historical.sha(files[name].encode()), digest, name)
            self.assertEqual(self.oracle.regular(ROOT, name).read_bytes(),
                             files[name].encode(), name)

    def entry(self):
        entry = getattr(self.generator, ENTRY, None)
        self.assertTrue(callable(entry), 'NATIVE_ORG_UNIT_ACCOUNT_ACTOR_CAPTURE_FUNCTION_MISSING')
        return entry

    def assert_exports(self, files):
        self.assertIsInstance(files, dict)
        self.assertEqual(set(files), {OUTPUT}, 'one opt-in read-only output only')
        self.assertTrue(isinstance(files[OUTPUT], str) and files[OUTPUT])
        self.assertEqual(files[OUTPUT].encode('utf8'), self.query,
                         'exact reviewed input bytes; no header/transform/truncation')

    @contextmanager
    def private_tree(self):
        # Missing callable, never missing production SQL, is the target RED.
        self.entry()
        with self.prior.private_tree() as root:
            for name in self.expected['bounded_outputs']:
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(self.oracle.regular(ROOT, name), target)
            (root / SOURCE).write_bytes(self.query)  # evidence-only private input
            yield root

    def generate(self, root):
        with patch.object(self.generator, 'ROOT', root):
            return self.entry()()

    def cli(self, root, *arguments):
        with patch.object(self.generator, 'ROOT', root), \
                patch.object(self.generator.sys, 'argv', [str(root / SCRIPT), *arguments]):
            self.generator.main()

    def refuses(self, root, action):
        before = inventory(root, strict=True)
        self.oracle.refuses(root, action)
        self.assertEqual(inventory(root, strict=True), before,
                         'refusal changed descendants or root directory metadata')

    def old_cli(self, root):
        self.prior.old_cli(root)
        for arguments in ((bounded.MODE,), (bounded.MODE, '--check')):
            before = inventory(root, strict='--check' in arguments)
            result = self.oracle.process(root, *arguments)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(inventory(root, strict='--check' in arguments), before,
                             'eleventh prior mode changed bytes or check wrote files')

    def test_00_all154_prior_definitions33_defaults_and11_modes_remain_positive(self):
        self.assertEqual(len(self.expected['old_definitions_except_main']), 154)
        self.assertEqual(len(self.oracle.expected['old_default_outputs']), 33)
        self.assertEqual(len(self.oracle.expected['old_specialized_modes']), 9)
        self.assertNotIn(OUTPUT, self.generator.generated_files())
        # setUp actually generated and verified old9, staging and bounded too.

    def test_01_refusal_oracle_rejects_success_and_mutation_before_valid_refusal(self):
        self.oracle.test_01_refusal_oracle_rejects_success_and_accepts_real_refusal()
        with tempfile.TemporaryDirectory(prefix='org-capture-refusal-control-') as temporary:
            root = Path(temporary)
            sentinel = root / 'sentinel'
            sentinel.write_bytes(b'private evidence only\n')
            for fault in ('bytes', 'root_metadata'):
                def mutate_then_refuse():
                    if fault == 'bytes':
                        sentinel.write_bytes(b'changed private evidence\n')
                    else:
                        meta = root.stat()
                        os.utime(root, ns=(meta.st_atime_ns, meta.st_mtime_ns + 1000000000))
                    raise ValueError('valid error cannot hide a write')
                with self.subTest(fault=fault), self.assertRaises(AssertionError):
                    self.refuses(root, mutate_then_refuse)

    def test_10_required_callable_exports_exact_bytes_without_writes(self):
        with self.private_tree() as root:
            before = inventory(root, strict=True)
            self.assert_exports(self.generate(root))
            self.assert_exports(self.generate(root))
            self.assertEqual(inventory(root, strict=True), before)

    def test_20_cli_creates_only_one_leaf_replays_checks_and_preserves_old_dispatch(self):
        with self.private_tree() as root:
            self.old_cli(root)  # no default/old mode may activate this export
            before = inventory(root)
            self.refuses(root, lambda: self.cli(root, MODE, '--check'))
            result = self.oracle.process(root, MODE)
            self.assertEqual(result.returncode, 0,
                             'NATIVE_ORG_UNIT_ACCOUNT_ACTOR_CAPTURE_CLI_REQUIRED: ' + result.stderr)
            after = inventory(root)
            self.assertEqual(set(after) - set(before), {OUTPUT})
            self.assertEqual({name: after[name] for name in before}, before)
            self.assertEqual(self.oracle.regular(root, OUTPUT).read_bytes(), self.query)
            result = self.oracle.process(root, MODE)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(inventory(root), after, 'byte replay is not idempotent')
            unchanged = inventory(root, strict=True)
            result = self.oracle.process(root, MODE, '--check')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(inventory(root, strict=True), unchanged, 'check wrote files')
            self.old_cli(root)
            self.assertEqual(inventory(root), after, 'old modes activated/changed new export')

    def test_30_corrupt_missing_and_nonregular_input_refuses_without_writes(self):
        with self.private_tree() as root:
            self.assert_exports(self.generate(root))
            self.cli(root, MODE)
            original = inventory(root)
            for fault in ('changed', 'missing', 'symlink', 'dangling', 'directory'):
                with self.subTest(fault=fault), historical.leaf_fault(root / SOURCE, fault):
                    self.refuses(root, lambda: self.generate(root))
                    self.refuses(root, lambda: self.cli(root, MODE))
                    self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                self.assertEqual(inventory(root), original, 'input fixture did not restore')

    def test_40_invalid_input_output_parents_refuse_without_writes(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = inventory(root)
            for name in ('ops/native-org-unit', 'ops'):
                for fault in ('missing', 'symlink', 'file'):
                    with self.subTest(parent=name, fault=fault), \
                            historical.parent_fault(root / name, fault):
                        # Loaded main avoids an unrelated missing script failure.
                        self.refuses(root, lambda: self.generate(root))
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(inventory(root), original, 'parent fixture did not restore')

    def test_50_check_detects_missing_changed_output_without_repair(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = inventory(root)
            for fault in ('changed', 'missing'):
                with self.subTest(fault=fault), historical.leaf_fault(root / OUTPUT, fault):
                    self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                self.assertEqual(inventory(root), original)

    def test_60_nonregular_destination_refuses_both_modes_without_writes(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = inventory(root)
            for fault in ('symlink', 'dangling', 'directory'):
                with self.subTest(fault=fault), historical.leaf_fault(root / OUTPUT, fault):
                    self.refuses(root, lambda: self.cli(root, MODE))
                    self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                self.assertEqual(inventory(root), original)

    def test_70_invalid_mode_arguments_refuse_before_dispatch_without_writes(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            with patch.object(self.generator, ENTRY, wraps=self.entry()) as selected:
                for arguments in ((MODE, '--extra'), (MODE, MODE), (MODE, '--check', '--check'),
                                  ('--check', MODE), (MODE, '--check', '--extra'),
                                  (MODE, bounded.MODE), ('--native-org-unit-account-actor-captur',)):
                    with self.subTest(arguments=arguments):
                        self.refuses(root, lambda: self.cli(root, *arguments))
                        before = inventory(root, strict=True)
                        self.assertNotEqual(self.oracle.process(root, *arguments).returncode, 0)
                        self.assertEqual(inventory(root, strict=True), before)
                selected.assert_not_called()

    def test_80_byte_oracle_accepts_exact_and_rejects_missing_extra_corrupt_or_nontext(self):
        self.assert_exports({OUTPUT: self.query.decode('utf8')})
        bad = ({}, {OUTPUT: self.query.decode('utf8'), 'ops/unrequested.sql': 'extra'},
               {OUTPUT: self.query}, {OUTPUT: ''}, {OUTPUT: self.query.decode('utf8') + '\n'},
               {OUTPUT: self.query[:-1].decode('utf8')},
               {OUTPUT: self.query.decode('utf8').replace('\n', '\r\n')})
        for files in bad:
            with self.subTest(keys=list(files)), self.assertRaises(AssertionError):
                self.assert_exports(files)


if __name__ == '__main__':
    unittest.main()
