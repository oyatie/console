"""Bounded read-only OrgUnit exports; PostgreSQL/workflow acceptance is separate.

Design d161c42fe3afad7254faea4f5f1a1eafb3edf24a0ee32b3937be2f982327abd5.
Reuse the accepted staging fixture and its strict private-tree fault helpers.
No historical generator, fixture, output, registry or native test is rewritten.
"""
from contextlib import contextmanager
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import test_native_org_unit_account_actor_staging_generator as historical

ROOT, SCRIPT = historical.ROOT, historical.SCRIPT
ENTRY = 'native_org_unit_closed_bounded_reader_files'
MODE = '--native-org-unit-closed-bounded-reader'
CAPTURE_FUNCTION = 'native_org_unit_closed_perimeter_capture_files'
CUSTODY_FUNCTION = 'native_org_unit_closed_perimeter_custody_files'
CAPTURE_MODE = '--native-org-unit-closed-perimeter-capture'
CUSTODY_MODE = '--native-org-unit-closed-perimeter-custody'
CAPTURE = 'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql'
STATE = 'ops/postgres-native-org-unit-closed-perimeter-v1-custody-state.sql'
APP_STATE = 'backend/app/src/native_org_unit_closed_perimeter_v1_custody_state.sql'
BOUNDED_CAPTURE = 'ops/postgres-capture-native-org-unit-closed-perimeter-v1-bounded-custody.sql'
BOUNDED_STATE = 'ops/postgres-native-org-unit-closed-perimeter-v1-bounded-custody-state.sql'
BOUNDED_APP_STATE = 'backend/app/src/native_org_unit_closed_perimeter_v1_bounded_custody_state.sql'
OUTPUTS = {BOUNDED_CAPTURE, BOUNDED_STATE, BOUNDED_APP_STATE}
CAPTURE_SHA256 = '6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010'
STATE_SHA256 = '670564ce4a107746d8f50d316014b4aca763af0335d9c38a01fcfa23ec9e46dc'
STAGING_DEFINITION_SHA256 = '8a0ff7aac4903fccd8e9bbd62ed0d9008ddd79770d275710928727086317fc02'
BEFORE = '), snapshots AS (\n'
AFTER = '), snapshots AS MATERIALIZED (\n'


class NativeOrgUnitClosedBoundedReaderGeneration(unittest.TestCase):
    def setUp(self):
        # Existing prerequisites execute before the absent new callable: the
        # intended RED cannot be an unrelated missing source or old-mode drift.
        self.oracle = historical.NativeOrgUnitAccountActorStagingGeneration()
        self.oracle.setUp()
        self.generator = self.oracle.generator
        self.expected = self.oracle.expected
        actual = historical.definitions(self.oracle.regular(ROOT, SCRIPT).read_text())
        self.assertEqual(actual.get('function:native_org_unit_account_actor_staging_files'),
                         STAGING_DEFINITION_SHA256, 'accepted staging definition changed')
        self.staging = self.generator.native_org_unit_account_actor_staging_files()
        self.oracle.assert_exports(self.staging, self.oracle.sources())
        for name, row in self.expected['required_outputs'].items():
            self.assertEqual(historical.sha(self.oracle.regular(ROOT, name).read_bytes()),
                             row['sha256'], name)
        self.capture = getattr(self.generator, CAPTURE_FUNCTION)()
        self.custody = getattr(self.generator, CUSTODY_FUNCTION)()
        for mode, files in ((CAPTURE_MODE, self.capture), (CUSTODY_MODE, self.custody)):
            pins = self.expected['old_specialized_modes'][mode]['outputs']
            self.assertEqual(set(files), set(pins), mode)
            for name, digest in pins.items():
                self.assertIsInstance(files[name], str)
                self.assertEqual(historical.sha(files[name].encode()), digest, name)
                self.assertEqual(self.oracle.regular(ROOT, name).read_bytes(),
                                 files[name].encode(), name)
        self.assertEqual(len(self.capture), 2, 'capture includes its unchanged owner export')
        self.assertEqual(len(self.custody), 3, 'custody includes its unchanged finalizer')
        self.assertEqual(historical.sha(self.capture[CAPTURE].encode()), CAPTURE_SHA256)
        self.assertEqual(historical.sha(self.custody[STATE].encode()), STATE_SHA256)
        self.assertEqual(self.custody[STATE], self.custody[APP_STATE])

    def entry(self):
        entry = getattr(self.generator, ENTRY, None)
        self.assertTrue(callable(entry), 'NATIVE_ORG_UNIT_CLOSED_BOUNDED_READER_FUNCTION_MISSING')
        return entry

    def assert_exports(self, files):
        self.assertEqual(set(files), OUTPUTS, 'exact three opt-in read-only exports only')
        for name in OUTPUTS:
            self.assertTrue(isinstance(files[name], str) and files[name], name)
        for original, count, names in (
                (self.capture[CAPTURE], 1, (BOUNDED_CAPTURE,)),
                (self.custody[STATE], 2, (BOUNDED_STATE, BOUNDED_APP_STATE))):
            self.assertEqual(original.count(BEFORE), count)
            self.assertEqual(original.count(AFTER), 0, 'historical query is not materialized')
            for name in names:
                query = files[name]
                self.assertEqual(query.count(BEFORE), 0, name)
                self.assertEqual(query.count(AFTER), count, name)
                self.assertEqual(query, original.replace(BEFORE, AFTER),
                                 'only the complete fixed snapshots token changes: ' + name)
                inverse = query.replace(AFTER, BEFORE)
                self.assertEqual(inverse.encode(), original.encode(),
                                 'inverse restores every predicate/serializer/hash byte: ' + name)
                digest = CAPTURE_SHA256 if name == BOUNDED_CAPTURE else STATE_SHA256
                self.assertEqual(historical.sha(inverse.encode()), digest, name)
        self.assertEqual(files[BOUNDED_STATE].encode(), files[BOUNDED_APP_STATE].encode())

    @contextmanager
    def private_tree(self):
        self.entry()  # genuine missing target is the first new-contract failure
        with tempfile.TemporaryDirectory(prefix='org-closed-bounded-reader-test-') as temporary:
            root = Path(temporary)
            names = {SCRIPT, historical.FIXTURE, *self.expected['old_input_blobs'],
                     *self.expected['module_oracles_only_no_production_implementation'],
                     *self.expected['required_outputs']}
            for name in sorted(names):
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(self.oracle.regular(ROOT, name), target)
            yield root

    def generate(self, root):
        with patch.object(self.generator, 'ROOT', root):
            return self.entry()()

    def cli(self, root, *arguments):
        with patch.object(self.generator, 'ROOT', root), \
                patch.object(self.generator.sys, 'argv', [str(root / SCRIPT), *arguments]):
            self.generator.main()

    def refuses(self, root, action):
        self.oracle.refuses(root, action)

    def old_cli(self, root):
        self.oracle.old_cli(root)
        for arguments in ((historical.MODE,), (historical.MODE, '--check')):
            before = historical.inventory(root, strict='--check' in arguments)
            result = self.oracle.process(root, *arguments)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(historical.inventory(root, strict='--check' in arguments), before,
                             'tenth prior mode changed accepted bytes or --check wrote files')

    def test_00_all153_old_definitions360_inputs33_defaults_and10_modes_remain_positive(self):
        # setUp independently verifies fixture152 and the153rd staging callable.
        self.assertEqual(len(self.expected['old_definitions_except_main']), 152)
        self.assertEqual(len(self.expected['old_input_blobs']), 360)
        self.assertEqual(len(self.expected['old_default_outputs']), 33)
        self.assertEqual(len(self.expected['old_specialized_modes']), 9)
        self.assertEqual(len(self.staging), 4)
        self.assertFalse(OUTPUTS & set(self.generator.generated_files()),
                         'default generation must not enroll bounded readers')

    def test_01_refusal_oracle_detects_false_success_and_accepts_real_refusals(self):
        self.oracle.test_01_refusal_oracle_rejects_success_and_accepts_real_refusal()

    def test_10_exact_delegation_regular_inputs_three_outputs_and_one_two_byte_inverses(self):
        with self.private_tree() as root:
            before = historical.inventory(root, strict=True)
            regular = self.generator.company_provenance_regular_path
            with patch.object(self.generator, CAPTURE_FUNCTION,
                              return_value=self.capture) as capture, \
                    patch.object(self.generator, CUSTODY_FUNCTION,
                                 return_value=self.custody) as custody, \
                    patch.object(self.generator, 'company_provenance_regular_path',
                                 wraps=regular) as paths:
                self.assert_exports(self.generate(root))
            capture.assert_called_once_with()
            custody.assert_called_once_with()
            for name in (CAPTURE, STATE, APP_STATE):
                self.assertTrue(any(call.args == (name,) and call.kwargs == {'required': True}
                                    for call in paths.call_args_list),
                                'pinned regular original source read required: ' + name)
            self.assertEqual(historical.inventory(root, strict=True), before,
                             'read-only source generation changed its complete input tree')
            self.assert_exports(self.generate(root))
            self.assertEqual(historical.inventory(root, strict=True), before)

    def test_20_bad_delegated_rosters_types_hashes_and_nonidentical_state_pair_refuse(self):
        with self.private_tree() as root:
            cases = []
            for function, original in ((CAPTURE_FUNCTION, self.capture),
                                       (CUSTODY_FUNCTION, self.custody)):
                for name in sorted(original):
                    cases.append((function, 'missing:' + name,
                                  {key: value for key, value in original.items() if key != name}))
                cases.append((function, 'extra', {**original, 'ops/unrequested.sql': 'extra\n'}))
            cases += [
                (CAPTURE_FUNCTION, 'nonstring_capture', {**self.capture, CAPTURE: b'not text'}),
                (CAPTURE_FUNCTION, 'changed_capture_hash',
                 {**self.capture, CAPTURE: self.capture[CAPTURE] + '\n'}),
                (CAPTURE_FUNCTION, 'already_materialized_capture',
                 {**self.capture, CAPTURE: self.capture[CAPTURE].replace(BEFORE, AFTER)}),
                (CUSTODY_FUNCTION, 'nonstring_state', {**self.custody, STATE: None}),
                (CUSTODY_FUNCTION, 'nonidentical_pair',
                 {**self.custody, APP_STATE: self.custody[APP_STATE] + '\n'}),
                (CUSTODY_FUNCTION, 'changed_pair_hash',
                 {**self.custody, STATE: self.custody[STATE] + '\n',
                  APP_STATE: self.custody[APP_STATE] + '\n'}),
                (CUSTODY_FUNCTION, 'already_materialized_pair',
                 {**self.custody, STATE: self.custody[STATE].replace(BEFORE, AFTER),
                  APP_STATE: self.custody[APP_STATE].replace(BEFORE, AFTER)}),
            ]
            for function, fault, bad in cases:
                with self.subTest(function=function, fault=fault), \
                        patch.object(self.generator, CAPTURE_FUNCTION,
                                     return_value=bad if function == CAPTURE_FUNCTION else self.capture), \
                        patch.object(self.generator, CUSTODY_FUNCTION,
                                     return_value=bad if function == CUSTODY_FUNCTION else self.custody):
                    self.refuses(root, lambda: self.generate(root))
                    self.refuses(root, lambda: self.cli(root, MODE))
                    self.refuses(root, lambda: self.cli(root, MODE, '--check'))
            self.assert_exports(self.generate(root))

    def test_30_pinned_original_and_inherited_source_leaves_refuse_before_any_write(self):
        with self.private_tree() as root:
            self.assert_exports(self.generate(root))
            migrations = sorted((root / 'backend/crates/platform/db/migrations').glob('*.sql'))
            self.assertEqual(len(migrations), 231)
            names = (CAPTURE, STATE, APP_STATE, 'ops/native-org-unit/closed-perimeter-v1.sql',
                     'ops/account-custody-migrations.sha384',
                     str(migrations[0].relative_to(root)), str(migrations[-1].relative_to(root)))
            original = historical.inventory(root)
            for name in names:
                for fault in ('changed', 'missing', 'symlink', 'dangling', 'directory'):
                    with self.subTest(input=name, fault=fault), historical.leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.generate(root))
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(historical.inventory(root), original, 'input fixture did not restore')

    def test_40_invalid_input_and_output_parents_refuse_all_modes_without_writes(self):
        with self.private_tree() as root:
            self.generate(root)
            original = historical.inventory(root)
            for name in ('ops', 'ops/native-org-unit', 'backend', 'backend/app',
                         'backend/app/src', 'backend/crates/platform/db/migrations'):
                for fault in ('missing', 'symlink', 'file'):
                    with self.subTest(parent=name, fault=fault), historical.parent_fault(root / name, fault):
                        self.refuses(root, lambda: self.generate(root))
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(historical.inventory(root), original, 'parent fixture did not restore')

    def test_50_private_cli_creates_only_three_leaves_checks_replays_and_preserves_old_dispatch(self):
        with self.private_tree() as root:
            self.old_cli(root)
            before = historical.inventory(root)
            self.refuses(root, lambda: self.cli(root, MODE, '--check'))
            created = self.oracle.process(root, MODE)
            self.assertEqual(created.returncode, 0,
                             'NATIVE_ORG_UNIT_CLOSED_BOUNDED_READER_CLI_REQUIRED: ' + created.stderr)
            after = historical.inventory(root)
            self.assertEqual(set(after) - set(before), OUTPUTS)
            self.assertEqual({name: after[name] for name in before}, before,
                             'new mode changed an old source/artifact')
            self.assert_exports({name: self.oracle.regular(root, name).read_text() for name in OUTPUTS})
            replay = self.oracle.process(root, MODE)
            self.assertEqual(replay.returncode, 0, replay.stderr)
            self.assertEqual(historical.inventory(root), after, 'byte replay is not idempotent')
            unchanged = historical.inventory(root, strict=True)
            checked = self.oracle.process(root, MODE, '--check')
            self.assertEqual(checked.returncode, 0, checked.stderr)
            self.assertEqual(historical.inventory(root, strict=True), unchanged, '--check wrote files')
            self.old_cli(root)
            self.assertEqual(historical.inventory(root), after, 'prior modes adopted bounded readers')

    def test_60_check_detects_each_missing_or_changed_output_without_repair(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = historical.inventory(root)
            for name in sorted(OUTPUTS):
                for fault in ('changed', 'missing'):
                    with self.subTest(output=name, fault=fault), historical.leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(historical.inventory(root), original)

    def test_70_every_nonregular_destination_is_refused_before_any_partial_write(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            original = historical.inventory(root)
            for name in sorted(OUTPUTS):
                for fault in ('symlink', 'dangling', 'directory'):
                    with self.subTest(output=name, fault=fault), historical.leaf_fault(root / name, fault):
                        self.refuses(root, lambda: self.cli(root, MODE))
                        self.refuses(root, lambda: self.cli(root, MODE, '--check'))
                    self.assertEqual(historical.inventory(root), original)

    def test_80_fixed_mode_rejects_unsupported_arguments_without_dispatch_or_writes(self):
        with self.private_tree() as root:
            self.cli(root, MODE)
            with patch.object(self.generator, ENTRY, wraps=self.entry()) as selected:
                for arguments in ((MODE, '--extra'), (MODE, MODE), (MODE, '--check', '--check'),
                                  ('--check', MODE), (MODE, '--check', '--extra'),
                                  (MODE, CAPTURE_MODE), ('--native-org-unit-closed-bounded',)):
                    with self.subTest(arguments=arguments):
                        self.refuses(root, lambda: self.cli(root, *arguments))
                        before = historical.inventory(root, strict=True)
                        self.assertNotEqual(self.oracle.process(root, *arguments).returncode, 0)
                        self.assertEqual(historical.inventory(root, strict=True), before)
                selected.assert_not_called()


if __name__ == '__main__':
    unittest.main()
