"""Uninstalled source export only; positive classification needs database proof."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path('ops/generate-account-custody.py')
SPEC = importlib.util.spec_from_file_location('provenance_generator', ROOT / SCRIPT)
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)

# Exact 33-name/hash map at approved base 7a96a8e14; never a successor fingerprint.
HISTORICAL_OUTPUTS_SHA256 = '63576789a001db496bd3b321b5308ddede0ac954d7c681a4d329fec88042637b'
CURRENT_OUTPUTS_SHA256 = '93a82a4c3e295d3218bb80353868b85c591a3aef522560173d99d4fb133e09a6'
SOURCES = (
    'ops/company-enrollment/provenance-v1.sql',
    'ops/company-enrollment/provenance-acl-v1.sql',
)
ARTIFACTS = {
    'ops/postgres-company-provenance-v1-owner.sql',
    'ops/postgres-capture-company-provenance-v1-custody.sql',
}


def inventory(root):
    entries = {}
    for path in root.rglob('*'):
        if path.is_symlink():
            identity = ('symlink', str(path.readlink()))
        elif path.is_file():
            identity = ('file', hashlib.sha256(path.read_bytes()).hexdigest())
        elif path.is_dir():
            identity = ('directory',)
        else:
            identity = ('special', path.lstat().st_mode)
        entries[str(path.relative_to(root))] = identity
    return entries


class CompanyProvenanceCaptureGeneration(unittest.TestCase):
    def test_cli_exports_only_uninstalled_successor(self):
        historical = GENERATOR.generated_files()
        frozen = {name: hashlib.sha256(content.encode()).hexdigest()
                  for name, content in historical.items()}
        self.assertEqual(len(frozen), 33)
        self.assertEqual(hashlib.sha256(json.dumps(
            frozen, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
            CURRENT_OUTPUTS_SHA256, 'current accepted names or bytes changed')
        # The additive 0231 Approval migration does not replace the historical
        # 230-record ledger. Pin both identities and all 32 other outputs.
        ledger_name = 'ops/account-custody-migrations.sha384'
        ledger = (ROOT / ledger_name).read_bytes()
        self.assertEqual(len(ledger.splitlines()), 231)
        self.assertEqual(hashlib.sha256(ledger).hexdigest(),
                         '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357')
        prefix = b''.join(ledger.splitlines(keepends=True)[:230])
        self.assertEqual(hashlib.sha256(prefix).hexdigest(),
                         '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325')
        original_frozen = {**frozen, ledger_name: hashlib.sha256(prefix).hexdigest()}
        self.assertEqual(hashlib.sha256(json.dumps(
            original_frozen, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
            HISTORICAL_OUTPUTS_SHA256, 'historical names or bytes changed')
        for name, expected in frozen.items():
            self.assertEqual(hashlib.sha256((ROOT / name).read_bytes()).hexdigest(),
                             expected, name)

        with tempfile.TemporaryDirectory(prefix='company-provenance-export-') as temporary:
            root = Path(temporary)
            inputs = {SCRIPT, *(Path(name) for name in historical)}
            for directory in ('ops', 'backend/crates/platform/db/migrations'):
                inputs.update(path.relative_to(ROOT)
                              for path in (ROOT / directory).rglob('*.sql')
                              if str(path.relative_to(ROOT)) not in ARTIFACTS)
            for name in inputs:
                source = ROOT / name
                self.assertFalse(source.is_symlink(), str(name))
                self.assertTrue(source.is_file(), str(name))
                target = root / name
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target)

            def cli(*arguments):
                return subprocess.run([sys.executable, str(root / SCRIPT), *arguments],
                                      cwd=root, capture_output=True, text=True, timeout=30)

            before = inventory(root)
            ordinary = cli('--check')
            self.assertEqual(ordinary.returncode, 0, ordinary.stderr)
            self.assertEqual(inventory(root), before, 'ordinary check wrote files')

            exported = cli('--company-provenance-capture')
            self.assertEqual(exported.returncode, 0,
                             'COMPANY_PROVENANCE_CLI_EXPORT_MISSING: ' + exported.stderr)
            after = inventory(root)
            self.assertEqual(set(after) - set(before), ARTIFACTS)
            self.assertEqual({name: after[name] for name in before}, before,
                             'export changed an existing source or historical artifact')
            self.assertEqual(cli('--company-provenance-capture').returncode, 0)
            self.assertEqual(inventory(root), after, 'export is not byte-idempotent')
            self.assertEqual(cli('--company-provenance-capture', '--check').returncode, 0)
            self.assertEqual(inventory(root), after, 'export check wrote files')
            self.assertEqual(cli().returncode, 0)
            self.assertEqual(cli('--check').returncode, 0)
            self.assertEqual(inventory(root), after, 'ordinary generation adopted successor')

            source = (root / 'ops/postgres-company-provenance-v1-owner.sql').read_text()
            # Input digest declarations are reviewed with the source candidate.
            # Database ABI/ACL/behavior are the separate approved native probe,
            # never inferred from comments or substrings in this export test.
            pins = GENERATOR.COMPANY_PROVENANCE_SOURCE_SHA256
            self.assertEqual(set(pins), set(SOURCES))
            for name in SOURCES:
                self.assertTrue((root / name).is_file(), name)
                self.assertRegex(pins[name], r'^[0-9a-f]{64}$')
                self.assertEqual(hashlib.sha256((root / name).read_bytes()).hexdigest(),
                                 pins[name], name)
            expected_source = ('-- Generated UNINSTALLED Company provenance source; not a custody finalizer.\n'
                               '-- No finalized profile or installation is authorized by this artifact.\n'
                               + '\n'.join('-- source: ' + name + '\n' + (root / name).read_text()
                                           for name in SOURCES))
            self.assertEqual(source, expected_source, 'export added or changed declared SQL')

            # Preserve the entire predecessor serializer. The extension selects
            # every signature/owner of the classifier and its private lock helper.
            historical_capture = (root / 'ops/postgres-capture-native-people-directory-custody.sql').read_text()
            self.assertTrue(historical_capture.endswith(';\n'))
            predecessor = historical_capture[:-2]
            anchor = '), owner_roles AS ('
            self.assertEqual(predecessor.count(anchor), 1)
            extended = predecessor.replace(anchor,
                " OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))\n" + anchor)
            captured = (root / 'ops/postgres-capture-company-provenance-v1-custody.sql').read_text()
            self.assertEqual(captured, extended + ';\n')

            # Check must detect drift without rewriting either artifact.
            for name in sorted(ARTIFACTS):
                with self.subTest(drift=name):
                    target = root / name
                    original = target.read_bytes()
                    target.write_bytes(original + b'\n')
                    drifted = inventory(root)
                    result = cli('--company-provenance-capture', '--check')
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('differs', result.stderr)
                    self.assertEqual(inventory(root), drifted)
                    target.write_bytes(original)
                    target.unlink()
                    missing = inventory(root)
                    result = cli('--company-provenance-capture', '--check')
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('differs', result.stderr)
                    self.assertEqual(inventory(root), missing, 'check recreated a missing artifact')
                    target.write_bytes(original)

            # Malicious source inputs must fail before publishing any output.
            originals = {name: (root / name).read_bytes() for name in SOURCES}
            for name in SOURCES:
                for fault in ('missing', 'changed', 'symlink'):
                    with self.subTest(source=name, fault=fault):
                        for artifact in ARTIFACTS:
                            (root / artifact).unlink(missing_ok=True)
                        target = root / name
                        original = originals[name]
                        target.unlink()
                        if fault == 'changed':
                            target.write_bytes(original + b'\n')
                        elif fault == 'symlink':
                            copied = root / 'exact-source.sql'
                            copied.write_bytes(original)
                            target.symlink_to(copied)
                        corrupt = inventory(root)
                        result = cli('--company-provenance-capture')
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('regular file' if fault != 'changed' else 'reviewed bytes',
                                      result.stderr)
                        self.assertEqual(inventory(root), corrupt, 'failed export wrote files')
                        self.assertFalse(any((root / artifact).exists() for artifact in ARTIFACTS))
                        target.unlink(missing_ok=True)
                        target.write_bytes(original)
                        (root / 'exact-source.sql').unlink(missing_ok=True)

            # Reject every invalid destination before publishing either artifact.
            for name in sorted(ARTIFACTS):
                for fault in ('symlink', 'dangling_symlink', 'directory'):
                    with self.subTest(output=name, fault=fault):
                        external = root / 'unrelated.sql'
                        target = root / name
                        if fault == 'directory':
                            target.mkdir()
                        else:
                            if fault == 'symlink':
                                expected = expected_source if name.endswith('-owner.sql') else extended + ';\n'
                                external.write_text(expected)
                            target.symlink_to(external)
                        unchanged = inventory(root)
                        result = cli('--company-provenance-capture')
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('regular file', result.stderr)
                        self.assertEqual(inventory(root), unchanged)
                        if fault == 'directory':
                            target.rmdir()
                        else:
                            target.unlink()
                        external.unlink(missing_ok=True)

            original_ops = root / 'ops'
            retained_ops = root / 'retained-ops'
            original_ops.rename(retained_ops)
            original_ops.symlink_to(retained_ops, target_is_directory=True)
            unchanged = inventory(root)
            result = cli('--company-provenance-capture')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('regular file', result.stderr)
            self.assertEqual(inventory(root), unchanged, 'linked parent redirected export')


if __name__ == '__main__':
    unittest.main()
