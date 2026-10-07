"""Opt-in Manager capture only; database/install/readiness remain unqualified.

Lifecycle design 999c457cae8cbae8640c6dc55e0819e0e2ee0cbba36e0ec67ed895b630255c8a.
Actual PolicyV1 predecessor, unchanged decoder and four reviewed functions.
All CLI/fault operations use private trees. No SQL or product build executes.
"""
from contextlib import contextmanager
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import types
import unittest
from unittest.mock import call, patch

import test_native_org_unit_account_actor_staging_generator as prior
from test_native_group_process_custody_generator import snapshot_fields

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = 'ops/generate-account-custody.py'
HISTORICAL_HANDOFF = 'backend/app/tests/auth_rest/native_account_company_handoff_browser.rs'
FIXTURE = 'ops/fixtures/native-company-information-manager-current-capture-export-contract-v1.json'
FIXTURE_SHA256 = 'd5a70980e78fd3ed6ad1bb1f4cbc0a7e07a6ae8c06a6bb703058ef29fd911cd6'
ENTRY = 'native_company_information_manager_current_capture_files'
MODE = '--native-company-information-manager-current-capture'


def inventory(root, strict=False):
    result = prior.inventory(root, strict=strict)
    if strict:
        meta = root.lstat()
        result['.'] = ('root', meta.st_mode, meta.st_size, meta.st_ino,
                       meta.st_mtime_ns, meta.st_ctime_ns)
    return result


class NativeCompanyInformationManagerCaptureGeneration(unittest.TestCase):
    def regular(self, root, name):
        return prior.NativeOrgUnitAccountActorStagingGeneration().regular(root, name)

    def setUp(self):
        raw = self.regular(ROOT, FIXTURE).read_bytes()
        self.assertEqual(prior.sha(raw), FIXTURE_SHA256)
        self.expected = json.loads(raw)
        self.assertEqual(self.expected['entry'], ENTRY)
        self.assertEqual(self.expected['mode'], MODE)
        self.assertEqual(self.expected['schema'],
                         'console.native_company_information_manager_current_capture_export_contract.v1')
        for key in ('phase_hashes_registered', 'database_owner_accepted',
                    'source_release_accepted', 'MVP_accepted'):
            self.assertIs(self.expected[key], False)
        source = self.regular(ROOT, SCRIPT).read_text()
        actual = prior.definitions(source)
        for name, digest in self.expected['old_definitions_except_main'].items():
            self.assertEqual(actual.get(name), digest, 'old definition changed: ' + name)
        self.generator = types.ModuleType('company_manager_capture_generator_test')
        self.generator.__file__ = str(ROOT / SCRIPT)
        exec(compile(source, str(ROOT / SCRIPT), 'exec'), self.generator.__dict__)
        # Genuine 33 defaults and every 13 existing modes work before target RED.
        self.old_positive()
        self.owner, self.capture = (self.expected[key]
                                   for key in ('owner_output', 'capture_output'))
        self.sources = {name: self.regular(ROOT, name).read_bytes()
                        for name in self.expected['source_order']}
        for name, raw in self.sources.items():
            self.assertEqual(prior.sha(raw), self.expected['sources'][name]['sha256'], name)
        self.predecessor = getattr(self.generator, self.expected['predecessor_function'])() + ';\n'
        self.assertEqual(prior.sha(self.predecessor.encode()), self.expected['predecessor_sha256'])
        self.assertEqual(self.regular(ROOT, self.expected['predecessor_output']).read_text(),
                         self.predecessor)
        for key in ('routine_anchor', 'snapshot_anchor'):
            self.assertEqual(self.predecessor.count(self.expected[key]), 1)
        self.query = self.predecessor.replace(self.expected['routine_anchor'],
            self.expected['routine_extension'] + self.expected['routine_anchor'])
        insertion = "  '" + self.expected['namespace_key'] + "'," + self.expected['namespace_expression'] + ',\n'
        self.query = self.query.replace(self.expected['snapshot_anchor'],
            self.expected['snapshot_anchor'] + insertion)
        self.owner_text = self.expected['owner_header'] + '\n'.join(
            '-- source: ' + name + '\n' + raw.decode() for name, raw in self.sources.items())

    def historical_handoff(self):
        # This retained fixture reference is not read by any generator entry.
        base = self.expected['base']
        row = self.expected['old_input_blobs'][HISTORICAL_HANDOFF]
        def git(*args):
            return subprocess.check_output(['git', '--no-replace-objects', '-c',
                'core.commitGraph=false', '-C', str(ROOT), *args], stderr=subprocess.PIPE)
        self.assertEqual(git('rev-parse', base + '^{tree}').decode().strip(),
                         self.expected['tree'], 'historical handoff source tree')
        expected = ('100644 blob ' + row['git_blob'] + '\t' + HISTORICAL_HANDOFF + '\0').encode()
        self.assertEqual(git('ls-tree', '-z', base, '--', HISTORICAL_HANDOFF), expected,
                         'historical handoff regular blob at exact path')
        raw = git('cat-file', 'blob', row['git_blob'])
        self.assertEqual(len(raw), row['bytes'], HISTORICAL_HANDOFF)
        self.assertEqual(prior.sha(raw), row['sha256'], HISTORICAL_HANDOFF)

    def old_positive(self):
        # The complete generator blob is base provenance; old definitions are frozen above.
        for name, row in self.expected['old_input_blobs'].items():
            if name == HISTORICAL_HANDOFF:
                self.historical_handoff()
            elif name != SCRIPT:
                self.assertEqual(prior.sha(self.regular(ROOT, name).read_bytes()), row['sha256'], name)
        cases = [('generated_files', self.expected['old_default_outputs'])]
        cases += [(row['function'], row['outputs'])
                  for row in self.expected['old_specialized_modes'].values()]
        for name, expected in cases:
            files = getattr(self.generator, name)()
            self.assertEqual(set(files), set(expected), name)
            for path, digest in expected.items():
                self.assertIsInstance(files[path], str)
                self.assertEqual(prior.sha(files[path].encode()), digest, path)
                self.assertEqual(prior.sha(self.regular(ROOT, path).read_bytes()), digest, path)

    def entry(self):
        function = getattr(self.generator, ENTRY, None)
        self.assertTrue(callable(function), 'COMPANY_INFORMATION_MANAGER_CURRENT_CAPTURE_EXPORT_MISSING')
        return function

    def assert_exports(self, files):
        self.assertEqual(set(files), {self.owner, self.capture}, 'two opt-in uninstalled outputs only')
        self.assertTrue(all(isinstance(value, str) and value for value in files.values()))
        self.assertEqual(files[self.owner], self.owner_text, 'full ordered function bodies plus final ACL')
        self.assertEqual(files[self.capture], self.query, 'only reviewed serializer/namespace deltas')
        fields = snapshot_fields(files[self.capture])
        self.assertEqual(fields, {**snapshot_fields(self.predecessor),
            self.expected['namespace_key']: self.expected['namespace_expression']})

    @contextmanager
    def private_tree(self):
        self.entry()
        with tempfile.TemporaryDirectory(prefix='company-manager-capture-test-') as temporary:
            root = Path(temporary)
            names = {SCRIPT, *self.expected['old_input_blobs'], *self.sources,
                     *self.expected['old_default_outputs']}
            names.update(name for row in self.expected['old_specialized_modes'].values()
                         for name in row['outputs'])
            for name in sorted(names):
                destination = root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(self.regular(ROOT, name), destination)
            yield root

    def generate(self, root):
        with patch.object(self.generator, 'ROOT', root):
            return self.entry()()

    def cli(self, root, *arguments):
        with patch.object(self.generator, 'ROOT', root), patch.object(
                self.generator.sys, 'argv', [str(root / SCRIPT), *arguments]):
            self.generator.main()

    def refuses(self, root, action):
        before = inventory(root, True)
        with self.assertRaises((SystemExit, ValueError)) as caught:
            action()
        if isinstance(caught.exception, SystemExit):
            self.assertNotIn(caught.exception.code, (None, 0), 'success is not refusal')
        self.assertEqual(inventory(root, True), before, 'refusal changed bytes or metadata')

    def old_cli(self, root):
        for args in ((), ('--check',)):
            self.cli(root, *args)
        for mode in self.expected['old_specialized_modes']:
            self.cli(root, mode)
            before = inventory(root, True)
            self.cli(root, mode, '--check')
            self.assertEqual(inventory(root, True), before)

    def test_00_156_definitions_33_defaults_all13_modes_and_decoder_are_positive(self):
        self.assertEqual(len(self.expected['old_definitions_except_main']), 156)
        self.assertEqual(len(self.expected['old_default_outputs']), 33)
        self.assertEqual(len(self.expected['old_specialized_modes']), 13)
        self.assertEqual(self.generator.company_enrollment_input_sql().encode(),
                         self.regular(ROOT, self.expected['decoder_source']).read_bytes())
        self.assertFalse({self.owner, self.capture} & set(self.generator.generated_files()))
        for row in self.expected['old_specialized_modes'].values():
            self.assertFalse({self.owner, self.capture} & set(row['outputs']))

    def test_01_oracle_detects_dropped_body_acl_decoder_namespace_metadata_and_wrong_predecessor(self):
        exact = {self.owner: self.owner_text, self.capture: self.query}
        self.assert_exports(exact)
        clauses = [self.owner_text.replace(raw.decode(), '', 1) for raw in self.sources.values()]
        clauses += [self.owner_text.replace('CREATE FUNCTION', 'CREATE PROCEDURE', 1),
                    self.owner_text.replace('REVOKE ALL ON FUNCTION', '-- omitted REVOKE', 1)]
        queries = [self.query.replace(self.expected['routine_extension'], '', 1),
                   self.query.replace('company_enrollment_decode_input_v1', 'dropped_decoder', 1),
                   self.query.replace(self.expected['namespace_expression'], 'NULL', 1),
                   self.query.replace("starts_with(p.proname,'identity_company_information_')",
                                      "n.nspname='public' AND p.prosecdef", 1),
                   self.query.replace("'source_sha256'", "'omitted_source_body'", 1),
                   self.query.replace("'config',p.proconfig", "'config',NULL", 1),
                   self.generator.native_people_directory_snapshot_query() + ';\n']
        for changed in clauses:
            with self.subTest(owner=prior.sha(changed.encode())), self.assertRaises(AssertionError):
                self.assert_exports({**exact, self.owner: changed})
        for changed in queries:
            with self.subTest(query=prior.sha(changed.encode())), self.assertRaises(AssertionError):
                self.assert_exports({**exact, self.capture: changed})
        for changed in ({}, {**exact, 'ops/extra.sql': 'extra'}, {**exact, self.owner: b'bytes'}):
            with self.subTest(keys=list(changed)), self.assertRaises(AssertionError):
                self.assert_exports(changed)

    def test_02_refusal_oracle_rejects_success_and_changed_file_or_root_metadata(self):
        with tempfile.TemporaryDirectory(prefix='manager-refusal-control-') as temporary:
            root = Path(temporary)
            sentinel = root / 'sentinel'
            sentinel.write_text('private evidence\n')
            with self.assertRaises(AssertionError):
                self.refuses(root, lambda: None)
            def false_success():
                raise SystemExit(0)
            with self.assertRaises(AssertionError):
                self.refuses(root, false_success)
            def implicit_success():
                raise SystemExit()
            with self.assertRaises(AssertionError):
                self.refuses(root, implicit_success)
            def mutation():
                sentinel.write_text('changed evidence\n')
                raise ValueError('a valid error cannot hide a write')
            with self.assertRaises(AssertionError):
                self.refuses(root, mutation)
            def root_metadata():
                meta = root.stat()
                os.utime(root, ns=(meta.st_atime_ns, meta.st_mtime_ns + 1000000000))
                raise ValueError("refusal cannot hide root metadata changes")
            with self.assertRaises(AssertionError):
                self.refuses(root, root_metadata)

    def test_03_historical_handoff_rejects_wrong_tree_mode_path_blob_and_content(self):
        base = self.expected['base']
        row = self.expected['old_input_blobs'][HISTORICAL_HANDOFF]
        args = [('rev-parse', base + '^{tree}'),
                ('ls-tree', '-z', base, '--', HISTORICAL_HANDOFF),
                ('cat-file', 'blob', row['git_blob'])]
        commands = [['git', '--no-replace-objects', '-c', 'core.commitGraph=false',
                     '-C', str(ROOT), *item] for item in args]
        exact = [subprocess.check_output(item, stderr=subprocess.PIPE) for item in commands]
        with patch.object(subprocess, 'check_output', side_effect=exact) as reader:
            self.historical_handoff()
            self.assertEqual(reader.call_args_list,
                             [call(item, stderr=subprocess.PIPE) for item in commands])
        faults = [('source-tree', 0, b'0' * 40 + b'\n'),
                  ('executable-mode', 1, exact[1].replace(b'100644', b'100755', 1)),
                  ('symlink-mode', 1, exact[1].replace(b'100644', b'120000', 1)),
                  ('path', 1, exact[1].replace(HISTORICAL_HANDOFF.encode(), b'other.rs', 1)),
                  ('blob', 1, exact[1].replace(row['git_blob'].encode(), b'0' * 40, 1)),
                  ('size', 2, exact[2] + b'\n'),
                  ('body', 2, bytes([exact[2][0] ^ 1]) + exact[2][1:])]
        for fault, index, changed in faults:
            responses = exact.copy()
            responses[index] = changed
            with self.subTest(fault=fault), patch.object(subprocess, 'check_output',
                    side_effect=responses), self.assertRaises(AssertionError):
                self.historical_handoff()
        for index, command in enumerate(commands):
            responses = exact[:index] + [subprocess.CalledProcessError(128, command)]
            with self.subTest(missing_object=index), patch.object(subprocess, 'check_output',
                    side_effect=responses), self.assertRaises(subprocess.CalledProcessError):
                self.historical_handoff()

    def test_04_current_handoff_drift_is_not_generator_input_but_migration_drift_is(self):
        with self.private_tree() as root:
            regular = self.regular
            with patch.object(self, 'regular', side_effect=lambda unused, name: regular(root, name)), \
                    patch.object(self.generator, 'ROOT', root):
                before = self.generator.generated_files()
                with prior.leaf_fault(root / HISTORICAL_HANDOFF, 'changed'):
                    self.old_positive()
                    self.assertEqual(self.generator.generated_files(), before)
                    self.assert_exports(self.generate(root))
                live = 'backend/crates/platform/db/migrations/0001_create_regions_branches.sql'
                with prior.leaf_fault(root / live, 'changed'):
                    with self.assertRaises(AssertionError):
                        self.old_positive()
                    self.assertNotEqual(self.generator.generated_files(), before)
                self.old_positive()
                self.assertEqual(self.generator.generated_files(), before)

    def test_10_named_export_pins_four_full_bodies_final_acl_and_decoder_dependency(self):
        with self.private_tree() as root:
            before = inventory(root, True)
            self.assert_exports(self.generate(root))
            self.assert_exports(self.generate(root))
            self.assertEqual(inventory(root, True), before)
            pins = getattr(self.generator, self.expected['source_pin_constant'])
            self.assertEqual(list(pins), self.expected['source_order'])
            self.assertEqual(pins, {name: row['sha256'] for name, row in self.expected['sources'].items()})
            self.assertEqual(getattr(self.generator, self.expected['dependency_pin_constant']),
                {self.expected['decoder_source']: self.expected['decoder_pin']['sha256']})
            routines = re.findall(r'CREATE FUNCTION\s+(public\.[a-z_0-9]+)\(', self.owner_text)
            self.assertEqual(routines, [s.split('(')[0] for s in self.expected['routine_signatures']])

    def test_20_private_cli_two_leaves_check_replay_and_old_dispatch_no_activation(self):
        with self.private_tree() as root:
            self.old_cli(root)
            before = inventory(root)
            self.refuses(root, lambda: self.cli(root, MODE, '--check'))
            self.cli(root, MODE)
            after = inventory(root)
            self.assertEqual(set(after) - set(before), {self.owner, self.capture})
            self.assertEqual({name: after[name] for name in before}, before)
            self.assert_exports({name: self.regular(root, name).read_text() for name in (self.owner, self.capture)})
            self.cli(root, MODE)
            self.assertEqual(inventory(root), after)
            readonly = inventory(root, True)
            self.cli(root, MODE, '--check')
            self.assertEqual(inventory(root, True), readonly)
            self.old_cli(root)
            self.assertEqual(inventory(root), after)

    def test_30_source_and_decoder_missing_changed_nonregular_refuse_without_writes(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            before = inventory(root)
            for name in (*self.sources, self.expected['decoder_source'],
                         'ops/native-company-policy/material-v1.sql'):
                for fault in ('missing', 'changed', 'symlink', 'dangling', 'directory'):
                    with self.subTest(name=name, fault=fault), prior.leaf_fault(root / name, fault):
                        for action in (lambda: self.generate(root), lambda: self.cli(root, MODE),
                                       lambda: self.cli(root, MODE, '--check')):
                            self.refuses(root, action)
                    self.assertEqual(inventory(root), before)

    def test_40_source_output_parent_refusals_preserve_all_bytes_and_metadata(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            before = inventory(root)
            for name in ('ops/native-company-information', 'ops/native-company-policy', 'ops'):
                for fault in ('missing', 'symlink', 'file'):
                    with self.subTest(name=name, fault=fault), prior.parent_fault(root / name, fault):
                        for action in (lambda: self.generate(root), lambda: self.cli(root, MODE),
                                       lambda: self.cli(root, MODE, '--check')):
                            self.refuses(root, action)
                    self.assertEqual(inventory(root), before)

    def test_50_missing_changed_output_check_and_nonregular_destinations_refuse(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            before = inventory(root)
            for name in (self.owner, self.capture):
                for fault in ('missing', 'changed', 'symlink', 'dangling', 'directory'):
                    with self.subTest(name=name, fault=fault), prior.leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                        if fault in ('symlink', 'dangling', 'directory'):
                            self.refuses(root, lambda: self.cli(root, MODE))
                    self.assertEqual(inventory(root), before)

    def test_60_invalid_arguments_refuse_before_export_dispatch(self):
        with self.private_tree() as root:
            with patch.object(self.generator, ENTRY, wraps=self.entry()) as selected:
                for args in ((MODE, MODE), ('--check', MODE), (MODE, '--extra'),
                             (MODE, '--check', '--check'), (MODE, '--company-provenance-capture')):
                    with self.subTest(arguments=args):
                        self.refuses(root, lambda: self.cli(root, *args))
                selected.assert_not_called()


if __name__ == '__main__':
    unittest.main()
