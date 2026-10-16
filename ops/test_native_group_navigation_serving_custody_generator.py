"""Serving-only navigation classifier contract; routed PostgreSQL RED is separate.

Design f067dfb28287d8819cfd1e94402702e2a487b3059bd534a00094aa8a2088979b.
All CLI writes use private temporary trees; historical suites remain unchanged.
"""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import test_native_group_navigation_finalizer_generator as finalizer
import test_native_group_process_serving_custody_generator as historical
from test_company_provenance_capture_generator import inventory

ROOT, SCRIPT, GENERATOR = historical.ROOT, historical.SCRIPT, historical.GENERATOR
ENTRY = 'native_group_process_navigation_serving_custody_files'
MODE = '--native-group-process-navigation-serving-custody'
OUTPUT = 'backend/app/src/native_group_process_navigation_serving_v1_custody_state.sql'
BEFORE = '), snapshots AS (\n'
AFTER = '), snapshots AS MATERIALIZED (\n'


class NativeGroupNavigationServingCustodyGeneration(unittest.TestCase):
    def entry(self):
        entry = getattr(GENERATOR, ENTRY, None)
        self.assertTrue(callable(entry), 'NATIVE_GROUP_NAVIGATION_SERVING_CUSTODY_FUNCTION_MISSING')
        return entry

    def files(self):
        files = self.entry()()
        self.assertEqual(set(files), {OUTPUT}, 'one distinct serving output only')
        self.assertTrue(isinstance(files[OUTPUT], str) and files[OUTPUT])
        return files

    def cli(self, *arguments):
        with patch.object(GENERATOR.sys, 'argv', [str(SCRIPT), *arguments]):
            GENERATOR.main()

    def refused_without_writes(self, root, *arguments):
        before = inventory(root)
        with self.assertRaises(SystemExit):
            self.cli(*arguments)
        self.assertEqual(inventory(root), before, 'refusal must preserve the private tree')

    def test_one_output_delegates_exact_history_and_inverse_preserves_every_predicate(self):
        oracle = historical.NativeGroupNavigationCustodyGeneration()
        old = oracle.custody()
        original = old[historical.NAVIGATION_CLASSIFIER]
        self.assertEqual(historical.sha(original.encode()), finalizer.CLASSIFIER_SHA256)
        delegated = GENERATOR.native_group_process_navigation_custody_files
        with patch.object(GENERATOR, 'native_group_process_navigation_custody_files',
                          wraps=delegated) as selected:
            query = self.files()[OUTPUT]
        selected.assert_called_once_with()
        self.assertEqual(original.count(BEFORE), 1)
        self.assertEqual(query.count(AFTER), 1)
        self.assertEqual(query.count(BEFORE), 0)
        self.assertEqual(query, original.replace(BEFORE, AFTER))
        inverse = query.replace(AFTER, BEFORE)
        self.assertEqual(inverse, original, 'no other query byte may change')
        recovered = dict.fromkeys(historical.NAVIGATION_ARTIFACTS, inverse)
        with patch.object(oracle, 'custody', return_value=recovered):
            oracle.test_navigation_two_artifacts_bind_exact_measured_same_variant_pairs_and_raw_capture()
            oracle.test_navigation_both_phases_require_true_effective_rights()
            oracle.test_navigation_unique_match_and_complete_absence_are_only_disclosure_paths()
        self.assertEqual(GENERATOR.native_group_process_navigation_custody_files(), old)

    def test_existing150_definitions_defaults_inputs_and_finalizer_remain_frozen(self):
        oracle = finalizer.NativeGroupNavigationFinalizerGeneration()
        oracle.test_preserves150_existing_definitions_and_all_historical_outputs()
        oracle.test_pinned_execution_entry_locks_replay_and_common_postchecks()
        self.assertNotIn(OUTPUT, GENERATOR.generated_files())

    def test_refuses_changed_original_hash_missing_extra_and_nonidentical_output_pair(self):
        self.files()
        old = historical.NativeGroupNavigationCustodyGeneration().custody()
        ops, app = historical.NAVIGATION_CLASSIFIER, historical.NAVIGATION_APP_CLASSIFIER
        cases = {
            'missing_app': {ops: old[ops]},
            'missing_ops': {app: old[app]},
            'extra': {**old, OUTPUT: old[ops]},
            'nonidentical': {**old, app: old[app] + '\n'},
            'changed_original_hash': {name: text + '\n' for name, text in old.items()},
        }
        for fault, files in cases.items():
            with self.subTest(fault=fault), patch.object(
                    GENERATOR, 'native_group_process_navigation_custody_files', return_value=files):
                with self.assertRaises((SystemExit, ValueError)):
                    self.entry()()
        self.files()

    def test_delegated_bad_historical_inputs_pins_and_phase_pairs_are_refused(self):
        self.files()
        migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
        self.assertEqual(len(migrations), 231)
        names = [historical.CAPTURE, historical.LEDGER, historical.NAVIGATION_SOURCE,
                 str(migrations[0].relative_to(ROOT)), str(migrations[-1].relative_to(ROOT))]
        original = GENERATOR.company_provenance_regular_path
        for name in names:
            with self.subTest(input=name), tempfile.TemporaryDirectory() as temporary:
                changed = Path(temporary) / 'changed-input'
                changed.write_bytes((ROOT / name).read_bytes() + b'\n')
                def regular(requested, *, required):
                    if requested == name:
                        self.assertTrue(required)
                        return changed
                    return original(requested, required=required)
                with patch.object(GENERATOR, 'company_provenance_regular_path', side_effect=regular):
                    with self.assertRaises((SystemExit, ValueError)):
                        self.entry()()
        with patch.object(GENERATOR, 'NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256', '0' * 64):
            with self.assertRaises((SystemExit, ValueError)):
                self.entry()()
        plain, observer = historical.NAVIGATION_PAIRS
        pairs = [(), (observer, plain),
                 (('plain', plain[1], observer[2]), ('observer', observer[1], plain[2])),
                 (('plain', plain[1], '0' * 64), observer)]
        for value in pairs:
            with self.subTest(pairs=value), patch.object(
                    GENERATOR, 'NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS', value):
                with self.assertRaises((SystemExit, ValueError)):
                    self.entry()()
        self.files()

    def test_cli_creates_only_new_leaf_checks_exact_bytes_and_restores_independently(self):
        files = self.files()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / OUTPUT
            target.parent.mkdir(parents=True)
            regular = GENERATOR.company_provenance_regular_path
            with patch.object(GENERATOR, 'ROOT', root), \
                    patch.object(GENERATOR, ENTRY, return_value=files) as selected, \
                    patch.object(GENERATOR, 'company_provenance_regular_path', wraps=regular) as paths, \
                    patch.object(GENERATOR, 'generated_files', side_effect=AssertionError('no defaults')), \
                    patch.object(GENERATOR, 'native_group_process_navigation_custody_files',
                                 side_effect=AssertionError('no historical output selection')):
                before = inventory(root)
                self.refused_without_writes(root, MODE, '--check')
                self.cli(MODE)
                self.assertEqual(target.read_bytes(), files[OUTPUT].encode())
                self.assertEqual(set(inventory(root)) - set(before), {OUTPUT})
                for fault in ('changed', 'missing'):
                    if fault == 'changed':
                        target.write_bytes(files[OUTPUT].encode() + b'\n')
                    else:
                        target.unlink()
                    self.refused_without_writes(root, MODE, '--check')
                    self.cli(MODE)
                    self.assertEqual(target.read_bytes(), files[OUTPUT].encode())
                    current = inventory(root)
                    self.cli(MODE, '--check')
                    self.assertEqual(inventory(root), current)
                self.assertEqual(selected.call_count, paths.call_count)
                for call in paths.call_args_list:
                    self.assertEqual(call.args, (OUTPUT,))
                    self.assertEqual(call.kwargs, {'required': False})

    def test_cli_refuses_symlink_nonregular_leaves_and_invalid_parents_in_both_modes(self):
        files = self.files()
        for fault in ('symlink_leaf', 'dangling_leaf', 'directory_leaf',
                      'missing_parent', 'symlink_parent', 'file_parent'):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                target = root / OUTPUT
                target.parent.parent.mkdir(parents=True)
                parent_fault = fault.endswith('_parent')
                broken = target.parent if parent_fault else target
                if not parent_fault:
                    target.parent.mkdir()
                if fault in ('symlink_leaf', 'symlink_parent'):
                    retained = root / 'retained'
                    if parent_fault:
                        retained.mkdir()
                    else:
                        retained.write_bytes(files[OUTPUT].encode())
                    broken.symlink_to(retained)
                elif fault == 'dangling_leaf':
                    broken.symlink_to(root / 'absent')
                elif fault == 'directory_leaf':
                    broken.mkdir()
                elif fault == 'file_parent':
                    broken.write_bytes(b'parent is not a directory')
                with patch.object(GENERATOR, 'ROOT', root), \
                        patch.object(GENERATOR, ENTRY, return_value=files):
                    self.refused_without_writes(root, MODE)
                    self.refused_without_writes(root, MODE, '--check')
                    if broken.is_symlink() or broken.is_file():
                        broken.unlink()
                    elif broken.exists():
                        broken.rmdir()
                    if parent_fault:
                        target.parent.mkdir()
                    self.cli(MODE)
                    self.assertEqual(target.read_bytes(), files[OUTPUT].encode())
                    current = inventory(root)
                    self.cli(MODE, '--check')
                    self.assertEqual(inventory(root), current)

    def test_cli_rejects_unsupported_combinations_and_preserves_default_dispatch(self):
        files = self.files()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            legacy = 'ops/historical.sql'
            (root / 'ops').mkdir()
            with patch.object(GENERATOR, 'ROOT', root), \
                    patch.object(GENERATOR, ENTRY, return_value=files) as selected, \
                    patch.object(GENERATOR, 'generated_files', return_value={legacy: 'historical\n'}) as defaults:
                for arguments in [('--check', MODE), (MODE, '--check', '--check'),
                                  (MODE, '--unknown'), (MODE, MODE),
                                  (MODE, '--native-group-process-navigation-custody')]:
                    with self.subTest(arguments=arguments):
                        self.refused_without_writes(root, *arguments)
                selected.assert_not_called()
                defaults.assert_not_called()
                self.cli()
                self.assertEqual((root / legacy).read_bytes(), b'historical\n')
                current = inventory(root)
                self.cli('--check')
                self.assertEqual(inventory(root), current)
                self.assertEqual(defaults.call_count, 2)
                selected.assert_not_called()


if __name__ == '__main__':
    unittest.main()
