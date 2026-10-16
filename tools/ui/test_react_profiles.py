"""Closed Account/People producer profiles; fixture artifacts are never compiled."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch

import test_react_resolution as resolution

bundle = resolution.bundle
PEOPLE = ('people.js', 'people.css', 'people-guard.js')
ACCOUNT = ('account.js', 'account.css', 'account-guard.js')
CONTROLLER = 'backend/crates/payroll/ui/src/native_account.js'


class ReactProfileContractTests(unittest.TestCase):
    def fixture(self, account=False):
        f = resolution.ReactResolutionTests()
        f.setUp()
        self.addCleanup(f.doCleanups)
        if account:
            replacements = dict(zip(('people.tsx', 'people-guard.ts', *PEOPLE),
                                    ('account.tsx', 'account-guard.ts', *ACCOUNT)))
            for old, new in replacements.items():
                prefix = bundle.CLIENT + '/src/'
                if prefix + old in f.inputs:
                    source = f.inputs.pop(prefix + old)
                    renamed = source.with_name(new)
                    (f.workspace / 'src' / old).unlink()
                    source.rename(renamed)
                    (f.workspace / 'src' / new).symlink_to(renamed)
                    f.inputs[prefix + new] = renamed
                if old in PEOPLE:
                    staged = f.args.compiled / old
                    actual = staged.resolve()
                    renamed = actual.with_name(new)
                    staged.unlink()
                    actual.rename(renamed)
                    (f.args.compiled / new).symlink_to(renamed)
            for name in ('runtime', 'guard'):
                encoded = json.dumps(getattr(f, name))
                for old, new in replacements.items():
                    encoded = encoded.replace(old, new)
                setattr(f, name, json.loads(encoded))
            f.inputs[CONTROLLER] = f.write('declared/controller/native_account.js',
                                          '/* declared controller custody fixture */')
        self.input_map(f)
        return f

    def input_map(self, f):
        f.args.inputs.write_text(json.dumps({key: str(path) for key, path in f.inputs.items()}))

    def record(self, f, profile):
        f.supply()
        f.args.profile = profile
        bundle.record_bundle(f.args)
        return json.loads((f.args.out_dir / 'bundle.lock.json').read_text())

    def account_metafiles(self, f):
        f.supply()
        prefix = bundle.CLIENT + '/src/'
        sources = {f.workspace / 'src' / key[len(prefix):]: path.resolve()
                   for key, path in f.inputs.items() if key.startswith(prefix)}
        packages = {f.workspace / 'node_modules' / name: Path(path).resolve()
                    for name, path in json.loads(f.args.packages.read_text()).items()}
        outputs = {name: (f.args.compiled / name).resolve(strict=True) for name in ACCOUNT}
        bundle.validate_metafile(f.args.runtime_metafile, f.workspace, sources, packages,
                                 [outputs['account.js'], outputs['account.css']], 'account.tsx')
        bundle.validate_metafile(f.args.guard_metafile, f.workspace, sources, packages,
                                 [outputs['account-guard.js']], 'account-guard.ts')

    def account_ready(self, f):
        self.account_metafiles(f)
        record = self.record(f, 'account')
        self.assertEqual(set(record['outputs']), set(ACCOUNT))
        self.assertEqual(record['producer']['kind'], 'buck2-native-react-account-v1')
        self.assertEqual(record['inputs'][CONTROLLER], bundle.file_hash(f.inputs[CONTROLLER]))
        self.assertEqual({p.name for p in f.args.out_dir.iterdir()},
                         {*ACCOUNT, 'bundle.lock.json', 'typecheck.tsbuildinfo'})
        return record

    def refuse(self, f, profile):
        f.supply()
        f.args.profile = profile
        with self.assertRaises(ValueError):
            bundle.record_bundle(f.args)
        self.assertFalse(f.args.out_dir.exists(), 'Refusal must precede any publication')

    def cli(self, f, *options):
        names = ('inputs', 'compiled', 'node', 'esbuild', 'out_dir', 'workspace',
                 'packages', 'runtime_metafile', 'guard_metafile')
        argv = ['react_bundle.py', 'record']
        for name in names:
            argv.extend(('--' + name.replace('_', '-'), str(getattr(f.args, name))))
        with patch('sys.argv', argv + list(options)):
            bundle.main()

    def test_people_explicit_and_legacy_keep_existing_record_and_outputs(self):
        f = self.fixture()
        f.supply()
        bundle.record_bundle(f.args)
        legacy = json.loads((f.args.out_dir / 'bundle.lock.json').read_text())
        explicit = self.record(f, 'people')
        self.assertEqual(explicit, legacy)
        self.assertEqual(bundle.OUTPUTS, PEOPLE)
        self.assertEqual(bundle.COMMITTED, 'backend/crates/payroll/ui/react')
        self.assertEqual(explicit['producer']['kind'], 'buck2-native-react-people-v1')
        self.assertEqual(set(explicit['outputs']), set(PEOPLE))
        self.assertEqual(bundle.validate_native(f.args.out_dir, f.inputs, f.lock), explicit)

    def test_account_fixture_metafiles_already_validate_at_real_closure_boundary(self):
        self.account_metafiles(self.fixture(account=True))

    def test_account_closed_profile_records_exact_native_artifact_contract(self):
        self.account_ready(self.fixture(account=True))

    def test_unknown_profiles_are_refused_after_healthy_people_control(self):
        for profile in ('', None, True, 'Account', 'people.tsx', '../account', 'arbitrary'):
            with self.subTest(profile=profile):
                f = self.fixture()
                self.record(f, 'people')
                self.refuse(f, profile)

    def test_account_profile_refuses_a_healthy_people_closure(self):
        f = self.fixture()
        self.record(f, 'people')
        f.inputs[CONTROLLER] = f.write('declared/controller/native_account.js',
                                      '/* declared controller custody fixture */')
        self.input_map(f)
        self.refuse(f, 'account')

    def test_account_fixed_entries_refuse_other_declared_entrypoints(self):
        for name, alternative in (('runtime', 'other.tsx'), ('guard', 'other-guard.ts')):
            with self.subTest(bundle=name):
                f = self.fixture(account=True)
                self.account_ready(f)
                source = f.write('declared/src/' + alternative, '/* other declared entry */')
                staged = f.workspace / 'src' / alternative
                staged.symlink_to(source)
                f.inputs[bundle.CLIENT + '/src/' + alternative] = source
                self.input_map(f)
                meta = getattr(f, name)
                meta['inputs'][str(staged)] = {'bytes': 1, 'imports': []}
                next(iter(meta['outputs'].values()))['entryPoint'] = str(staged)
                self.refuse(f, 'account')

    def test_account_missing_extra_and_people_outputs_are_refused(self):
        for name, mutation in (('runtime', 'missing'), ('guard', 'missing'),
                               ('runtime', 'extra'), ('guard', 'extra'),
                               ('runtime', 'people'), ('guard', 'people'),
                               ('runtime', 'physical-missing'), ('guard', 'physical-missing'),
                               ('runtime', 'physical-extra')):
            with self.subTest(bundle=name, mutation=mutation):
                f = self.fixture(account=True)
                self.account_ready(f)
                meta = getattr(f, name)
                first = next(iter(meta['outputs']))
                if mutation == 'physical-missing':
                    (f.args.compiled / ('account.js' if name == 'runtime' else 'account-guard.js')).unlink()
                elif mutation == 'physical-extra':
                    f.write('compiled/extra.js', '/* undeclared physical output fixture */')
                elif mutation == 'missing':
                    meta['outputs'].pop(first)
                else:
                    output = ('people.js' if name == 'runtime' else 'people-guard.js') \
                             if mutation == 'people' else 'extra.js'
                    path = f.write('compiled/' + output, '/* substituted output fixture */')
                    row = copy.deepcopy(meta['outputs'][first])
                    if mutation == 'people':
                        meta['outputs'].pop(first)
                    meta['outputs'][str(path.resolve())] = row
                self.refuse(f, 'account')

    def test_account_controller_custody_requires_declared_regular_existing_input(self):
        for mutation in ('undeclared', 'missing', 'symlink', 'directory'):
            with self.subTest(controller=mutation):
                f = self.fixture(account=True)
                self.account_ready(f)
                path = f.inputs[CONTROLLER]
                if mutation == 'undeclared':
                    del f.inputs[CONTROLLER]
                else:
                    path.unlink()
                    if mutation == 'symlink':
                        path.symlink_to(f.write('outside/native_account.js', '/* outside controller */'))
                    elif mutation == 'directory':
                        path.mkdir()
                self.input_map(f)
                self.refuse(f, 'account')

    def test_account_native_and_committed_validation_reject_profile_and_file_mismatch(self):
        f = self.fixture(account=True)
        self.account_ready(f)
        candidate = f.args.out_dir
        committed = f.root / 'committed-account'
        committed.mkdir()
        for name in (*ACCOUNT, 'bundle.lock.json'):
            (committed / name).write_bytes((candidate / name).read_bytes())

        def validate(profile):
            f.serial += 1
            output = f.root / ('validated-' + str(f.serial))
            argv = ['react_bundle.py', 'validate', '--profile', profile,
                    '--inputs', str(f.args.inputs), '--candidate', str(candidate),
                    '--committed', str(committed), '--out-dir', str(output)]
            with patch('sys.argv', argv):
                bundle.main()
            return output

        validated = validate('account')
        self.assertEqual({p.name for p in validated.iterdir()}, set(ACCOUNT))
        with self.assertRaises(ValueError):
            validate('people')
        for directory in (candidate, committed):
            with self.subTest(directory=directory.name):
                extra = directory / 'extra.js'
                extra.write_text('/* undeclared validation output fixture */')
                with self.assertRaises(ValueError):
                    validate('account')
                extra.unlink()
                path = directory / 'account.css'
                original = path.read_bytes()
                path.unlink()
                with self.assertRaises(ValueError):
                    validate('account')
                path.write_bytes(original)

    def test_cli_selects_closed_profiles_and_never_accepts_caller_entry_paths(self):
        f = self.fixture()
        f.supply()
        self.cli(f, '--profile', 'people')
        self.assertEqual(set(json.loads((f.args.out_dir / 'bundle.lock.json').read_text())['outputs']), set(PEOPLE))
        for option in ('--entry', '--runtime-entry', '--guard-entry'):
            f.supply()
            with self.subTest(option=option), self.assertRaises(SystemExit) as caught:
                self.cli(f, '--profile', 'people', option, '../arbitrary.tsx')
            self.assertEqual(caught.exception.code, 2)
            self.assertFalse(f.args.out_dir.exists())
        f = self.fixture(account=True)
        f.supply()
        self.cli(f, '--profile', 'account')
        self.assertEqual(set(json.loads((f.args.out_dir / 'bundle.lock.json').read_text())['outputs']), set(ACCOUNT))

    def account_pair(self):
        f = self.fixture(account=True)
        record = self.account_ready(f)
        candidate = f.args.out_dir
        committed = f.root / 'committed-account'
        committed.mkdir()
        for name in (*ACCOUNT, 'bundle.lock.json'):
            (committed / name).write_bytes((candidate / name).read_bytes())
        output = self.validate_account_pair(f, candidate, committed)
        self.assertEqual({p.name for p in output.iterdir()}, set(ACCOUNT))
        return f, candidate, committed, record

    def validate_account_pair(self, f, candidate, committed):
        f.serial += 1
        output = f.root / ('integrity-validated-' + str(f.serial))
        argv = ['react_bundle.py', 'validate', '--profile', 'account',
                '--inputs', str(f.args.inputs), '--candidate', str(candidate),
                '--committed', str(committed), '--out-dir', str(output)]
        with patch('sys.argv', argv):
            bundle.main()
        return output

    def refuse_account_pair(self, f, candidate, committed):
        with self.assertRaises(ValueError):
            self.validate_account_pair(f, candidate, committed)
        self.assertFalse((f.root / ('integrity-validated-' + str(f.serial))).exists(),
                         'Invalid Account custody must not publish an artifact')

    def test_account_record_shape_kind_lock_and_producer_pins_remain_closed(self):
        f, candidate, committed, record = self.account_pair()
        mutations = [lambda r: r.update(format=True), lambda r: r.update(format=False),
                     lambda r: r.update(format=2), lambda r: r.update(extra=True),
                     lambda r: r.update(inputs=[]), lambda r: r.update(outputs=[]),
                     lambda r: r.update(producer=[]),
                     lambda r: r.update(dependency_lock_sha256='c' * 64),
                     lambda r: r['producer'].update(kind='buck2-native-react-people-v1'),
                     lambda r: r['producer'].update(kind='npm-wrapper'),
                     lambda r: r['producer'].update(host='unknown'),
                     lambda r: r['producer'].update(host=[]),
                     lambda r: r['producer'].update(extra=True),
                     lambda r: r['producer'].update(tools=[]),
                     lambda r: r['producer'].update(versions=[]),
                     lambda r: r['producer'].update(typecheck_sha256=''),
                     lambda r: r['producer'].update(typecheck_sha256=123),
                     lambda r: r['producer']['tools'].update(node='c' * 64),
                     lambda r: r['producer']['tools'].update(esbuild='c' * 64),
                     lambda r: r['producer']['tools'].update(extra='c' * 64),
                     lambda r: r['producer']['versions'].update(extra='1')]
        invalid = [None, [], 1, {}]
        for mutation in mutations:
            changed = copy.deepcopy(record)
            mutation(changed)
            invalid.append(changed)
        for key in record:
            changed = copy.deepcopy(record)
            del changed[key]
            invalid.append(changed)
        for key in record['producer']:
            changed = copy.deepcopy(record)
            del changed['producer'][key]
            invalid.append(changed)
        for section in ('tools', 'versions'):
            for key in record['producer'][section]:
                changed = copy.deepcopy(record)
                del changed['producer'][section][key]
                invalid.append(changed)
        for key in record['producer']['versions']:
            changed = copy.deepcopy(record)
            changed['producer']['versions'][key] = '0.0.0'
            invalid.append(changed)
        for directory in (candidate, committed):
            manifest = directory / 'bundle.lock.json'
            original = manifest.read_bytes()
            for number, changed in enumerate(invalid):
                with self.subTest(directory=directory.name, corruption=number):
                    manifest.write_text(json.dumps(changed))
                    self.refuse_account_pair(f, candidate, committed)
                    manifest.write_bytes(original)

    def test_account_hashes_bind_controller_sources_and_every_output(self):
        f, candidate, committed, record = self.account_pair()
        for key, path in f.inputs.items():
            with self.subTest(input=key):
                original = path.read_bytes()
                path.write_bytes(original + b'corruption')
                self.refuse_account_pair(f, candidate, committed)
                path.unlink()
                self.refuse_account_pair(f, candidate, committed)
                path.write_bytes(original)
        for directory in (candidate, committed):
            manifest = directory / 'bundle.lock.json'
            original_manifest = manifest.read_bytes()
            for section in ('inputs', 'outputs'):
                for key in (*record[section], 'undeclared'):
                    for mutation in ('hash', 'omit') if key != 'undeclared' else ('extra',):
                        with self.subTest(directory=directory.name, section=section,
                                          identity=key, mutation=mutation):
                            changed = copy.deepcopy(record)
                            if mutation == 'omit':
                                del changed[section][key]
                            else:
                                changed[section][key] = 'c' * 64
                            manifest.write_text(json.dumps(changed))
                            self.refuse_account_pair(f, candidate, committed)
                            manifest.write_bytes(original_manifest)
            for name in ACCOUNT:
                path = directory / name
                original = path.read_bytes()
                with self.subTest(directory=directory.name, output=name):
                    path.write_bytes(original + b'corruption')
                    self.refuse_account_pair(f, candidate, committed)
                    path.unlink()
                    self.refuse_account_pair(f, candidate, committed)
                    path.write_bytes(original)

    def test_account_actual_tools_and_native_typecheck_are_verified(self):
        f, candidate, committed, record = self.account_pair()
        for name in ('node', 'esbuild'):
            path = getattr(f.args, name)
            original = path.read_bytes()
            with self.subTest(tool=name):
                path.write_bytes(original + b'corruption')
                self.refuse(f, 'account')
                path.unlink()
                self.refuse(f, 'account')
                path.write_bytes(original)
        check = candidate / 'typecheck.tsbuildinfo'
        manifest = candidate / 'bundle.lock.json'
        original_check, original_manifest = check.read_bytes(), manifest.read_bytes()
        changed = copy.deepcopy(record)
        changed['producer']['typecheck_sha256'] = 'c' * 64
        manifest.write_text(json.dumps(changed))
        self.refuse_account_pair(f, candidate, committed)
        manifest.write_bytes(original_manifest)
        for invalid in ('{invalid-json', '{}', '{"version":"0.0.0"}', '{"version":true}'):
            with self.subTest(typecheck=invalid):
                check.write_text(invalid)
                changed = copy.deepcopy(record)
                changed['producer']['typecheck_sha256'] = bundle.file_hash(check)
                manifest.write_text(json.dumps(changed))
                self.refuse_account_pair(f, candidate, committed)
                check.write_bytes(original_check)
                manifest.write_bytes(original_manifest)
        check.unlink()
        self.refuse_account_pair(f, candidate, committed)
        check.write_bytes(original_check)

    def test_account_committed_output_cannot_forge_a_self_consistent_record(self):
        f, candidate, committed, record = self.account_pair()
        original_manifest = (committed / 'bundle.lock.json').read_bytes()
        for name in ACCOUNT:
            with self.subTest(output=name):
                path = committed / name
                original = path.read_bytes()
                path.write_text('forged but self-consistent Account ' + name)
                changed = copy.deepcopy(record)
                changed['outputs'][name] = bundle.file_hash(path)
                (committed / 'bundle.lock.json').write_text(json.dumps(changed))
                self.refuse_account_pair(f, candidate, committed)
                path.write_bytes(original)
                (committed / 'bundle.lock.json').write_bytes(original_manifest)



if __name__ == '__main__':
    unittest.main()
