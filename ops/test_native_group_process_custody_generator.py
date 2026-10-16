"""Additive Group capture generation contract; no database/profile acceptance."""
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'ops/fixtures/native-group-process-capture-contract-v1.json'
FIXTURE_SHA256 = '6844113a2382df781d3d8f625a110bee5bca59b5bba5c2ba8815ed931fd3af8e'
SPEC = importlib.util.spec_from_file_location('group_capture_generator', ROOT / 'ops/generate-account-custody.py')
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)
OWNER = 'ops/postgres-native-group-process-v1-owner.sql'
CAPTURE = 'ops/postgres-capture-native-group-process-v1-custody.sql'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def contract():
    raw = FIXTURE.read_bytes()
    assert sha(raw) == FIXTURE_SHA256, 'reviewed capture-contract fixture differs'
    value = json.loads(raw)
    assert value['schema'] == 'console.native_group_process_capture_contract.v1'
    return value


def snapshot_fields(query):
    # Read only the existing outer snapshot JSON arguments; no SQL execution.
    marker = '), snapshots AS (\n SELECT jsonb_build_object(\n'
    assert query.count(marker) == 1, 'complete snapshot boundary missing/duplicated'
    text = query.split(marker, 1)[1]
    parts, depth, quote, start, index = [], 0, False, 0, 0
    while index < len(text):
        char = text[index]
        if not quote and char == '-' and index + 1 < len(text) and text[index + 1] == '-':
            index = text.find('\n', index)
            assert index >= 0, 'unterminated snapshot comment'
            continue
        if quote:
            if char == "'":
                if index + 1 < len(text) and text[index + 1] == "'":
                    index += 2
                    continue
                quote = False
        elif char == "'":
            quote = True
        elif char == '(':
            depth += 1
        elif char == ')':
            if depth == 0:
                parts.append(text[start:index].strip())
                break
            depth -= 1
        elif char == ',' and depth == 0:
            parts.append(text[start:index].strip())
            start = index + 1
        index += 1
    assert not quote and depth == 0 and len(parts) % 2 == 0
    fields = {}
    for key, value in zip(parts[::2], parts[1::2]):
        key = re.sub(r'^\s*--[^\n]*\n?', '', key, flags=re.M).strip()
        key = re.fullmatch(r"'([a-z_0-9]+)'", key).group(1)
        assert key not in fields, 'duplicate snapshot property'
        fields[key] = value
    return fields


class NativeGroupCaptureGeneration(unittest.TestCase):
    def capture(self):
        self.assertTrue(callable(getattr(GENERATOR, 'native_group_process_capture_files', None)),
                        'NATIVE_GROUP_CAPTURE_FUNCTION_MISSING')
        result = GENERATOR.native_group_process_capture_files()
        self.assertEqual(set(result), {OWNER, CAPTURE})
        self.assertTrue(all(isinstance(value, str) and value for value in result.values()))
        return result

    def test_named_capture_generator_has_complete_exact_sources(self):
        files, expected = self.capture(), contract()
        self.assertEqual(len(expected['group_sql_sources']), 14)
        owner = files[OWNER]
        markers = list(re.finditer(r'^-- source: (ops/native-group-process/[a-z0-9-]+\.sql)\n', owner, re.M))
        self.assertEqual(len(markers), 14)
        self.assertEqual({marker.group(1) for marker in markers},
                         {row['path'] for row in expected['group_sql_sources']})
        self.assertEqual([marker.group(1) for marker in markers],
                         expected['group_source_installation_order'],
                         'reviewed source dependency order must precede final ACL')
        self.assertTrue(all(not row.strip() or row.lstrip().startswith('--')
                            for row in owner[:markers[0].start()].splitlines()))
        for index, marker in enumerate(markers):
            name = marker.group(1)
            row = next(row for row in expected['group_sql_sources'] if row['path'] == name)
            path = ROOT / name
            self.assertTrue(path.is_file() and not path.is_symlink())
            for parent in path.parents:
                if parent == ROOT:
                    break
                self.assertFalse(parent.is_symlink())
            raw = path.read_bytes()
            self.assertEqual(len(raw), row['size'], name)
            self.assertEqual(sha(raw), row['sha256'], name)
            end = markers[index + 1].start() if index + 1 < len(markers) else len(owner)
            chunk = owner[marker.end():end].encode()
            self.assertTrue(chunk.startswith(raw), name)
            self.assertEqual(chunk[len(raw):].strip(), b'', 'extra SQL outside reviewed source: ' + name)
        routines = re.findall(r'CREATE(?: OR REPLACE)? FUNCTION\s+(public\.[a-z_0-9]+)\(', owner)
        self.assertEqual(len(routines), 41)
        self.assertEqual(set(routines), set(expected['group_routines']))
        self.assertEqual(re.findall(r'CREATE TABLE\s+(public\.[a-z_0-9]+)', owner), expected['group_tables'])
        acl = (ROOT / 'ops/native-group-process/acl-v1.sql').read_text()
        self.assertEqual(set(re.findall(r"\('(public\.[a-z_0-9]+)\(", acl)), set(routines))
        self.assertEqual(re.findall(r"\('([^']+)'\s*,\s*'console_rt'\)", acl), expected['group_runtime_signatures'])
        self.assertEqual(len(expected['compiled_source_identity']), 3)
        for row in expected['compiled_source_identity']:
            raw = (ROOT / row['path']).read_bytes()
            self.assertEqual(len(raw), row['size'], row['path'])
            self.assertEqual(sha(raw), row['sha256'], row['path'])

    def test_complete83_capture_retains_metadata_and_reserved_namespaces(self):
        query = self.capture()[CAPTURE]
        expected = contract()
        fields = snapshot_fields(query)
        predecessor = GENERATOR.native_org_unit_closed_perimeter_capture_files()[
            'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql']
        old = snapshot_fields(predecessor)
        self.assertEqual(set(fields), set(old) | set(expected['reserved_namespace_expressions']),
                         'only the four declared namespace records extend the snapshot')
        for key in old:
            self.assertEqual(fields[key], old[key], 'predecessor snapshot expression changed: ' + key)
        relation_begin = 'WITH wanted(name) AS (VALUES\n'
        relation_end = '), relations AS (\n'
        self.assertEqual(query.count(relation_begin), 1)
        self.assertEqual(predecessor.count(relation_begin), 1)
        old_relations = predecessor.split(relation_begin, 1)[1].split(relation_end, 1)[0]
        actual_relations = query.split(relation_begin, 1)[1].split(relation_end, 1)[0]
        old_names = re.findall(r"\('([a-z_0-9]+)'\)", old_relations)
        self.assertEqual(len(old_names), 76)
        self.assertEqual(len(set(old_names)), 76)
        group_names = [name.split('.', 1)[1] for name in expected['group_tables']]
        wanted = old_relations.rstrip() + ',\n ' + ',\n '.join(
            "('" + name + "')" for name in group_names) + '\n'
        self.assertEqual(actual_relations, wanted, 'complete predecessor76 plus exact Group7 roster')
        self.assertEqual(re.findall(r"\('([a-z_0-9]+)'\)", actual_relations), old_names + group_names)
        routine_begin = '), routine_records AS (\n'
        routine_end = '), owner_roles AS (\n'
        self.assertEqual(query.count(routine_begin), 1)
        self.assertEqual(predecessor.count(routine_begin), 1)
        old_routines = predecessor.split(routine_begin, 1)[1].split(routine_end, 1)[0]
        actual_routines = query.split(routine_begin, 1)[1].split(routine_end, 1)[0]
        group_routines = ','.join("('" + name.replace('.', "','", 1) + "')"
                                  for name in expected['group_routines'])
        wanted_routines = old_routines + ' OR (n.nspname,p.proname) IN (VALUES ' + group_routines + ')\n'
        self.assertEqual(actual_routines, wanted_routines,
                         'unchanged full routine metadata/predicates plus exact Group41 roster')
        for key, expression in expected['reserved_namespace_expressions'].items():
            self.assertEqual(fields[key], expression, 'whole unfiltered namespace expression: ' + key)
            for family in expected['reserved_naming_families']:
                self.assertIn("'" + family + "'", fields[key], key + '/' + family)
        for marker in ['required_schemas', 'effective_rights', 'default_privileges',
                       'external_owner_table_grants', 'external_owner_column_grants',
                       'identity_arguments', 'argdefaults', 'acl_is_null', 'leakproof']:
            self.assertIn(marker, query)
        self.assertIn("starts_with(p.proname,'identity_native_group_process", query)
        # No other predecessor metadata/observer/rights-input CTE is mutable.
        # Reconstruct the entire frozen query through only these finite deltas;
        # unchanged outer fields cannot mask altered relation_shapes/records.
        rights_begin = '\n ) AS record\n), company_startup_rights AS (\n SELECT\n'
        rights_end = ' AS valid\n), snapshots AS ('
        self.assertEqual(predecessor.count(rights_begin), 1)
        self.assertEqual(predecessor.count(rights_end), 1)
        old_rights = predecessor.split(rights_begin, 1)[1].split(rights_end, 1)[0]
        wanted_rights = old_rights
        for before, after in expected['rights_cardinality_replacements']:
            self.assertEqual(wanted_rights.count(before), 1)
            wanted_rights = wanted_rights.replace(before, after)
        snapshot_begin = '), snapshots AS (\n SELECT jsonb_build_object(\n'
        namespaces = ''.join("  '" + key + "'," + expression + ',\n'
                             for key, expression in expected['reserved_namespace_expressions'].items())
        deltas = [
            (relation_begin + old_relations + relation_end,
             relation_begin + wanted + relation_end),
            (routine_begin + old_routines + routine_end,
             routine_begin + wanted_routines + routine_end),
            (snapshot_begin, snapshot_begin + namespaces),
            (rights_begin + old_rights + rights_end,
             rights_begin + wanted_rights + rights_end),
            ('AS native_directory_startup_rights_valid FROM snapshots',
             'AS native_group_process_startup_rights_valid FROM snapshots'),
        ]
        wanted_query = predecessor
        for before, after in deltas:
            self.assertEqual(wanted_query.count(before), 1, 'unique frozen query delta')
            wanted_query = wanted_query.replace(before, after)
        self.assertEqual(query, wanted_query,
                         'complete predecessor query permits only five finite delta regions')

    def test_new83_rights_do_not_reinterpret_historical18_flag(self):
        query = self.capture()[CAPTURE]
        predecessor = GENERATOR.native_org_unit_closed_perimeter_capture_files()[
            'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql']
        begin = "   'startup_final_rights_valid',\n"
        end = '\n ) AS record\n), company_startup_rights AS (\n SELECT\n'
        self.assertEqual(query.split(begin, 1)[1].split(end, 1)[0],
                         predecessor.split(begin, 1)[1].split(end, 1)[0])
        rights_end = ' AS valid\n), snapshots AS ('
        self.assertEqual(query.count(end), 1)
        self.assertEqual(predecessor.count(end), 1)
        self.assertEqual(query.count(rights_end), 1)
        self.assertEqual(predecessor.count(rights_end), 1)
        rights = query.split(end, 1)[1].split(rights_end, 1)[0]
        old_rights = predecessor.split(end, 1)[1].split(rights_end, 1)[0]
        expected_rights = old_rights
        replacements = contract()['rights_cardinality_replacements']
        self.assertEqual(replacements, [
            ['count(*)=73 AND count(oid)=73', 'count(*)=83 AND count(oid)=83'],
            ['count(*)=82 AND count(oid)=82', 'count(*)=92 AND count(oid)=92'],
            ['count(*)=656 AND bool_and(allowed IS FALSE)', 'count(*)=736 AND bool_and(allowed IS FALSE)'],
            ['count(DISTINCT name)=73', 'count(DISTINCT name)=83'],
        ])
        for before, after in replacements:
            self.assertEqual(expected_rights.count(before), 1, 'actual frozen cardinality preimage')
            expected_rights = expected_rights.replace(before, after)
        self.assertEqual(rights, expected_rights,
                         'complete rights predicate permits only four declared cardinality replacements')
        for token in ['count(*)=83 AND count(oid)=83', 'count(*)=92 AND count(oid)=92',
                      'count(*)=736 AND bool_and(allowed IS FALSE)', 'count(DISTINCT name)=83',
                      'count(*) FILTER (WHERE expected_execute)=2', 'grantable IS FALSE']:
            self.assertIn(token, rights)
        self.assertIn('AS native_group_process_startup_rights_valid FROM snapshots', query)

    def test_all54_pins_retained_and48_immutable_files_unchanged(self):
        expected = contract()
        self.assertEqual(len(expected['all54_original_pins']), 54)
        self.assertEqual(len(expected['frozen48_paths']), 48)
        self.assertEqual(len(expected['declared_writer_seams']), 6)
        self.assertIsNone(expected['phase_pairs'], 'capture test cannot invent profile pairs')
        pins = {row['path']: row for row in expected['all54_original_pins']}
        self.assertEqual(len(pins), 54)
        self.assertEqual(set(pins), set(expected['frozen48_paths']) | set(expected['declared_writer_seams']))
        self.assertFalse(set(expected['frozen48_paths']) & set(expected['declared_writer_seams']))
        for name in expected['frozen48_paths']:
            path = ROOT / name
            self.assertTrue(path.is_file() and not path.is_symlink(), name)
            raw = path.read_bytes()
            self.assertEqual(len(raw), pins[name]['size'], name)
            self.assertEqual(sha(raw), pins[name]['sha256'], name)

    def test_historical_generator_definitions_and_default_bytes_unchanged(self):
        expected = contract()
        tree = ast.parse((ROOT / 'ops/generate-account-custody.py').read_text())
        definitions = {}
        for node in tree.body:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name != 'main':
                key = 'function:' + node.name
            elif isinstance(node, ast.Assign) and all(isinstance(target, ast.Name) for target in node.targets):
                key = 'assignment:' + ','.join(target.id for target in node.targets)
            else:
                continue
            self.assertNotIn(key, definitions, 'historical definition shadowed')
            definitions[key] = sha(ast.dump(node, include_attributes=False).encode())
        for key, digest in expected['historical_generator_definitions'].items():
            self.assertEqual(definitions.get(key), digest, key)
        outputs = GENERATOR.generated_files()
        self.assertEqual(set(outputs), set(expected['historical_default_outputs']))
        for name, digest in expected['historical_default_outputs'].items():
            self.assertEqual(sha(outputs[name].encode()), digest, name)

    def test_changed_missing_or_symlinked_sources_refuse_capture(self):
        self.capture()  # positive control precedes every source mutation
        expected = contract()
        for target in expected['group_sql_sources']:
            for mutation in ('changed', 'missing', 'symlink'):
                with self.subTest(source=target['path'], mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                    directory = Path(temporary)
                    for row in expected['group_sql_sources']:
                        destination = directory / row['path']
                        destination.parent.mkdir(parents=True, exist_ok=True)
                        destination.write_bytes((ROOT / row['path']).read_bytes())
                    changed = directory / target['path']
                    if mutation == 'changed':
                        changed.write_bytes(changed.read_bytes() + b'\n')
                    elif mutation == 'missing':
                        changed.unlink()
                    else:
                        exact = changed.with_suffix('.exact')
                        changed.rename(exact)
                        changed.symlink_to(exact)
                    original = GENERATOR.company_provenance_regular_path
                    def regular(name, *, required):
                        if name.startswith('ops/native-group-process/'):
                            path = directory / name
                            if path.is_symlink() or not path.is_file():
                                raise SystemExit('Group source must be a regular file')
                            return path
                        return original(name, required=required)
                    with patch.object(GENERATOR, 'company_provenance_regular_path', side_effect=regular):
                        with self.assertRaises((SystemExit, ValueError), msg=target['path'] + '/' + mutation):
                            GENERATOR.native_group_process_capture_files()


if __name__ == '__main__':
    unittest.main()
