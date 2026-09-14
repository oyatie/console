"""Local oracle controls; no Kubernetes access or production-authority bypass."""
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
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


if __name__ == '__main__':
    unittest.main()
