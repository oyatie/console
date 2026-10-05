"""Required four-source OrgUnit actor export; no DB/profile/serving acceptance.

Design 6bb74306149b7b1790c4d89283edc1d7ed542b8101cbdabfe7b01c65ff86d25a.
All generation writes and hostile paths are confined to private temporary trees.
SQL in the fixture is an exact byte oracle, never a product implementation.
"""
import ast
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = 'ops/generate-account-custody.py'
FIXTURE = 'ops/fixtures/native-org-unit-account-actor-staging-source-contract-v1.json'
FIXTURE_SHA256 = '8b78060c401b8988be48ace63603bbd696235fbd5fe1228ce9752e60d1e624a7'
ENTRY = 'native_org_unit_account_actor_staging_files'
MODE = '--native-org-unit-account-actor-staging'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def definitions(source):
    result = {}
    for node in ast.parse(source).body:
        if isinstance(node, ast.FunctionDef) and node.name != 'main':
            name = 'function:' + node.name
        elif isinstance(node, ast.Assign) and all(isinstance(t, ast.Name) for t in node.targets):
            name = 'assignment:' + ','.join(t.id for t in node.targets)
        else:
            continue
        if name in result:
            raise AssertionError('duplicate generator definition: ' + name)
        result[name] = sha(ast.get_source_segment(source, node).encode())
    return result


def inventory(root, *, strict=False):
    result = {}
    for path in root.rglob('*'):
        if path.is_symlink():
            identity = ('symlink', os.readlink(path))
        elif path.is_file():
            identity = ('file', sha(path.read_bytes()))
        elif path.is_dir():
            identity = ('directory',)
        else:
            identity = ('special', path.lstat().st_mode)
        if strict:
            stat = path.lstat()
            identity += (stat.st_mode, stat.st_size, stat.st_ino,
                         stat.st_mtime_ns, stat.st_ctime_ns)
        result[str(path.relative_to(root))] = identity
    return result


@contextmanager
def leaf_fault(path, kind):
    saved = path.with_name(path.name + '.test-custody-original')
    if saved.exists() or saved.is_symlink():
        raise AssertionError('fault backup already exists')
    path.rename(saved)
    try:
        if kind == 'changed':
            path.write_bytes(saved.read_bytes() + b'\n')
        elif kind == 'symlink':
            path.symlink_to(saved)  # exact approved bytes still must be refused
        elif kind == 'dangling':
            path.symlink_to(saved.with_name(saved.name + '.absent'))
        elif kind == 'directory':
            path.mkdir()
        elif kind != 'missing':
            raise AssertionError('unknown leaf fault')
        yield
    finally:
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.exists():
            shutil.rmtree(path)
        saved.rename(path)


@contextmanager
def parent_fault(path, kind):
    saved = path.with_name(path.name + '.test-custody-parent')
    if saved.exists() or saved.is_symlink():
        raise AssertionError('fault parent backup already exists')
    path.rename(saved)
    try:
        if kind == 'symlink':
            path.symlink_to(saved, target_is_directory=True)
        elif kind == 'file':
            path.write_bytes(b'not a directory\n')
        elif kind != 'missing':
            raise AssertionError('unknown parent fault')
        yield
    finally:
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.exists():
            shutil.rmtree(path)
        saved.rename(path)


class NativeOrgUnitAccountActorStagingGeneration(unittest.TestCase):
    def setUp(self):
        raw = self.regular(ROOT, FIXTURE).read_bytes()
        self.assertEqual(sha(raw), FIXTURE_SHA256, 'reviewed immutable fixture differs')
        self.expected = json.loads(raw)
        self.assertEqual(self.expected['schema'],
                         'console.native_org_unit_account_actor_staging_source_contract.v1')
        self.assertEqual(self.expected['base'], 'c8a4bfdcbf7b60704fd1db1ecdbfcf08e559acf4')
        self.assertEqual(self.expected['design_sha256'],
                         '6bb74306149b7b1790c4d89283edc1d7ed542b8101cbdabfe7b01c65ff86d25a')
        source = self.regular(ROOT, SCRIPT).read_text()
        actual = definitions(source)
        for name, digest in self.expected['old_definitions_except_main'].items():
            self.assertEqual(actual.get(name), digest, 'old definition changed: ' + name)
        # Load trusted source without writing a __pycache__ beside repository SQL.
        self.generator = types.ModuleType('org_actor_staging_generator_test')
        self.generator.__file__ = str(ROOT / SCRIPT)
        exec(compile(source, str(ROOT / SCRIPT), 'exec'), self.generator.__dict__)
        # Genuine historical prerequisites execute BEFORE the absent new target.
        self.old_positive()

    def regular(self, root, name):
        path = root / name
        for parent in path.parents:
            if parent == root:
                break
            self.assertTrue(parent.is_dir() and not parent.is_symlink(), name)
        self.assertTrue(path.is_file() and not path.is_symlink(), name)
        return path

    def old_positive(self):
        for name, digest in self.expected['old_input_blobs'].items():
            self.assertEqual(sha(self.regular(ROOT, name).read_bytes()), digest, name)
        cases = [('generated_files', self.expected['old_default_outputs'])]
        cases += [(row['function'], row['outputs'])
                  for row in self.expected['old_specialized_modes'].values()]
        for name, expected in cases:
            entry = getattr(self.generator, name, None)
            self.assertTrue(callable(entry), 'historical generator prerequisite: ' + name)
            files = entry()
            self.assertEqual(set(files), set(expected), name)
            for path, digest in expected.items():
                self.assertIsInstance(files[path], str)
                self.assertEqual(sha(files[path].encode()), digest, path)
                self.assertEqual(sha(self.regular(ROOT, path).read_bytes()), digest, path)

    def entry(self):
        entry = getattr(self.generator, ENTRY, None)
        self.assertTrue(callable(entry), 'ORG_ACCOUNT_ACTOR_STAGING_SOURCE_TARGET_REQUIRED')
        return entry

    def sources(self):
        # Called only AFTER the required callable assertion. The RED base has
        # neither module; missing source is not manufactured as the target RED.
        result = {}
        for name, row in self.expected['module_oracles_only_no_production_implementation'].items():
            raw = self.regular(ROOT, name).read_bytes()
            self.assertEqual(raw, row['utf8'].encode(), name)
            self.assertEqual(sha(raw), row['sha256'], name)
            result[name] = raw
        return result

    def assert_exports(self, files, sources):
        expected = self.expected['required_outputs']
        self.assertEqual(set(files), set(expected), 'exact four uninstalled exports only')
        self.assertEqual(len(expected), 4)
        for name, row in expected.items():
            self.assertIsInstance(files[name], str)
            self.assertTrue(files[name], name)
            raw = files[name].encode()
            if row['kind'] == 'module':
                oracle = (row['header'] + '-- source: ' + row['source'] + '\n').encode()
                oracle += sources[row['source']]
            else:
                oracle = self.regular(ROOT, row['source']).read_bytes()
                self.assertEqual(sha(oracle), self.expected['old_input_blobs'][row['source']])
            self.assertEqual(raw, oracle, 'exact source composition: ' + name)
            self.assertEqual(sha(raw), row['sha256'], name)

    @contextmanager
    def private_tree(self):
        self.entry()  # intentionally fails on clean base before module checks
        self.sources()
        with tempfile.TemporaryDirectory(prefix='org-account-actor-source-test-') as temporary:
            root = Path(temporary)
            names = {SCRIPT, *self.expected['old_input_blobs'],
                     *self.expected['module_oracles_only_no_production_implementation']}
            for name in sorted(names):
                source = self.regular(ROOT, name)
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)
            yield root

    def generate(self, root):
        with patch.object(self.generator, 'ROOT', root):
            return self.entry()()

    def cli(self, root, *arguments):
        with patch.object(self.generator, 'ROOT', root), \
                patch.object(self.generator.sys, 'argv', [str(root / SCRIPT), *arguments]):
            self.generator.main()

    def process(self, root, *arguments):
        environment = dict(os.environ)
        environment['PYTHONDONTWRITEBYTECODE'] = '1'
        return subprocess.run([sys.executable, str(root / SCRIPT), *arguments],
                              cwd=root, env=environment, capture_output=True,
                              text=True, timeout=30)

    def refuses(self, root, action):
        before = inventory(root, strict=True)
        with self.assertRaises((SystemExit, ValueError, OSError)) as caught:
            action()
        self.assertEqual(inventory(root, strict=True), before,
                         'refusal wrote bytes, identities or metadata')
        if isinstance(caught.exception, SystemExit):
            self.assertNotIn(caught.exception.code, (None, 0),
                             'successful CLI exit is not refusal')

    def old_cli(self, root):
        modes = [(), ('--check',)]
        for mode in self.expected['old_specialized_modes']:
            modes += [(mode,), (mode, '--check')]
        for arguments in modes:
            before = inventory(root, strict='--check' in arguments)
            result = self.process(root, *arguments)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(inventory(root, strict='--check' in arguments), before,
                             'old CLI changed accepted bytes or --check wrote files')

    def test_00_frozen_old_definitions_inputs_and_callable_modes_remain_positive(self):
        # setUp proves these independently; deliberately no new-target requirement.
        self.assertEqual(len(self.expected['old_default_outputs']), 33)
        self.assertEqual(len(self.expected['old_specialized_modes']), 9)
        self.assertEqual(len(self.expected['old_definitions_except_main']), 152)
        self.assertEqual(len(self.expected['old_input_blobs']), 360)

    def test_01_refusal_oracle_rejects_success_and_accepts_real_refusal(self):
        def fail(error):
            raise error

        with tempfile.TemporaryDirectory(prefix='org-source-refusal-oracle-') as temporary:
            root = Path(temporary)
            (root / 'sentinel').write_bytes(b'private refusal evidence control\n')
            before = inventory(root, strict=True)
            for error in (SystemExit(0), SystemExit(None)):
                with self.subTest(success_exit=error.code):
                    with self.assertRaises(AssertionError):
                        self.refuses(root, lambda: fail(error))
                    self.assertEqual(inventory(root, strict=True), before)
            with self.subTest(normal_return=True):
                with self.assertRaises(AssertionError):
                    self.refuses(root, lambda: None)
                self.assertEqual(inventory(root, strict=True), before)
            for error in (SystemExit(2), SystemExit('invalid input'),
                          ValueError('bounded refusal'), OSError('bounded path refusal')):
                with self.subTest(refusal=type(error).__name__, reason=str(error)):
                    self.refuses(root, lambda: fail(error))
                    self.assertEqual(inventory(root, strict=True), before)

    def test_10_required_target_exports_exact_reviewed_sql_and_complete_captures(self):
        self.entry()
        sources = self.sources()
        with self.private_tree() as root:
            before = inventory(root, strict=True)
            self.assert_exports(self.generate(root), sources)
            self.assertEqual(inventory(root, strict=True), before,
                             'source generation function mutated its inputs')

    def test_20_private_cli_exact_four_outputs_replay_check_and_old_dispatch(self):
        with self.private_tree() as root:
            self.old_cli(root)
            before = inventory(root)
            self.refuses(root, lambda: self.cli(root, MODE, '--check'))
            result = self.process(root, MODE)
            self.assertEqual(result.returncode, 0,
                             'ORG_ACCOUNT_ACTOR_STAGING_CLI_REQUIRED: ' + result.stderr)
            after = inventory(root)
            self.assertEqual(set(after) - set(before), set(self.expected['required_outputs']))
            self.assertEqual({name: after[name] for name in before}, before,
                             'new mode changed an old source/artifact')
            emitted = {name: self.regular(root, name).read_text()
                       for name in self.expected['required_outputs']}
            self.assert_exports(emitted, self.sources())
            self.assertEqual(self.process(root, MODE).returncode, 0)
            self.assertEqual(inventory(root), after, 'byte replay is not idempotent')
            unchanged = inventory(root, strict=True)
            self.assertEqual(self.process(root, MODE, '--check').returncode, 0)
            self.assertEqual(inventory(root, strict=True), unchanged, '--check wrote files')
            self.old_cli(root)
            self.assertEqual(inventory(root), after, 'old/default dispatch adopted new outputs')

    def test_30_changed_missing_and_nonregular_input_leaves_refuse_without_writes(self):
        with self.private_tree() as root:
            self.assert_exports(self.generate(root), self.sources())
            names = set(self.expected['module_oracles_only_no_production_implementation'])
            names |= {row['source'] for row in self.expected['required_outputs'].values()
                      if row['kind'] == 'copy'}
            original = inventory(root)
            for name in sorted(names):
                for fault in ('changed', 'missing', 'symlink', 'dangling', 'directory'):
                    with self.subTest(input=name, fault=fault), leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.generate(root))
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(inventory(root), original, 'fault fixture did not restore')

    def test_40_invalid_input_parents_refuse_without_writes(self):
        with self.private_tree() as root:
            self.generate(root)
            original = inventory(root)
            # Includes shared capture/input/output ancestor ops. Use real loaded
            # main dispatch in memory when its script pathname is made invalid.
            for parent in (root / 'ops/native-org-unit', root / 'ops'):
                for fault in ('missing', 'symlink', 'file'):
                    with self.subTest(parent=str(parent.relative_to(root)), fault=fault), \
                            parent_fault(parent, fault):
                        self.refuses(root, lambda: self.generate(root))
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(inventory(root), original, 'parent fixture did not restore')

    def test_50_check_detects_each_missing_or_changed_output_without_repair(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = inventory(root)
            for name in sorted(self.expected['required_outputs']):
                for fault in ('changed', 'missing'):
                    with self.subTest(output=name, fault=fault), leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(inventory(root), original)

    def test_60_nonregular_output_leaves_refuse_both_modes_before_any_write(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = inventory(root)
            for name in sorted(self.expected['required_outputs']):
                for fault in ('symlink', 'dangling', 'directory'):
                    with self.subTest(output=name, fault=fault), leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(inventory(root), original)

    def test_70_fixed_new_mode_rejects_invalid_arguments_without_writes(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            for arguments in ((MODE, '--extra'), (MODE, '--check', '--check'),
                              ('--check', MODE), (MODE, '--check', '--extra'),
                              ('--native-org-unit-account-actor-stage',),
                              (MODE, '--company-provenance-capture')):
                with self.subTest(arguments=arguments):
                    self.refuses(root, lambda: self.cli(root, *arguments))
                    before = inventory(root, strict=True)
                    self.assertNotEqual(self.process(root, *arguments).returncode, 0)
                    self.assertEqual(inventory(root, strict=True), before)


if __name__ == '__main__':
    unittest.main()
