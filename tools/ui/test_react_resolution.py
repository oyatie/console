"""Declared esbuild resolution at the actual record boundary; no business data."""
from argparse import Namespace
from contextlib import chdir
import os
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('react_bundle', Path(__file__).with_name('react_bundle.py'))
bundle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bundle)


class ReactResolutionTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.workspace = self.root / 'workspace'
        self.inputs = {}
        for name in ['people.tsx', 'people-guard.ts', 'people.css']:
            source = self.write('declared/src/' + name, '/* declared ' + name + ' */')
            staged = self.workspace / 'src' / name
            staged.parent.mkdir(parents=True, exist_ok=True)
            staged.symlink_to(source)
            self.inputs[bundle.CLIENT + '/src/' + name] = source
        self.node = self.write('tools/node', 'pinned node fixture')
        self.esbuild = self.write('tools/esbuild', 'pinned esbuild fixture')
        packages = ['react', 'react-dom', 'scheduler', 'typescript', '@types/react', '@types/react-dom', 'csstype']
        self.lock = {'format': 1, 'versions': {'node': '26.7.0', 'esbuild': '0.28.2', 'typescript': '5.9.3'},
                     'hosts': {'test-host': {name: {'executable_sha256': bundle.file_hash(path)}
                                             for name, path in [('node', self.node), ('esbuild', self.esbuild)]}},
                     'packages': [{'name': name, 'sha256': bundle.digest(name.encode())} for name in packages]}
        self.lock['packages'].extend({'name': name, 'sha256': bundle.digest(name.encode())}
                                     for name in ['@esbuild/darwin-arm64', '@esbuild/linux-x64'])
        package_roots = {}
        for name in packages:
            actual = self.write('declared/packages/' + name + '/index.js', '/* pinned package fixture */').parent
            package_roots[name] = str(actual)
            staged = self.workspace / 'node_modules' / name
            staged.parent.mkdir(parents=True, exist_ok=True)
            staged.symlink_to(actual, target_is_directory=True)
        package_record = self.write('packages.json', json.dumps(package_roots))
        for name, data in [('dependencies.lock.json', json.dumps(self.lock)),
                           ('dependencies.lock.bzl', bundle.lock_projection(self.lock))]:
            self.inputs[bundle.CLIENT + '/' + name] = self.write('declared/' + name, data)
        input_record = self.write('inputs.json', json.dumps({key: str(path) for key, path in self.inputs.items()}))
        compiled = self.root / 'compiled'
        for name in bundle.OUTPUTS:
            actual = self.write('native-output/' + name, '/* compiled ' + name + ' */')
            compiled.mkdir(parents=True, exist_ok=True)
            (compiled / name).symlink_to(actual)
        self.write('compiled/typecheck.tsbuildinfo', json.dumps({'version': '5.9.3'}))
        self.runtime = self.meta(['src/people.tsx', 'src/people.css', 'node_modules/react/index.js',
                                  'node_modules/react-dom/index.js', 'node_modules/scheduler/index.js'], 'people.js')
        self.runtime['outputs'][str((compiled / 'people.css').resolve())] = {
            'bytes': 1, 'imports': [], 'inputs': {str(self.workspace / 'src/people.css'): {'bytesInOutput': 1}}}
        self.runtime['inputs'][str(self.workspace / 'src/people.tsx')]['imports'] = [
            {'path': str(self.workspace / 'node_modules/react/index.js'),
             'kind': 'import-statement', 'original': 'react'},
            {'path': str(self.workspace / 'node_modules/react-dom/index.js'),
             'kind': 'import-statement', 'original': 'react-dom'}]
        self.runtime['inputs'][str(self.workspace / 'node_modules/react-dom/index.js')]['imports'] = [
            {'path': str(self.workspace / 'node_modules/scheduler/index.js'),
             'kind': 'require-call', 'original': 'scheduler'}]
        self.guard = self.meta(['src/people-guard.ts'], 'people-guard.js')
        self.args = Namespace(inputs=input_record, node=self.node, esbuild=self.esbuild, compiled=compiled,
                              out_dir=self.root / 'recorded', workspace=self.workspace, packages=package_record,
                              runtime_metafile=self.root / 'runtime.meta.json',
                              guard_metafile=self.root / 'guard.meta.json')
        self.serial = 0

    def write(self, relative, data):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(data)
        return path

    def meta(self, inputs, output):
        paths = [str(self.workspace / path) for path in inputs]
        return {'inputs': {path: {'bytes': 1, 'imports': []} for path in paths},
                'outputs': {str((self.root / 'compiled' / output).resolve()): {
                    'bytes': 1, 'entryPoint': paths[0], 'imports': [],
                    'inputs': {path: {'bytesInOutput': 1} for path in paths}}}}

    def supply(self, runtime=None, guard=None):
        self.args.runtime_metafile.write_text(json.dumps(self.runtime if runtime is None else runtime))
        self.args.guard_metafile.write_text(json.dumps(self.guard if guard is None else guard))
        self.serial += 1
        self.args.out_dir = self.root / ('recorded-' + str(self.serial))

    def refuse(self):
        with self.assertRaises(ValueError):
            bundle.record_bundle(self.args)
        self.assertFalse(self.args.out_dir.exists(), 'Refused resolution must not publish an artifact')

    def test_declared_current_closure_records_without_format_change(self):
        self.assertTrue(all((self.args.compiled / name).is_symlink() for name in bundle.OUTPUTS))
        self.assertEqual(len(self.lock['packages']), 9)
        self.assertEqual(len(json.loads(self.args.packages.read_text())), 7)
        self.supply()
        bundle.record_bundle(self.args)
        record = json.loads((self.args.out_dir / 'bundle.lock.json').read_text())
        self.assertEqual(set(record), {'format', 'inputs', 'outputs', 'dependency_lock_sha256', 'producer'})
        self.assertEqual(record['format'], 1)
        self.assertEqual(bundle.verify(record, self.inputs, bundle.output_paths(self.args.out_dir),
                                       self.lock, self.args.out_dir / 'typecheck.tsbuildinfo'), [])
        self.assertEqual({path.name for path in self.args.out_dir.iterdir()},
                         {*bundle.OUTPUTS, 'bundle.lock.json', 'typecheck.tsbuildinfo'})

    def test_ancestor_node_modules_inputs_are_refused_in_each_bundle(self):
        for bundle_name in ['runtime', 'guard']:
            for package in ['unlisted', 'react']:
                with self.subTest(bundle=bundle_name, package=package):
                    ancestor = self.write('node_modules/' + package + '/index.js', '/* ancestor install */')
                    meta = copy.deepcopy(getattr(self, bundle_name))
                    meta['inputs'][str(ancestor)] = {'bytes': 1, 'imports': []}
                    next(iter(meta['inputs'].values()))['imports'] = [
                        {'path': str(ancestor), 'kind': 'import-statement'}]
                    self.supply(**{bundle_name: meta})
                    self.refuse()

    def test_undeclared_sources_and_normalized_local_css_escapes_are_refused(self):
        for relative in ['workspace/src/undeclared.ts', 'workspace/src/nested/people.tsx',
                         'workspace/src/../../outside.css', 'workspace/node_modules/react/../../unlisted/index.js',
                         'workspace/node_modules/react-extra/index.js']:
            with self.subTest(path=relative):
                escaped = self.write(relative, '/* undeclared source */')
                meta = copy.deepcopy(self.runtime)
                meta['inputs'][str(escaped)] = {'bytes': 1, 'imports': []}
                self.supply(runtime=meta)
                self.refuse()

    def test_declared_source_name_cannot_hide_a_retargeted_symlink(self):
        staged = self.workspace / 'src/people.css'
        staged.unlink()
        staged.symlink_to(self.write('outside.css', '/* undeclared CSS target */'))
        self.supply()
        self.refuse()

    def test_external_javascript_edges_in_inputs_and_outputs_are_refused(self):
        for bundle_name in ['runtime', 'guard']:
            for section in ['inputs', 'outputs']:
                with self.subTest(bundle=bundle_name, section=section):
                    meta = copy.deepcopy(getattr(self, bundle_name))
                    next(iter(meta[section].values()))['imports'] = [
                        {'path': 'https://example.invalid/late.js', 'kind': 'import-statement', 'external': True}]
                    self.supply(**{bundle_name: meta})
                    self.refuse()

    def test_external_css_import_and_url_edges_are_refused(self):
        for section, kind in [('inputs', 'import-rule'), ('outputs', 'url-token')]:
            with self.subTest(section=section, kind=kind):
                meta = copy.deepcopy(self.runtime)
                key = str(self.workspace / 'src/people.css') if section == 'inputs' else str((self.args.compiled / 'people.css').resolve())
                meta[section][key]['imports'] = [
                    {'path': 'https://example.invalid/remote.css', 'kind': kind, 'external': True}]
                self.supply(runtime=meta)
                self.refuse()

    def test_missing_and_malformed_metadata_are_refused_before_publication(self):
        for bundle_name in ['runtime', 'guard']:
            path = getattr(self.args, bundle_name + '_metafile')
            for invalid in ['missing', '{invalid-json', 'null', '[]', '{}',
                            '{"inputs":[],"outputs":{}}', '{"inputs":{},"outputs":[]}']:
                with self.subTest(bundle=bundle_name, invalid=invalid):
                    self.supply()
                    if invalid == 'missing':
                        path.unlink()
                    else:
                        path.write_text(invalid)
                    self.refuse()


    def test_cwd_relative_native_paths_and_symlinked_packages_record(self):
        def relative(meta):
            result = copy.deepcopy(meta)
            for section in ['inputs', 'outputs']:
                result[section] = {os.path.relpath(path, self.root): value for path, value in result[section].items()}
                for value in result[section].values():
                    for edge in value['imports']:
                        edge['path'] = os.path.relpath(edge['path'], self.root)
                    if 'entryPoint' in value:
                        value['entryPoint'] = os.path.relpath(value['entryPoint'], self.root)
                    if 'inputs' in value:
                        value['inputs'] = {os.path.relpath(path, self.root): contribution
                                           for path, contribution in value['inputs'].items()}
            return result
        self.supply(runtime=relative(self.runtime), guard=relative(self.guard))
        packages = json.loads(self.args.packages.read_text())
        self.args.packages.write_text(json.dumps({name: os.path.relpath(path, self.root)
                                                 for name, path in packages.items()}))
        self.args.workspace = Path('workspace')
        self.args.compiled = Path('compiled')
        with chdir(self.root):
            bundle.record_bundle(self.args)
        self.assertEqual(bundle.validate_native(self.args.out_dir, self.inputs, self.lock)['format'], 1)

    def test_locked_package_name_cannot_hide_retargeted_root_or_file(self):
        staged = self.workspace / 'node_modules/react'
        actual = staged.resolve()
        staged.unlink()
        staged.symlink_to(self.write('outside/react/index.js', '/* undeclared root */').parent,
                          target_is_directory=True)
        self.supply()
        self.refuse()
        staged.unlink()
        staged.symlink_to(actual, target_is_directory=True)
        (actual / 'index.js').unlink()
        (actual / 'index.js').symlink_to(self.write('outside/react-file.js', '/* undeclared file */'))
        self.supply()
        self.refuse()

    def test_declared_package_map_is_required_and_matches_locked_names(self):
        original = self.args.packages.read_text()
        for invalid in ['missing', '{invalid-json', 'null', '[]', '{}',
                        json.dumps({'react': str(self.root / 'declared/packages/react')})]:
            with self.subTest(map=invalid):
                self.supply()
                if invalid == 'missing':
                    self.args.packages.unlink()
                else:
                    self.args.packages.write_text(invalid)
                self.refuse()
                self.args.packages.write_text(original)

    def test_metadata_closure_cannot_be_empty_or_misidentify_outputs_and_entries(self):
        cases = []
        for name in ['runtime', 'guard']:
            for section in ['inputs', 'outputs']:
                meta = copy.deepcopy(getattr(self, name))
                meta[section] = {}
                cases.append((name, meta))
        no_css = copy.deepcopy(self.runtime)
        no_css['outputs'].pop(str((self.args.compiled / 'people.css').resolve()))
        cases.append(('runtime', no_css))
        extra = copy.deepcopy(self.runtime)
        extra['outputs'][str(self.args.compiled / 'extra.css')] = {'bytes': 1, 'imports': [], 'inputs': {}}
        cases.append(('runtime', extra))
        renamed = copy.deepcopy(self.guard)
        renamed['outputs'][str(self.args.compiled / 'renamed.js')] = renamed['outputs'].pop(
            str((self.args.compiled / 'people-guard.js').resolve()))
        cases.append(('guard', renamed))
        for name, entry in [('runtime', 'people-guard.ts'), ('guard', 'people.tsx')]:
            meta = copy.deepcopy(getattr(self, name))
            next(iter(meta['outputs'].values()))['entryPoint'] = str(self.workspace / 'src' / entry)
            cases.append((name, meta))
        for number, (name, meta) in enumerate(cases):
            with self.subTest(case=number, bundle=name):
                self.supply(**{name: meta})
                self.refuse()


if __name__ == '__main__':
    unittest.main()
