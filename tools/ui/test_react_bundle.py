"""Corruption controls for the new React producer; no business provisioning."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('react_bundle', Path(__file__).with_name('react_bundle.py'))
bundle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bundle)


class ReactCustodyTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.source = self.root / 'source.tsx'
        self.source.write_text('export const name = "한글";')
        self.inputs = {'clients/desktop-web/src/people.tsx': self.source}
        self.outputs = {}
        for name in bundle.OUTPUTS:
            path = self.root / name
            path.write_text(name)
            self.outputs[name] = path
        self.lock = {'format': 1, 'versions': {'node': '26.7.0', 'esbuild': '0.28.2', 'typescript': '5.9.3'},
                     'hosts': {'test-host': {'node': {'executable_sha256': 'a' * 64},
                                             'esbuild': {'executable_sha256': 'b' * 64}}}, 'packages': []}
        self.check = self.root / 'typecheck.tsbuildinfo'
        self.check.write_text(json.dumps({'version': '5.9.3'}))
        self.record = {'format': 1, 'inputs': bundle.hashes(self.inputs),
                       'outputs': bundle.hashes(self.outputs),
                       'dependency_lock_sha256': bundle.digest(bundle.canonical(self.lock)),
                       'producer': {'kind': 'buck2-native-react-people-v1', 'host': 'test-host',
                                    'tools': {'node': 'a' * 64, 'esbuild': 'b' * 64},
                                    'versions': self.lock['versions'],
                                    'typecheck_sha256': bundle.file_hash(self.check)}}

    def check_record(self, record=None, check=True):
        return bundle.verify(self.record if record is None else record, self.inputs, self.outputs,
                             self.lock, self.check if check else None)

    def test_positive_native_and_committed_records(self):
        self.assertEqual(self.check_record(), [])
        self.assertEqual(self.check_record(check=False), [])

    def test_output_edit_and_missing_output_are_detected(self):
        for name, path in self.outputs.items():
            with self.subTest(name=name):
                original = path.read_bytes()
                path.write_bytes(original + b'corruption')
                self.assertTrue(self.check_record())
                path.unlink()
                self.assertTrue(self.check_record())
                path.write_bytes(original)

    def test_input_edit_missing_and_omission_are_detected(self):
        original = self.source.read_bytes()
        self.source.write_bytes(original + b'corruption')
        self.assertTrue(self.check_record())
        self.source.unlink()
        self.assertTrue(self.check_record())
        self.source.write_bytes(original)
        del self.record['inputs'][next(iter(self.inputs))]
        self.assertTrue(self.check_record())

    def test_extra_input_and_output_are_detected(self):
        for key in ['inputs', 'outputs']:
            with self.subTest(key=key):
                record = copy.deepcopy(self.record)
                record[key]['unknown'] = 'c' * 64
                self.assertTrue(self.check_record(record))

    def test_dependency_and_host_tool_pin_changes_are_detected(self):
        for mutate in [lambda r: r.update(dependency_lock_sha256='c' * 64),
                       lambda r: r['producer'].update(host='unknown'),
                       lambda r: r['producer']['tools'].update(node='c' * 64),
                       lambda r: r['producer']['versions'].update(esbuild='0.0.0'),
                       lambda r: r['producer'].update(kind='npm-wrapper'),
                       lambda r: r['producer'].update(typecheck_sha256='')]:
            record = copy.deepcopy(self.record)
            mutate(record)
            self.assertTrue(self.check_record(record))

    def test_typecheck_corruption_missing_and_wrong_version_are_detected(self):
        self.check.write_text(json.dumps({'version': '0.0.0'}))
        self.record['producer']['typecheck_sha256'] = bundle.file_hash(self.check)
        self.assertTrue(self.check_record())
        self.check.unlink()
        self.assertTrue(self.check_record())

    def test_closed_record_shape_is_required(self):
        for invalid in [None, [], 1, {}, {**self.record, 'unreviewed': True}]:
            with self.subTest(invalid=type(invalid).__name__):
                self.assertTrue(bundle.verify(invalid, self.inputs, self.outputs, self.lock, self.check))



    def test_nested_shape_wrong_types_and_missing_fields_are_rejected(self):
        mutations = [lambda r: r.update(format=True),
                     lambda r: r.update(format=False),
                     lambda r: r.update(inputs=[]),
                     lambda r: r.update(outputs=[]),
                     lambda r: r.update(producer=[]),
                     lambda r: r['producer'].update(extra=True),
                     lambda r: r['producer']['tools'].update(extra='c' * 64),
                     lambda r: r['producer']['versions'].update(extra='1'),
                     lambda r: r['producer'].update(tools=[]),
                     lambda r: r['producer'].update(versions=[]),
                     lambda r: r['producer'].update(host=[]),
                     lambda r: r['producer'].update(typecheck_sha256=123),
                     lambda r: r['inputs'].update({next(iter(self.inputs)): 123})]
        for mutation in mutations:
            record = copy.deepcopy(self.record)
            mutation(record)
            self.assertTrue(self.check_record(record))
        for key in self.record:
            record = copy.deepcopy(self.record)
            del record[key]
            self.assertTrue(self.check_record(record))
        for key in self.record['producer']:
            record = copy.deepcopy(self.record)
            del record['producer'][key]
            self.assertTrue(self.check_record(record))

    def native_candidate(self):
        candidate = self.root / 'native'
        candidate.mkdir()
        for name, path in self.outputs.items():
            (candidate / name).write_bytes(path.read_bytes())
        (candidate / 'typecheck.tsbuildinfo').write_bytes(self.check.read_bytes())
        (candidate / 'bundle.lock.json').write_text(json.dumps(self.record))
        return candidate

    def committed_directory(self):
        destination = self.root / bundle.COMMITTED
        destination.mkdir(parents=True)
        for name, path in self.outputs.items():
            (destination / name).write_bytes(path.read_bytes())
        (destination / 'bundle.lock.json').write_text(json.dumps(self.record))
        return destination

    def test_native_comparison_rejects_self_consistent_forgery_and_absent_typecheck(self):
        candidate = self.native_candidate()
        committed = self.committed_directory()
        bundle.validate_pair(candidate, committed, self.inputs, self.lock)
        name = bundle.OUTPUTS[0]
        (committed / name).write_text('forged but self-consistent')
        record = copy.deepcopy(self.record)
        record['outputs'][name] = bundle.file_hash(committed / name)
        (committed / 'bundle.lock.json').write_text(json.dumps(record))
        with self.assertRaises(ValueError):
            bundle.validate_pair(candidate, committed, self.inputs, self.lock)
        (candidate / 'typecheck.tsbuildinfo').unlink()
        with self.assertRaises(ValueError):
            bundle.validate_native(candidate, self.inputs, self.lock)

    def test_invalid_typecheck_json_and_extra_candidate_file_are_rejected(self):
        candidate = self.native_candidate()
        check = candidate / 'typecheck.tsbuildinfo'
        check.write_text('{not-json')
        record = copy.deepcopy(self.record)
        record['producer']['typecheck_sha256'] = bundle.file_hash(check)
        (candidate / 'bundle.lock.json').write_text(json.dumps(record))
        with self.assertRaises(ValueError):
            bundle.validate_native(candidate, self.inputs, self.lock)
        check.write_bytes(self.check.read_bytes())
        (candidate / 'bundle.lock.json').write_text(json.dumps(self.record))
        (candidate / 'unexpected.map').write_text('undeclared')
        with self.assertRaises(ValueError):
            bundle.validate_native(candidate, self.inputs, self.lock)

    def previous_publication(self):
        committed = self.committed_directory()
        for name in bundle.OUTPUTS:
            (committed / name).write_text('previous-' + name)
        old_record = copy.deepcopy(self.record)
        old_record['outputs'] = bundle.hashes({name: committed / name for name in bundle.OUTPUTS})
        (committed / 'bundle.lock.json').write_text(json.dumps(old_record))
        self.assertEqual(bundle.verify(old_record, self.inputs,
                                      {name: committed / name for name in bundle.OUTPUTS}, self.lock), [])
        return committed, old_record

    def test_successful_publication_replaces_old_output_with_verified_native_bytes(self):
        candidate = self.native_candidate()
        committed, _ = self.previous_publication()
        with patch.object(bundle, 'checkout_inputs', return_value=self.inputs), \
             patch.object(bundle, 'checkout_lock', return_value=self.lock):
            bundle.publish(candidate, self.root)
        for name in [*bundle.OUTPUTS, 'bundle.lock.json']:
            self.assertEqual((candidate / name).read_bytes(), (committed / name).read_bytes())
        bundle.validate_pair(candidate, committed, self.inputs, self.lock)

    def test_publication_rechecks_source_before_writes_and_after_copy(self):
        candidate = self.native_candidate()
        committed, old_record = self.previous_publication()
        old_bytes = {name: (committed / name).read_bytes() for name in [*bundle.OUTPUTS, 'bundle.lock.json']}
        old_manifest = old_bytes['bundle.lock.json']
        with patch.object(bundle, 'checkout_inputs', return_value=self.inputs), \
             patch.object(bundle, 'checkout_lock', return_value=self.lock):
            original = self.source.read_bytes()
            self.source.write_bytes(original + b'changed')
            with self.assertRaises(ValueError):
                bundle.publish(candidate, self.root)
            for name, original_bytes in old_bytes.items():
                self.assertEqual((committed / name).read_bytes(), original_bytes)
            self.source.write_bytes(original)
            real_copy = bundle.shutil.copyfile
            def change_source(src, dst):
                result = real_copy(src, dst)
                self.source.write_bytes(original + b'changed during publication')
                return result
            with patch.object(bundle.shutil, 'copyfile', side_effect=change_source):
                with self.assertRaises(ValueError):
                    bundle.publish(candidate, self.root)
            self.assertEqual((committed / 'bundle.lock.json').read_bytes(), old_manifest)

    def test_interrupted_publication_keeps_old_manifest_and_invalidates_mixed_outputs(self):
        candidate = self.native_candidate()
        committed, old_record = self.previous_publication()
        old_manifest = (committed / 'bundle.lock.json').read_bytes()
        with patch.object(bundle, 'checkout_inputs', return_value=self.inputs), \
             patch.object(bundle, 'checkout_lock', return_value=self.lock):
            real_copy = bundle.shutil.copyfile
            count = 0
            def interrupted(src, dst):
                nonlocal count
                count += 1
                if count == 2:
                    raise OSError('injected interrupted publication')
                return real_copy(src, dst)
            with patch.object(bundle.shutil, 'copyfile', side_effect=interrupted):
                with self.assertRaises(OSError):
                    bundle.publish(candidate, self.root)
        self.assertEqual((committed / 'bundle.lock.json').read_bytes(), old_manifest)
        self.assertEqual((committed / bundle.OUTPUTS[0]).read_bytes(), (candidate / bundle.OUTPUTS[0]).read_bytes())
        self.assertTrue(bundle.verify(old_record, self.inputs,
                                     {name: committed / name for name in bundle.OUTPUTS}, self.lock))
        with self.assertRaises(ValueError):
            bundle.validate_pair(candidate, committed, self.inputs, self.lock)


if __name__ == '__main__':
    unittest.main()
