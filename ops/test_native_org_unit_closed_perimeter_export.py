"""Uninstalled byte export only; installed OrgUnit correctness is a separate gate.

Contract: closed-perimeter design V3, SHA-256
6355d9a206e9390fd677dcb8e9b97dba061227e492f84eb8b23ca20c2640e4a4.
Approved source basis: 4eec2ad004127dd041e18b71b551c18c755aa09b plus
the independently reviewed compatibility checkpoint, never a runtime profile pin.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import types
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path('ops/generate-account-custody.py')
GENERATOR = types.ModuleType('closed_org_unit_export_generator')
GENERATOR.__file__ = str(ROOT / SCRIPT)
exec(compile((ROOT / SCRIPT).read_bytes(), GENERATOR.__file__, 'exec'),
     GENERATOR.__dict__)

# Existing exact output identities, not fabricated successor SQL fingerprints.
CURRENT_DEFAULTS_SHA256 = '93a82a4c3e295d3218bb80353868b85c591a3aef522560173d99d4fb133e09a6'
HISTORICAL_DEFAULTS_SHA256 = '63576789a001db496bd3b321b5308ddede0ac954d7c681a4d329fec88042637b'
SOURCE = 'ops/native-org-unit/closed-perimeter-v1.sql'
MODE = '--native-org-unit-closed-perimeter-capture'
OWNER = 'ops/postgres-native-org-unit-closed-perimeter-v1-owner.sql'
CAPTURE = 'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql'
ARTIFACTS = {OWNER, CAPTURE}


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def inventory(root):
    result = {}
    for path in root.rglob('*'):
        if path.is_symlink():
            identity = ('symlink', str(path.readlink()))
        elif path.is_file():
            identity = ('file', digest(path.read_bytes()))
        elif path.is_dir():
            identity = ('directory',)
        else:
            identity = ('special', path.lstat().st_mode)
        result[str(path.relative_to(root))] = identity
    return result


class NativeOrgUnitClosedPerimeterExport(unittest.TestCase):
    def test_explicit_uninstalled_export_preserves_custody(self):
        defaults = GENERATOR.generated_files()
        frozen = {name: digest(content.encode()) for name, content in defaults.items()}
        self.assertEqual(len(frozen), 33)
        self.assertEqual(digest(json.dumps(frozen, sort_keys=True,
                                         separators=(',', ':')).encode()),
                         CURRENT_DEFAULTS_SHA256, 'default output names or bytes changed')
        ledger_name = 'ops/account-custody-migrations.sha384'
        ledger = (ROOT / ledger_name).read_bytes()
        self.assertEqual(len(ledger.splitlines()), 231)
        self.assertEqual(digest(ledger),
                         '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357')
        prefix = b''.join(ledger.splitlines(keepends=True)[:230])
        self.assertEqual(digest(prefix),
                         '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325')
        historical = {**frozen, ledger_name: digest(prefix)}
        self.assertEqual(digest(json.dumps(historical, sort_keys=True,
                                         separators=(',', ':')).encode()),
                         HISTORICAL_DEFAULTS_SHA256, 'historical default custody changed')
        for name, expected in frozen.items():
            self.assertEqual(digest((ROOT / name).read_bytes()), expected, name)

        with tempfile.TemporaryDirectory(prefix='native-org-unit-closed-export-') as temporary:
            root = Path(temporary)
            inputs = {SCRIPT, *(Path(name) for name in defaults)}
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
                return subprocess.run([sys.executable, '-I', '-B', '-S',
                                       str(root / SCRIPT), *arguments], cwd=root,
                                      capture_output=True, text=True, timeout=30)

            before = inventory(root)
            ordinary = cli('--check')
            self.assertEqual(ordinary.returncode, 0, ordinary.stderr)
            self.assertEqual(inventory(root), before, 'ordinary check wrote files')
            exported = cli(MODE)
            self.assertEqual(exported.returncode, 0,
                             'NATIVE_ORG_UNIT_CLOSED_CLI_EXPORT_MISSING: ' + exported.stderr)
            after = inventory(root)
            self.assertEqual(set(after) - set(before), ARTIFACTS,
                             'explicit export published unexpected files')
            self.assertEqual({name: after[name] for name in before}, before,
                             'export changed existing sources or custody')
            for arguments in ((MODE,), (MODE, '--check'), (), ('--check',)):
                result = cli(*arguments)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(inventory(root), after, 'export/check/default changed bytes')

            # The source digest is established by independent candidate source
            # review. This test verifies its complete byte export, not its SQL
            # authorization behavior or a hash guessed before that review.
            pins = GENERATOR.NATIVE_ORG_UNIT_CLOSED_PERIMETER_SOURCE_SHA256
            self.assertEqual(set(pins), {SOURCE})
            self.assertRegex(pins[SOURCE], r'^[0-9a-f]{64}$')
            source = (root / SOURCE).read_bytes()
            self.assertEqual(digest(source), pins[SOURCE])
            expected_owner = (
                '-- Generated UNINSTALLED native OrgUnit closed-perimeter source; not a custody finalizer.\n'
                '-- No finalized profile or installation is authorized by this artifact.\n'
                '-- source: ' + SOURCE + '\n' + source.decode('utf-8')).encode()
            self.assertEqual((root / OWNER).read_bytes(), expected_owner,
                             'export truncated, substituted or appended declared SQL')

            # Retain the complete prior serializer. Capture the added relations
            # and every schema/kind/owner of guard names, plus the existing
            # immutable trigger functions whose modes this phase may promote.
            predecessor = GENERATOR.company_provenance_capture_files()[
                'ops/postgres-capture-company-provenance-v1-custody.sql']
            relation_anchor = " ('users')\n), relations AS ("
            routine_anchor = '), owner_roles AS ('
            self.assertEqual(predecessor.count(relation_anchor), 1)
            self.assertEqual(predecessor.count(routine_anchor), 1)
            expected_capture = predecessor.replace(relation_anchor,
                " ('users'),\n ('org_units'),\n ('org_unit_revisions'),\n"
                " ('org_unit_source_bindings')\n), relations AS (").replace(routine_anchor,
                " OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN "
                "('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))\n"
                + routine_anchor)
            self.assertEqual((root / CAPTURE).read_bytes(), expected_capture.encode(),
                             'capture omitted or changed prior/declared metadata')

            # --check reports every missing/drifted output and never repairs it.
            for name in sorted(ARTIFACTS):
                target = root / name
                original = target.read_bytes()
                for fault in ('drift', 'missing'):
                    with self.subTest(output=name, fault=fault):
                        target.unlink()
                        if fault == 'drift':
                            target.write_bytes(original + b'\n')
                        changed = inventory(root)
                        result = cli(MODE, '--check')
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('differs', result.stderr)
                        self.assertEqual(inventory(root), changed, 'check repaired output')
                        target.write_bytes(original)

            # Source refusal must precede every output, for both export/check.
            original = (root / SOURCE).read_bytes()
            for fault in ('missing', 'drift', 'symlink'):
                with self.subTest(source=SOURCE, fault=fault):
                    for name in ARTIFACTS:
                        (root / name).unlink(missing_ok=True)
                    target = root / SOURCE
                    target.unlink()
                    linked = root / 'exact-unrelated-source.sql'
                    if fault == 'drift':
                        target.write_bytes(original + b'\n')
                    elif fault == 'symlink':
                        linked.write_bytes(original)
                        target.symlink_to(linked)
                    changed = inventory(root)
                    for arguments in ((MODE,), (MODE, '--check')):
                        result = cli(*arguments)
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn('reviewed bytes' if fault == 'drift' else 'regular file',
                                      result.stderr)
                        self.assertEqual(inventory(root), changed, 'source refusal wrote files')
                    target.unlink(missing_ok=True)
                    target.write_bytes(original)
                    linked.unlink(missing_ok=True)

            # Validate all destinations before publishing either artifact.
            for name in sorted(ARTIFACTS):
                for fault in ('symlink', 'dangling_symlink', 'directory'):
                    with self.subTest(output=name, fault=fault):
                        target = root / name
                        unrelated = root / 'unrelated.sql'
                        if fault == 'directory':
                            target.mkdir()
                        else:
                            if fault == 'symlink':
                                unrelated.write_bytes(b'unrelated immutable bytes\n')
                            target.symlink_to(unrelated)
                        changed = inventory(root)
                        for arguments in ((MODE,), (MODE, '--check')):
                            result = cli(*arguments)
                            self.assertNotEqual(result.returncode, 0)
                            self.assertIn('regular file', result.stderr)
                            self.assertEqual(inventory(root), changed, 'bad destination wrote files')
                        if fault == 'directory':
                            target.rmdir()
                        else:
                            target.unlink()
                        unrelated.unlink(missing_ok=True)

            ops = root / 'ops'
            retained = root / 'retained-ops'
            ops.rename(retained)
            ops.symlink_to(retained, target_is_directory=True)
            changed = inventory(root)
            for arguments in ((MODE,), (MODE, '--check')):
                result = cli(*arguments)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('regular file', result.stderr)
                self.assertEqual(inventory(root), changed, 'linked parent redirected output')


if __name__ == '__main__':
    unittest.main()
