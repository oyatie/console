"""Synthetic oracle controls; no Buck/compiler/bindgen/browser execution."""
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('candidate', HERE / 'test_native_hydration.py')
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)

class ActionBindingControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='hydration-action-control-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.evidence = self.root / 'evidence'; self.evidence.mkdir()
        self.emitted = 'output_artifacts/CPPL/product.so'
        self.consumed = 'cas/1234567890abcdef/CPPL/product.so'
        self.module = b'\0asm\x01\0\0\0synthetic-oracle-fixture'
        self.write(self.emitted, self.module)
        self.write(self.consumed, self.module)
        self.write('tools/wasm-bindgen', b'synthetic executable identity, never run')
        self.write('linker_wrapper.sh', b'synthetic linker identity, never run')
        self.write('compiler.args', ('\n'.join([
            '--crate-type=cdylib', '--target=wasm32-unknown-unknown',
            '--crate-name=console_payroll_ui', '--cfg=feature="hydrate"',
            '--cfg=feature="islands"', '-Copt-level=3',
            '--emit=link=' + self.emitted, '-Clinker=linker_wrapper.sh',
        ]) + '\n').encode())
        self.compiler = {'identity': 'root//backend/crates/payroll/ui:console-payroll-ui-hydrate (rustc cdylib [pic])',
                         'reproducer': {'details': {'command': ['synthetic-wrapper', '@compiler.args']}}}
        self.bindgen = {'identity': 'root//backend/crates/payroll/ui:console-payroll-ui-wasm-bundle (wasm_bindgen)',
                       'reproducer': {'details': {'command': ['python', 'helper.py', 'produce', '--input',
                           self.consumed, '--bindgen', 'tools/wasm-bindgen', '--out-dir', 'bundle']}}}
        self.rows = [self.compiler, self.bindgen]
        for path in oracle.PAIR:
            payload = self.module if path.suffix == '.wasm' else b'// synthetic output pair'
            self.write('bundle/' + path.name, payload)
            self.write(str(path), payload)

    def write(self, name, data):
        path = self.root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(data)

    def inspect(self):
        def command(argv, *_args, **_kwargs):
            if argv[:3] == ['tools/buck2', 'log', 'what-ran']:
                return SimpleNamespace(returncode=0, stdout='\n'.join(json.dumps(x) for x in self.rows), stderr='')
            self.assertEqual(argv, [str(self.root / 'tools/wasm-bindgen'), '--version'])
            return SimpleNamespace(returncode=0, stdout='wasm-bindgen 0.2.123\n', stderr='')
        with patch.object(oracle, 'command', command):
            oracle.inspect_native_actions(self.root, self.evidence, {Path('synthetic-event-log')})

    def test_cas_relocation_accepts_full_identical_bytes_and_records_both_paths(self):
        self.inspect()
        receipt = json.loads((self.evidence / 'native-bindgen-actions.json').read_text())[0]
        self.assertNotEqual(self.emitted, self.consumed)
        self.assertEqual(self.consumed, receipt['linked_input'])
        self.assertEqual(self.emitted, receipt['compiler_output'])
        self.assertEqual(oracle.digest(self.root / self.emitted), receipt['linked_input_sha256'])
        self.assertEqual(64, len(receipt['linked_input_sha256']))

    def test_same_path_remains_accepted(self):
        argv = self.bindgen['reproducer']['details']['command']; argv[argv.index('--input') + 1] = self.emitted
        self.inspect()

    def test_mismatching_actual_input_rejects_even_with_emitted_path_elsewhere_in_argv(self):
        self.write(self.consumed, b'\0asm\x01\0\0\0different-module')
        self.bindgen['reproducer']['details']['command'] += ['--unrelated', self.emitted]
        with self.assertRaisesRegex(oracle.SemanticFailure, 'does not match a witnessed native compiler output'):
            self.inspect()

    def test_absent_compiler_action_remains_prerequisite_failure(self):
        self.rows = [self.bindgen]
        with self.assertRaisesRegex(oracle.Prerequisite, 'native WASM cdylib compiler action unavailable'):
            self.inspect()

    def test_absent_bindgen_action_remains_prerequisite_failure(self):
        self.rows = [self.compiler]
        with self.assertRaisesRegex(oracle.Prerequisite, 'bindgen action consuming the just-linked'):
            self.inspect()

    def test_missing_actual_input_selector_rejects(self):
        argv = self.bindgen['reproducer']['details']['command']; argv[argv.index('--input')] = '--other'
        with self.assertRaisesRegex(oracle.Prerequisite, 'exactly one actual --input'):
            self.inspect()

    def test_duplicate_actual_input_selector_rejects(self):
        self.bindgen['reproducer']['details']['command'] += ['--input', self.emitted]
        with self.assertRaisesRegex(oracle.Prerequisite, 'exactly one actual --input'):
            self.inspect()

    def test_non_wasm_consumed_bytes_reject(self):
        self.write(self.consumed, b'not-wasm')
        with self.assertRaisesRegex(oracle.SemanticFailure, 'actual bindgen input is not a WASM module'):
            self.inspect()

    def test_missing_consumed_file_rejects(self):
        (self.root / self.consumed).unlink()
        with self.assertRaisesRegex(oracle.Prerequisite, 'actual bindgen input file unavailable'):
            self.inspect()

    def test_changed_published_pair_still_rejects(self):
        self.write(str(oracle.PAIR[0]), b'// different published bytes')
        with self.assertRaisesRegex(oracle.SemanticFailure, 'published pair is not the actual bindgen'):
            self.inspect()

if __name__ == '__main__':
    unittest.main(verbosity=2)
