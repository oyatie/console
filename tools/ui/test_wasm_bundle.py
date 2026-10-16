"""Bundle integrity controls. Synthetic bindgen is not native-build evidence."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('bundle', SOURCE / 'tools/ui/wasm_bundle.py')
bundle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bundle)

class IntegrityHelperTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='hydration-helper-unit-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.checkout = self.root / 'checkout'
        self.checkout.mkdir()
        for name in bundle.RECIPES + [bundle.SRC + '/lib.rs']:
            path = self.checkout / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('fixture input: ' + name + '\n')
        (self.checkout / 'rust-toolchain.toml').write_text('[toolchain]\nchannel = "1.98.1"\n')
        (self.checkout / bundle.CRATE / 'Cargo.toml').write_text('[dependencies]\nwasm-bindgen = { version = "0.2.123" }\n')
        subprocess.run(['git','init','-q',str(self.checkout)],check=True)
        subprocess.run(['git','-C',str(self.checkout),'add','-A'],check=True)
        self.inputs = bundle.checkout_inputs(self.checkout, strict=True)
        generation={name:bundle.file_hash(self.inputs[name]) for name in bundle.GENERATION_INPUTS}
        (self.checkout/'third-party/rust/hydrate/input-lock.json').write_text(json.dumps({'inputs':generation}))
        self.snapshot = self.root / 'snapshot'
        bundle.snapshot(self.inputs, self.snapshot)
        self.wasm = self.root / 'compiled.wasm'
        self.wasm.write_bytes(b'\0asm\x01\0\0\0unit-fixture-v1')
        self.tool = self.root / 'wasm-bindgen'
        self.tool.write_text('#!' + sys.executable + '\n' + '''import sys
from pathlib import Path
if sys.argv[1:] == ['--version']:
    print('wasm-bindgen 0.2.123')
else:
    out=Path(sys.argv[sys.argv.index('--out-dir')+1])
    payload=Path(sys.argv[1]).read_bytes()
    (out/'console_payroll_ui.js').write_text('// fixture '+payload.hex())
    (out/'console_payroll_ui_bg.wasm').write_bytes(payload)
''')
        self.tool.chmod(0o755)

    def produce(self, name='candidate'):
        candidate=self.root/name
        bundle.produce(self.snapshot,self.wasm,self.tool,candidate)
        return candidate

    def outputs(self, directory):
        return {name: directory/Path(name).name for name in bundle.OUTPUTS}

    def test_producer_records_copied_snapshot_not_mutable_checkout(self):
        original=(self.snapshot / bundle.SRC / 'lib.rs').read_bytes()
        (self.checkout / bundle.SRC / 'lib.rs').write_text('changed after snapshot')
        candidate=self.produce()
        record=bundle.load_manifest(candidate/'bundle.lock.json')
        self.assertEqual(hashlib.sha256(original).hexdigest(),record['inputs'][bundle.SRC+'/lib.rs'])
        self.assertEqual(bundle.file_hash(self.wasm),record['producer']['compiled_wasm_sha256'])
        self.assertEqual(bundle.file_hash(self.tool),record['producer']['bindgen_executable_sha256'])
        self.assertTrue(any('lib.rs has changed' in error for error in bundle.verify(record,self.inputs,self.outputs(candidate))))

    def test_snapshot_mutation_is_rejected_before_publication(self):
        (self.snapshot / bundle.SRC / 'lib.rs').write_text('corruption')
        with self.assertRaisesRegex(ValueError,'immutable compiler source snapshot was changed'):
            self.produce()
        self.assertFalse((self.root/'candidate').exists())

    def test_stale_dependency_projection_is_rejected_before_snapshot(self):
        (self.checkout/'backend/Cargo.toml').write_text('changed dependency root')
        fresh=self.root/'fresh-snapshot'
        with self.assertRaisesRegex(ValueError,'hydration dependency metadata is stale'):
            bundle.snapshot(self.inputs,fresh)
        self.assertFalse(fresh.exists())

    def test_wrong_tool_version_is_rejected(self):
        self.tool.write_text(self.tool.read_text().replace('0.2.123','0.2.99'))
        with self.assertRaisesRegex(ValueError,'bindgen executable version'):
            self.produce()
        self.assertFalse((self.root/'candidate').exists())

    def test_bindgen_failure_never_emits_manifest(self):
        self.tool.write_text(self.tool.read_text().replace("out=Path(sys.argv", "raise SystemExit(19)\n    out=Path(sys.argv"))
        with self.assertRaises(subprocess.CalledProcessError):
            self.produce()
        self.assertFalse((self.root/'candidate/bundle.lock.json').exists())

    def test_mixed_outputs_fail_integrity(self):
        candidate=self.produce()
        record=bundle.load_manifest(candidate/'bundle.lock.json')
        self.assertEqual([],bundle.verify(record,self.inputs,self.outputs(candidate)))
        (candidate/'console_payroll_ui.js').write_text('mixed revision')
        self.assertTrue(any('does not match the hash recorded' in error for error in bundle.verify(record,self.inputs,self.outputs(candidate))))

    def test_publish_checks_current_sources_before_any_write(self):
        candidate=self.produce()
        output=self.checkout/bundle.OUTPUTS[0]
        output.parent.mkdir(parents=True)
        output.write_bytes(b'previous')
        (self.checkout/bundle.SRC/'lib.rs').write_text('unbuilt')
        with self.assertRaisesRegex(ValueError,'lib.rs has changed'):
            bundle.publish(candidate,self.checkout)
        self.assertEqual(b'previous',output.read_bytes())
        self.assertFalse((self.checkout/bundle.MANIFEST).exists())

    def test_interrupted_publication_preserves_manifest_and_rejects_mixed_pair(self):
        (self.checkout/bundle.OUTPUTS[0]).parent.mkdir(parents=True)
        candidate=self.produce()
        bundle.publish(candidate,self.checkout)
        manifest=(self.checkout/bundle.MANIFEST).read_bytes()
        self.wasm.write_bytes(b'\0asm\x01\0\0\0unit-fixture-v2')
        next_candidate=self.produce('next')
        copy=shutil.copyfile
        copied=[]
        def interrupted(source,target):
            copied.append(str(target))
            if len(copied)==2:
                raise OSError('injected interruption')
            return copy(source,target)
        with patch.object(bundle.shutil,'copyfile',interrupted):
            with self.assertRaisesRegex(OSError,'injected interruption'):
                bundle.publish(next_candidate,self.checkout)
        self.assertEqual(manifest,(self.checkout/bundle.MANIFEST).read_bytes())
        self.assertTrue(bundle.verify(json.loads(manifest),self.inputs,{name:self.checkout/name for name in bundle.OUTPUTS}))

    def test_validator_copies_the_exact_native_action_pair(self):
        candidate=self.produce()
        validated=self.root/'validated'
        bundle.validate(candidate/'bundle.lock.json',self.inputs,self.outputs(candidate),candidate,validated)
        for name,path in self.outputs(candidate).items():
            self.assertEqual(path.read_bytes(),(validated/Path(name).name).read_bytes())

    def test_equal_native_outputs_accept_distinct_publisher_host_provenance(self):
        candidate=self.produce()
        published=self.root/'published-other-host'
        shutil.copytree(candidate,published)
        record=bundle.load_manifest(published/'bundle.lock.json')
        record['producer']['bindgen_executable_sha256']='b'*64
        record['producer']['compiled_wasm_sha256']='c'*64
        (published/'bundle.lock.json').write_text(json.dumps(record))
        self.assertEqual([],bundle.verify(record,self.inputs,self.outputs(published)))
        validated=self.root/'validated-other-host'
        bundle.validate(published/'bundle.lock.json',self.inputs,self.outputs(published),candidate,validated)
        for name,path in self.outputs(candidate).items():
            self.assertEqual(path.read_bytes(),(validated/Path(name).name).read_bytes())
        self.assertEqual('b'*64,bundle.load_manifest(published/'bundle.lock.json')['producer']['bindgen_executable_sha256'],
                         'validation must preserve the publisher provenance record')

    def test_validator_rejects_self_consistent_forged_bundle_before_output(self):
        candidate=self.produce()
        forged=self.root/'forged'
        shutil.copytree(candidate,forged)
        (forged/'console_payroll_ui.js').write_text('// old or manually altered output')
        record=bundle.load_manifest(forged/'bundle.lock.json')
        record['outputs']=bundle.hashes(self.outputs(forged))
        record['producer']['compiled_wasm_sha256']='a'*64
        (forged/'bundle.lock.json').write_text(json.dumps(record))
        self.assertEqual([],bundle.verify(record,self.inputs,self.outputs(forged)),
                         'fixture must pass self-attested checks to isolate native-output binding')
        validated=self.root/'validated'
        with self.assertRaisesRegex(ValueError,'does not match the native producer action'):
            bundle.validate(forged/'bundle.lock.json',self.inputs,self.outputs(forged),candidate,validated)
        self.assertFalse(validated.exists())

    def test_standalone_write_refuses_and_preserves_manifest(self):
        (self.checkout/bundle.OUTPUTS[0]).parent.mkdir(parents=True)
        bundle.publish(self.produce(),self.checkout)
        manifest=(self.checkout/bundle.MANIFEST).read_bytes()
        result=subprocess.run([sys.executable,str(SOURCE/'tools/ui/wasm_bundle.py'),'check','--root',str(self.checkout),'--write'],capture_output=True,text=True)
        self.assertEqual(1,result.returncode)
        self.assertIn('requires an action-bound native producer bundle',result.stderr)
        self.assertEqual(manifest,(self.checkout/bundle.MANIFEST).read_bytes())

if __name__=='__main__':
    unittest.main()
