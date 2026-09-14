"""Local oracle controls; no Kubernetes access or production-authority bypass."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parent

class ExistingDrillRegression(unittest.TestCase):
    def test_drill_refuses_success_without_independent_expected_state(self):
        source = (ROOT / 'cnpg-restore-drill.sh').read_text()
        verification = source.split('# --- verification ', 1)[1].split('\n', 1)[1]
        # Run only the verification code with synthetic command output. This is
        # a control of the real script, not a live CNPG restore or deployment.
        fake = '''
set -euo pipefail
script_dir="$1"
scratch_namespace=synthetic
recovery_cluster=synthetic
database=console
target_time='2026-09-14 12:00:00+00'
verification_manifest=''
kubectl() {
  case "$*" in
    *pg_is_in_recovery*) echo f ;;
    *'count(*)'*) echo 1 ;;
    *pg_last_committed_xact*) echo n/a ;;
    *exec*) echo 'public.lost_payroll : 0' ;;
    *) echo synthetic-primary ;;
  esac
}
'''
        result = subprocess.run(['bash', '-c', fake + verification, 'oracle-test', str(ROOT)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertNotIn('cnpg_restore_drill_complete=ok', result.stdout)




def oracle():
    spec = importlib.util.spec_from_file_location('recovery_manifest', ROOT / 'recovery-manifest.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ManifestControls(unittest.TestCase):
    def setUp(self):
        self.module = oracle()
        self.header = {'database': 'console', 'database_oid': '42', 'system_identifier': '1234', 'in_recovery': False}
        self.table = {'table': ['public', 'payroll'], 'columns': [['amount', 'bigint', True]]}
        self.hashes = ['1' * 64, '2' * 64]
        self.expected = dict(self.header, tables=[dict(self.table, rows=2, sha256=hashlib.sha256(('1' * 64 + '\n' + '2' * 64 + '\n').encode()).hexdigest())])

    def stream(self, hashes=None, header=None, table=None, complete=True):
        return iter([json.dumps(header or self.header) + '\n', json.dumps(table or self.table) + '\n', *[h + '\n' for h in (self.hashes if hashes is None else hashes)], *(['{"complete":true}\n'] if complete else [])])

    def test_independent_positive_content_and_count(self):
        self.assertEqual(self.module.collect(self.stream()), self.expected)

    def test_missing_row_changes_oracle(self):
        self.assertNotEqual(self.module.collect(self.stream(hashes=self.hashes[:1])), self.expected)

    def test_same_count_corruption_changes_oracle(self):
        self.assertNotEqual(self.module.collect(self.stream(hashes=['1' * 64, '3' * 64])), self.expected)

    def test_duplicate_row_changes_oracle(self):
        self.assertNotEqual(self.module.collect(self.stream(hashes=['1' * 64, '1' * 64])), self.expected)

    def test_pitr_overshoot_extra_row_changes_oracle(self):
        self.assertNotEqual(self.module.collect(self.stream(hashes=[*self.hashes, '3' * 64])), self.expected)

    def test_empty_table_is_not_empty_database(self):
        value = self.module.collect(self.stream(hashes=[]))
        self.assertEqual(value['tables'][0]['rows'], 0)
        self.assertEqual(value['tables'][0]['sha256'], hashlib.sha256(b'').hexdigest())
        with self.assertRaises(ValueError):
            self.module.collect(iter([json.dumps(self.header), '{"complete":true}']))

    def test_missing_completion_refused_even_after_all_expected_rows(self):
        with self.assertRaises(ValueError):
            self.module.collect(self.stream(complete=False))

    def test_output_after_completion_refused(self):
        with self.assertRaises(ValueError):
            self.module.collect(iter([*self.stream(), '1' * 64]))

    def test_duplicate_table_refused(self):
        rows = list(self.stream())
        rows.insert(-1, json.dumps(self.table))
        with self.assertRaises(ValueError):
            self.module.collect(iter(rows))

    def test_unsorted_and_nonhash_output_refused(self):
        for hashes in [list(reversed(self.hashes)), ['private payroll value'], ['1' * 63]]:
            with self.subTest(hashes=hashes), self.assertRaises(ValueError):
                self.module.collect(self.stream(hashes=hashes))

    def test_replica_still_in_recovery_refused(self):
        with self.assertRaises(ValueError):
            self.module.collect(self.stream(header=dict(self.header, in_recovery=True)))

    def test_wrong_database_or_physical_cluster_changes_oracle(self):
        for key in ['database', 'database_oid', 'system_identifier']:
            self.assertNotEqual(self.module.collect(self.stream(header=dict(self.header, **{key: ('different' if key == 'database' else '99')}))), self.expected)

    def test_column_change_detected_even_without_rows(self):
        self.assertNotEqual(self.module.collect(self.stream(hashes=[])), self.module.collect(self.stream(hashes=[], table=dict(self.table, columns=[['amount', 'text', True]]))))

    def test_cli_reference_binding_and_failure_nondisclosure(self):
        import tempfile
        with tempfile.TemporaryDirectory() as tmp:
            expected = Path(tmp) / 'expected.json'
            expected.write_text(json.dumps({'version': 1, 'target_time': '2026-09-14 12:00:00+00', 'state': self.expected}))
            args = [sys.executable, str(ROOT / 'recovery-manifest.py'), 'verify', '--expected', str(expected), '--database', 'console', '--target-time', '2026-09-14 12:00:00+00']
            nominal = subprocess.run(args, input=''.join(self.stream()), capture_output=True, text=True)
            self.assertEqual(nominal.returncode, 0, nominal.stderr)
            self.assertIn('verify_pitr_state=match', nominal.stdout)
            for stream in [self.stream(hashes=['sensitive-wage-value']), self.stream(hashes=self.hashes[:1])]:
                bad = subprocess.run(args, input=''.join(stream), capture_output=True, text=True)
                self.assertNotEqual(bad.returncode, 0)
                self.assertEqual(bad.stderr, 'recovery_verification=failed\n')
                self.assertEqual(bad.stdout, '')
            for index, replacement in [(-1, 'different-time'), (-3, 'different-db')]:
                changed = list(args); changed[index] = replacement
                bad = subprocess.run(changed, input=''.join(self.stream()), capture_output=True, text=True)
                self.assertNotEqual(bad.returncode, 0)

class HarnessCustodyControls(unittest.TestCase):
    def test_lost_create_reply_cleans_only_predeclared_owner(self):
        self.check_cleanup('lost-create')

    def test_unrelated_name_collision_never_removed(self):
        self.check_cleanup('unrelated-owner')

    def test_failed_removal_still_inspected(self):
        self.check_cleanup('failed-removal')

    def test_timed_out_removal_still_inspected(self):
        self.check_cleanup('timed-out-removal')

    def check_cleanup(self, scenario):
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location('harness', ROOT / 'recovery-manifest.integration.test.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        calls = []
        name = None
        def fake(*args, **kwargs):
            nonlocal name
            calls.append(args)
            if args[1] == 'create':
                name = args[args.index('--name') + 1]
                self.assertIn('console.recovery.run=' + name, args)
                raise subprocess.TimeoutExpired(args, 30)
            if args[1] == 'inspect' and '--format' in args:
                return subprocess.CompletedProcess(args, 0, json.dumps({'console.recovery.run': 'unrelated' if scenario == 'unrelated-owner' else name}), '')
            if args[1] == 'rm':
                self.assertEqual(args[-1], name)
                if scenario == 'timed-out-removal':
                    raise subprocess.TimeoutExpired(args, 30)
                return subprocess.CompletedProcess(args, 1 if scenario == 'failed-removal' else 0, '', '')
            if args[1] == 'inspect':
                return subprocess.CompletedProcess(args, 1, '[]', 'Error response from daemon: No such container: ' + name)
            return subprocess.CompletedProcess(args, 0, '', '')
        with patch.dict('os.environ', {'DOCKER_CONTEXT': 'synthetic'}), patch.object(module, 'run', fake), self.assertRaises((subprocess.TimeoutExpired, AssertionError)):
            module.main()
        if scenario == 'unrelated-owner':
            self.assertFalse(any(args[1] == 'rm' for args in calls))
        else:
            self.assertEqual(calls[-1][1:4], ('inspect', '--type', 'container'))
            self.assertEqual(calls[-1][-1], name)


if __name__ == '__main__':
    unittest.main()
