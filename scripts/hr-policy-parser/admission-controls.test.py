"""Fake-status-only controls of the actual fixed owner; no native or product proof."""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
import sys
sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent

def module(name, file):
    import sys
    spec = importlib.util.spec_from_file_location(name, file)
    value = importlib.util.module_from_spec(spec)
    sys.modules[name] = value
    spec.loader.exec_module(value)
    return value

qualification = module('hr_qualification', HERE / 'qualify-admission.py')
fixed_path = Path(os.environ['CONSOLE_HR_POLICY_FIXED_FANOUT'])
if hashlib.sha256(fixed_path.read_bytes()).hexdigest() != qualification.FIXED_SHA:
    raise ValueError('controls require the authenticated actual fixed shared owner')
owner = module('fixed_fanout_owner', fixed_path)
ITEM = {'lane': qualification.LANE, 'probe': qualification.PROBE, 'red_exit': 101}

class StatusOnlyControls(unittest.TestCase):
    def run_status(self, value, item=ITEM):
        calls = []
        def runner(*args, **kwargs):
            calls.append((args, kwargs))
            return SimpleNamespace(returncode=value)
        lane = owner.admit_lane(item, cwd=HERE, runner=runner)
        spec = owner.admit_spec([item], cwd=HERE, runner=runner)
        return lane, spec, calls

    def test_explicit101_accepts_only_actual_integer101(self):
        lane, spec, calls = self.run_status(101)
        self.assertIsNone(lane); self.assertEqual(spec, []); self.assertEqual(len(calls), 2)
        for status in [0, 1, 2, 32, 126, 127, 128, 130, 137, 139, 255, -9, -15, True, 101.0]:
            with self.subTest(status=status):
                lane, spec, calls = self.run_status(status)
                self.assertIsNotNone(lane); self.assertEqual(len(spec), 1)

    def test_malformed_present_red_exit_refuses_before_probe(self):
        for value in [None, True, 101.0, '101', 0, 126, -9]:
            lane, spec, calls = self.run_status(101, {**ITEM, 'red_exit': value})
            self.assertIsNotNone(lane); self.assertEqual(len(spec), 1); self.assertEqual(calls, [])
        lane, spec, _ = self.run_status(101, {**ITEM, 'red_exit': 100})
        self.assertIsNotNone(lane); self.assertEqual(len(spec), 1)

    def test_shared_optional_defaults_and_empty_spec_remain_compatible(self):
        item = dict(ITEM); del item['red_exit']
        self.assertIsNone(self.run_status(1, item)[0])
        self.assertEqual(owner.admit_spec([], cwd=HERE, runner=lambda *_a, **_k: self.fail('empty generic spec executes no probe')), [])

class OperationalQualificationControls(unittest.TestCase):
    def test_required_one_item101_binding(self):
        self.assertEqual(qualification.qualify_spec([ITEM]), ITEM)
        for spec in [[], {}, [ITEM, ITEM], [None], [{'lane': ITEM['lane'], 'probe': ITEM['probe']}]]:
            with self.subTest(spec=spec):
                with self.assertRaises(ValueError): qualification.qualify_spec(spec)
        for key, values in {'red_exit': [None, True, 101.0, '101', 1, 0, 126], 'lane': ['', 'other'], 'probe': ['', 'node --test child.mjs', 'missing']}.items():
            for value in values:
                with self.assertRaises(ValueError): qualification.qualify_spec([{**ITEM, key: value}])

    def test_exact_invocation_binding_before_any_admission_call(self):
        b = {'cwd': str(qualification.ROOT), 'python': {'path': '/pinned/python'}, 'fixed_tool': {'path': '/fixed/tools/lanes/fanout.py', 'sha256': qualification.FIXED_SHA, 'commit': qualification.FIXED_COMMIT}, 'spec': {'path': '/reviewed/spec.json'}, 'spec_value': [ITEM]}
        command = ['/pinned/python', '/fixed/tools/lanes/fanout.py', 'admit', '--spec', '/reviewed/spec.json']
        self.assertEqual(qualification.qualify_invocation(b, command, b['cwd']), command)
        for bad in [command[:2] + ['admit'], command[:2] + ['admit', '--probe', ITEM['probe']], command[:2] + ['run', '--spec', '/reviewed/spec.json'], ['/other/python'] + command[1:], command[:1] + ['/wrong/tool'] + command[2:]]:
            with self.assertRaises(ValueError): qualification.qualify_invocation(b, bad, b['cwd'])
        with self.assertRaises(ValueError): qualification.qualify_invocation(b, command, '/wrong/cwd')
        for key in ['commit', 'sha256']:
            bad = copy.deepcopy(b); bad['fixed_tool'][key] = 'wrong'
            with self.assertRaises(ValueError): qualification.qualify_invocation(bad, command, b['cwd'])

if __name__ == '__main__':
    unittest.main(verbosity=2)
