"""Additive generated Group serving protocol; runtime PG histories are separate.

Test-source proposal only. The pairs below are exact measured diagnostic
observations, not a claim of profile admission or browser/release acceptance.
No business rows are populated by this suite.
"""
import ast
import hashlib
import json
from pathlib import Path
import re
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'ops/generate-account-custody.py'
GENERATOR = types.ModuleType('native_group_serving_custody_generator')
GENERATOR.__file__ = str(SCRIPT)
exec(compile(SCRIPT.read_bytes(), str(SCRIPT), 'exec'), GENERATOR.__dict__)

FROZEN_FIXTURE = ROOT / 'ops/fixtures/native-group-process-capture-contract-v1.json'
FROZEN_FIXTURE_SHA256 = '6844113a2382df781d3d8f625a110bee5bca59b5bba5c2ba8815ed931fd3af8e'
OWNER = 'ops/postgres-native-group-process-v1-owner.sql'
CAPTURE = 'ops/postgres-capture-native-group-process-v1-custody.sql'
LEDGER = 'ops/account-custody-migrations.sha384'
CLASSIFIER = 'ops/postgres-native-group-process-v1-custody-state.sql'
APP_CLASSIFIER = 'backend/app/src/native_group_process_v1_custody_state.sql'
FINALIZER = 'ops/postgres-finalize-native-group-process-v1.sql'
ARTIFACTS = {CLASSIFIER, APP_CLASSIFIER, FINALIZER}
INPUT_SHA256 = {
    OWNER: 'cbf641175a7bf589bd46fc21dc775fe2fab8b1a8ab7e46b04dee3c32422038f9',
    CAPTURE: '3406bac381896fab4e9a1c079110d3dc770a9d89b0b564e7744034379f60cd3b',
    LEDGER: '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357',
}
EXPECTED_PAIRS = (
    ('plain',
     'e14842248916f3d79770fba90adb18a6eb947ec4be052f04103b87acae1dd651',
     'ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862'),
    ('observer',
     '342aedf98ddcc8646cb50576abdf0cacbdd3a0cc5adf56e3931b678b91250a99',
     'cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6'),
)
FROZEN_GROUP_DEFINITIONS = {'assignment:NATIVE_GROUP_PROCESS_SOURCE_SHA256': 'f11286e0f96e0822906964c432138c2cb00567f8605faad7ef5393d17166a697', 'assignment:NATIVE_GROUP_PROCESS_COMPILED_SOURCE_SHA256': 'c70a2a85f5a61b077302fdb868befb2b73146577b2e2a9e1c1d9b5b46288ee6e', 'assignment:NATIVE_GROUP_PROCESS_NAMESPACE_EXPRESSIONS': '3e63e961d34f1f09a9ce4612b47c9a26d1c06fadcbd3aff94d657c040241459b', 'function:native_group_process_capture_files': '265ae7569e8a78956605d12be3482a916fbc45600cf91c5412d1097e76a9ffad'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def contract():
    if not FROZEN_FIXTURE.is_file() or FROZEN_FIXTURE.is_symlink():
        raise AssertionError('frozen Group capture fixture must be a regular file')
    raw = FROZEN_FIXTURE.read_bytes()
    if sha(raw) != FROZEN_FIXTURE_SHA256:
        raise AssertionError('frozen Group capture fixture differs')
    value = json.loads(raw)
    if value['schema'] != 'console.native_group_process_capture_contract.v1':
        raise AssertionError('frozen Group capture fixture schema differs')
    return value


def compact(sql):
    # Generated protocol inspection only, never a SQL interpreter. Capture and
    # embedded owner SQL are separately byte-pinned before their removal.
    return re.sub(r'\s+', ' ', re.sub(r'^\s*--[^\n]*$', '', sql, flags=re.M)).strip()


def definition_hashes(source):
    found = {}
    for node in ast.parse(source).body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name != 'main':
            key = 'function:' + node.name
        elif isinstance(node, ast.Assign) and all(isinstance(t, ast.Name) for t in node.targets):
            key = 'assignment:' + ','.join(t.id for t in node.targets)
        else:
            continue
        if key in found:
            raise AssertionError('definition shadowed: ' + key)
        found[key] = sha(ast.dump(node, include_attributes=False).encode())
    return found


def relation_rosters(sql):
    # The bounded finalizer's ordinary-relation VALUES rosters, not metadata
    # inside the separately pinned capture/source. CTE names remain arbitrary.
    pattern = (r'\b[a-z_0-9]+\s*\(name\)\s+AS\s*\(\s*VALUES\s*'
               r'((?:\(\s*\'[a-z_0-9]+\'\s*\)\s*,?\s*)+)\)')
    return [re.findall(r"\(\s*'([a-z_0-9]+)'\s*\)", match)
            for match in re.findall(pattern, sql)]


class NativeGroupServingCustodyGeneration(unittest.TestCase):
    def custody(self):
        entry = getattr(GENERATOR, 'native_group_process_custody_files', None)
        self.assertTrue(callable(entry), 'NATIVE_GROUP_SERVING_CUSTODY_FUNCTION_MISSING')
        result = entry()
        self.assertEqual(set(result), ARTIFACTS, 'additive serving protocol publishes exactly three artifacts')
        self.assertTrue(all(isinstance(value, str) and value for value in result.values()))
        self.assertEqual(result[CLASSIFIER].encode(), result[APP_CLASSIFIER].encode(),
                         'ops and application classifiers must be byte-identical')
        return result

    def test_exact_three_artifacts_embed_complete_frozen_capture_and_measured_pairs(self):
        files = self.custody()
        self.assertEqual(tuple(GENERATOR.NATIVE_GROUP_PROCESS_PHASE_PAIRS), EXPECTED_PAIRS)
        self.assertEqual(len({digest for row in EXPECTED_PAIRS for digest in row[1:]}), 4)
        for name, digest in INPUT_SHA256.items():
            path = ROOT / name
            self.assertTrue(path.is_file() and not path.is_symlink(), name)
            self.assertEqual(sha(path.read_bytes()), digest, name)
        query = files[CLASSIFIER]
        capture = (ROOT / CAPTURE).read_text().removesuffix(';\n')
        self.assertEqual(query.count(capture), 1, 'classifier must embed the complete exact83 capture once')
        phases = re.search(
            r'phase_pairs\s*\(variant\s*,\s*predecessor83\s*,\s*installed83\)'
            r'\s+AS\s*\((.*?)\n\)', query, re.S)
        self.assertIsNotNone(phases, 'declared paired measured phase relation missing')
        actual = re.findall(r"\('([^']+)','([0-9a-f]{64})','([0-9a-f]{64})'\)", compact(phases.group(1)))
        self.assertEqual(tuple(actual), EXPECTED_PAIRS, 'variant halves must remain paired and distinct')
        self.assertRegex(compact(phases.group(1)),
                         r"^VALUES \('[^']+','[0-9a-f]{64}','[0-9a-f]{64}'\), \('[^']+','[0-9a-f]{64}','[0-9a-f]{64}'\)$")

    def test_exact_matching_requires_false_predecessor_and_true_installed_rights(self):
        query = self.custody()[CLASSIFIER]
        match = re.search(
            r'matching_phase(?:\s*\(variant\s*,\s*phase\))?\s+AS\s*\((.*?)\n\),', query, re.S)
        self.assertIsNotNone(match, 'unique matching phase relation missing')
        branches = re.split(r'\bUNION ALL\b', compact(match.group(1)))
        self.assertEqual(len(branches), 2)
        observed = set()
        for branch in branches:
            aliases = re.search(r'FROM phase_pairs (\w+) CROSS JOIN full83 (\w+)', branch)
            self.assertIsNotNone(aliases, 'match must use the same complete83 capture and declared pair')
            pair, snapshot = aliases.groups()
            phase = re.search(r"SELECT " + re.escape(pair) + r"\.variant\s*,\s*'(installed|predecessor)'(?:\s*::\s*text)?", branch)
            self.assertIsNotNone(phase)
            phase = phase.group(1)
            observed.add(phase)
            expected_column, verdict = ('installed83', 'TRUE') if phase == 'installed' else ('predecessor83', 'FALSE')
            self.assertRegex(branch, re.escape(snapshot) + r'\.snapshot_sha256\s*=\s*'
                             + re.escape(pair) + r'\.' + expected_column)
            self.assertRegex(branch, re.escape(snapshot)
                             + r'\.native_group_process_startup_rights_valid\s+IS\s+' + verdict)
            self.assertNotRegex(branch, r'\b(?:COALESCE|OR)\b', 'NULL/alternate hash cannot become a matching phase')
        self.assertEqual(observed, {'installed', 'predecessor'})

    def test_complete_namespace_absence_is_fallback_not_install_authority(self):
        query, expected = self.custody()[CLASSIFIER], contract()
        absent = re.search(r'namespace_absence\s+AS\s*\((.*?)\n\)', query, re.S)
        self.assertIsNotNone(absent, 'all four reserved namespaces must be examined')
        predicate = compact(absent.group(1))
        for key in expected['reserved_namespace_expressions']:
            # jsonb_build_object emits explicit JSON null for jsonb_agg of an
            # empty namespace; ->> IS NULL or exact JSON-null comparison is
            # correct. -> IS NULL alone would reject the measured absence.
            self.assertRegex(predicate,
                r"(?:\w+\.)?snapshot\s*(?:->>\s*'" + key
                + r"'\s+IS\s+NULL|->\s*'" + key + r"'\s*=\s*'null'\s*::\s*jsonb)")
        self.assertNotRegex(predicate, r'\b(?:OR|COALESCE)\b', 'partial namespace absence must fail closed')
        for expression in expected['reserved_namespace_expressions'].values():
            self.assertIn(expression, query, 'every schema/owner/kind and all three prefixes remain captured')
        cases = list(re.finditer(r'\bSELECT\s+CASE\b', query))
        self.assertTrue(cases, 'classifier decision missing')
        tail = compact(query[cases[-1].start():])
        branches = dict((state, condition) for condition, state in re.findall(
            r"WHEN (.*?) THEN '(native_group_process\.[^']+)'", tail))
        self.assertEqual(set(branches), {
            'native_group_process.finalized', 'native_group_process.install_required', 'native_group_process.absent'})
        for state in ('native_group_process.finalized', 'native_group_process.install_required'):
            self.assertRegex(branches[state], r'count\(\*\) FROM matching_phase\)\s*=\s*1')
        self.assertIn("'installed'", branches['native_group_process.finalized'])
        self.assertNotIn('namespace_absence', branches['native_group_process.finalized'])
        self.assertIn("'predecessor'", branches['native_group_process.install_required'])
        for state in ('native_group_process.install_required', 'native_group_process.absent'):
            self.assertRegex(branches[state], r'valid FROM namespace_absence\) IS TRUE')
        self.assertRegex(branches['native_group_process.absent'],
                         r'count\(\*\) FROM matching_phase\)\s*=\s*0',
                         'mixed/duplicate matching phases cannot fall through to historical serving')
        self.assertRegex(tail, r"ELSE 'native_group_process\.profile_mismatch' END AS state;$")

    def test_historical_definitions_defaults_and_capture_contract_remain_frozen(self):
        self.custody()
        expected = contract()
        actual = definition_hashes(SCRIPT.read_text())
        for key, digest in {**expected['historical_generator_definitions'], **FROZEN_GROUP_DEFINITIONS}.items():
            self.assertEqual(actual.get(key), digest, key)
        defaults = GENERATOR.generated_files()
        self.assertEqual(set(defaults), set(expected['historical_default_outputs']))
        self.assertFalse(set(defaults) & ARTIFACTS, 'Group serving is opt-in, never a historical default')
        for name, digest in expected['historical_default_outputs'].items():
            self.assertEqual(sha(defaults[name].encode()), digest, name)
            self.assertEqual(sha((ROOT / name).read_bytes()), digest, name)
        pins = {row['path']: row for row in expected['all54_original_pins']}
        for name in expected['frozen48_paths']:
            raw = (ROOT / name).read_bytes()
            self.assertEqual((len(raw), sha(raw)), (pins[name]['size'], pins[name]['sha256']), name)
        self.assertIsNone(expected['phase_pairs'], 'historical capture fixture remains uninstalled')
        capture = GENERATOR.native_group_process_capture_files()
        for name in (OWNER, CAPTURE):
            self.assertEqual(sha(capture[name].encode()), INPUT_SHA256[name], name)
        ledger = (ROOT / LEDGER).read_bytes()
        self.assertEqual(len(ledger.splitlines()), 231)
        self.assertEqual(sha(b''.join(ledger.splitlines(keepends=True)[:230])),
                         '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325')

    def test_malformed_duplicate_mixed_and_unmeasured_phase_pairs_are_refused(self):
        self.custody()  # positive control before corruption
        plain, observer = EXPECTED_PAIRS
        cases = {
            'missing': (),
            'one_variant': (plain,),
            'wrong_width': (plain[:2], observer),
            'duplicate_variant': (plain, ('plain', *observer[1:])),
            'wrong_order': (observer, plain),
            'duplicate_hash': (plain, ('observer', plain[1], observer[2])),
            'null_hash': (('plain', None, plain[2]), observer),
            'short_hash': (('plain', plain[1][:-1], plain[2]), observer),
            'uppercase_hash': (('plain', plain[1].upper(), plain[2]), observer),
            'nonhex_hash': (('plain', 'g' * 64, plain[2]), observer),
            'reversed_phases': (('plain', plain[2], plain[1]), observer),
            'mixed_installed_variant': (('plain', plain[1], observer[2]), ('observer', observer[1], plain[2])),
            'mixed_predecessor_variant': (('plain', observer[1], plain[2]), ('observer', plain[1], observer[2])),
            'unmeasured_valid_hash': (('plain', '0' * 64, plain[2]), observer),
        }
        for name, value in cases.items():
            with self.subTest(corruption=name), patch.object(GENERATOR, 'NATIVE_GROUP_PROCESS_PHASE_PAIRS', value):
                with self.assertRaises((SystemExit, ValueError), msg=name):
                    GENERATOR.native_group_process_custody_files()

    def test_mutated_declared_sources_captures_ledger_and_migrations_are_refused(self):
        self.custody()  # refusal controls are meaningful only after positive generation
        expected = contract()
        migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
        self.assertEqual(len(migrations), 231)
        names = [row['path'] for row in expected['group_sql_sources'] + expected['compiled_source_identity']]
        names += [OWNER, CAPTURE, LEDGER, str(migrations[0].relative_to(ROOT)), str(migrations[-1].relative_to(ROOT))]
        self.assertEqual(len(names), 22)
        original = GENERATOR.company_provenance_regular_path
        for name in names:
            for corruption in ('changed', 'missing', 'symlink'):
                with self.subTest(source=name, corruption=corruption), tempfile.TemporaryDirectory() as temporary:
                    target = Path(temporary) / 'mutated-input'
                    raw = (ROOT / name).read_bytes()
                    if corruption == 'changed':
                        target.write_bytes(raw + b'\n')
                    elif corruption == 'symlink':
                        retained = Path(temporary) / 'retained-exact-input'
                        retained.write_bytes(raw)
                        target.symlink_to(retained)
                    def regular(requested, *, required):
                        if requested == name:
                            if target.is_symlink() or not target.is_file():
                                raise SystemExit('mutated input is not a regular file')
                            return target
                        return original(requested, required=required)
                    with patch.object(GENERATOR, 'company_provenance_regular_path', side_effect=regular):
                        with self.assertRaises((SystemExit, ValueError), msg=name + '/' + corruption):
                            GENERATOR.native_group_process_custody_files()

    def test_finalizer_retains_predecessor76_then_full83_for_install_and_replay(self):
        files = self.custody()
        query, finalizer = files[CLASSIFIER], files[FINALIZER]
        owner = (ROOT / OWNER).read_text()
        self.assertEqual(finalizer.count(owner), 1, 'execute exactly the declared14 owner bundle')
        source_at = finalizer.index(owner)
        delimiter = re.search(r'EXECUTE (\$[a-z_0-9]+\$)$', finalizer[:source_at])
        self.assertIsNotNone(delimiter, 'reviewed owner must be an exact EXECUTE dollar-quoted body')
        quoted_end = owner + delimiter.group(1) + ';'
        self.assertTrue(finalizer[source_at:].startswith(quoted_end))
        inspection = query.removesuffix(' AS state;\n')
        self.assertEqual(finalizer.count(inspection), 2,
                         'exact classifier runs under76 locks and again under83 locks')
        protocol = finalizer.replace(inspection, '__exact_classifier__')
        source_at = protocol.index(owner)
        before, after = protocol[:source_at], protocol[source_at + len(quoted_end):]
        prior = (ROOT / 'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql').read_text()
        roster = re.search(r'WITH wanted\(name\) AS \(VALUES\n(.*?)\n\), relations AS \(', prior, re.S)
        self.assertIsNotNone(roster)
        predecessor = sorted(re.findall(r"\('([a-z_0-9]+)'\)", roster.group(1)))
        all83 = sorted(predecessor + [name.split('.', 1)[1] for name in contract()['group_tables']])
        self.assertEqual((len(predecessor), len(set(predecessor))), (76, 76))
        self.assertEqual((len(all83), len(set(all83))), (83, 83))
        before_rosters, after_rosters = relation_rosters(before), relation_rosters(after)
        self.assertGreaterEqual(len(before_rosters), 3, 'predecessor shape, ordered lock and retained-lock census')
        self.assertTrue(all(names == predecessor for names in before_rosters),
                        'never require or lock the Group7 before its exact installation')
        self.assertGreaterEqual(len(after_rosters), 3, 'full83 shape, ordered lock and retained-lock census')
        self.assertTrue(all(names == all83 for names in after_rosters),
                        'both installed and replay paths retain exactly83 ordinary relations')
        for text, count in ((before, 76), (after, 83)):
            sql = compact(text)
            self.assertRegex(sql, r'FOR \w+ IN WITH')
            self.assertRegex(sql, r'ORDER BY \w+\.relname COLLATE "C"')
            self.assertEqual(sql.count('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE'), 1)
            self.assertRegex(sql, r'count\(\*\)\s*=\s*' + str(count))
            self.assertRegex(sql, r'count\(DISTINCT \w+\.relation\)\s*=\s*' + str(count))
            self.assertIn('JOIN pg_catalog.pg_locks', sql)
            for guard in (r"\w+\.pid\s*=\s*(?:pg_catalog\.)?pg_backend_pid\(\)",
                          r"\w+\.locktype\s*=\s*'relation'", r"\w+\.mode\s*=\s*'AccessExclusiveLock'", r'\w+\.granted'):
                self.assertRegex(sql, guard)
        before_sql, after_sql = compact(before), compact(after)
        received = re.search(r'INTO (\w+)\s*,\s*(\w+)', before_sql)
        self.assertIsNotNone(received)
        phase, variant = received.groups()
        self.assertRegex(before_sql, r'IF ' + re.escape(phase)
                         + r"\s*=\s*'native_group_process\.install_required' THEN EXECUTE ")
        self.assertRegex(before_sql, re.escape(variant) + r' IS NULL')
        saved = re.search(r'(\w+)\s*:=\s*' + re.escape(variant) + r'\s*;', before_sql)
        self.assertIsNotNone(saved, 'preserve the exact selected plain/observer variant before installation')
        self.assertNotRegex(before_sql, r'\bRETURN\s*;', 'replay must reach common full83 tail')
        self.assertRegex(before_sql, re.escape(phase) + r" IS DISTINCT FROM 'native_group_process\.finalized'")
        self.assertRegex(after_sql, re.escape(phase) + r" IS DISTINCT FROM 'native_group_process\.finalized'")
        self.assertRegex(after_sql, re.escape(variant) + r' IS DISTINCT FROM ' + re.escape(saved.group(1)))
        self.assertIn('SET CONSTRAINTS ALL IMMEDIATE;', after_sql)
        self.assertLess(after_sql.index('SET CONSTRAINTS ALL IMMEDIATE;'), after_sql.index('__exact_classifier__'))
        for boundary in (before_sql, after_sql):
            self.assertIn('FROM expected_migrations', boundary)
            self.assertIn('FULL JOIN public._sqlx_migrations', boundary)
            self.assertRegex(boundary, r'count\(\*\)\s*=\s*231')
            self.assertRegex(boundary, r'\w+\.success IS TRUE')
        records = [(int(version), checksum) for version, checksum in
                   (line.split('\t') for line in (ROOT / LEDGER).read_text().splitlines())]
        ledgers = re.findall(r'expected_migrations\(version,checksum\) AS \(\s*VALUES\s*(.*?)\n \)', protocol, re.S)
        self.assertEqual(len(ledgers), 2)
        for ledger in ledgers:
            actual = [(int(version), checksum) for version, checksum in re.findall(r"\((\d+),'([0-9a-f]{96})'\)", ledger)]
            self.assertEqual(actual, records, 'unchanged231 records before and after installation/replay')
        for value in ("current_user<>'console_buck_admin'", "rolsuper FROM pg_catalog.pg_roles", "IS NOT TRUE",
                      "starts_with(current_database(),'_sqlx_test_')", "'buck-sqlx-superuser-v1'",
                      "'read committed'", "'pg_catalog, pg_temp'", "'off'", "mode='ShareLock'"):
            self.assertIn(value, before_sql)
        for name, limit in (('lock_timeout', 1000), ('statement_timeout', 60000),
                            ('idle_in_transaction_session_timeout', 30000), ('transaction_timeout', 120000)):
            self.assertRegex(before_sql, r"current_setting\('" + name + r"'\).*?BETWEEN 1 AND " + str(limit))
        self.assertIn('actual', finalizer[:finalizer.index('DO ')])
        self.assertIn('console_account_owner', finalizer[:finalizer.index('DO ')])
        self.assertIn('FOR UPDATE', finalizer[:finalizer.index('DO ')])
        self.assertIn('trusted caller obligation', finalizer[:finalizer.index('DO ')])
        self.assertNotRegex(compact(protocol), r"locktype\s*=\s*'tuple'", 'role-row proof may not be manufactured from pg_locks')
        self.assertNotRegex(compact(protocol), r'\b(?:INSERT INTO|UPDATE|DELETE FROM|TRUNCATE(?: TABLE)?) public\._sqlx_migrations\b')


# Additive navigation correction stage: source input and two read-only
# classifiers only. Atomic finalizer and runtime acceptance remain separate.
NAVIGATION_SOURCE = 'ops/native-group-process/navigation-head-revision-v1.sql'
NAVIGATION_SOURCE_SHA256 = '83164dd8ac21cc30bbb7e6ed07c37fa184b148c4377fde22414740483cdb2f90'
NAVIGATION_CLASSIFIER = 'ops/postgres-native-group-process-navigation-v1-custody-state.sql'
NAVIGATION_APP_CLASSIFIER = 'backend/app/src/native_group_process_navigation_v1_custody_state.sql'
NAVIGATION_ARTIFACTS = {NAVIGATION_CLASSIFIER, NAVIGATION_APP_CLASSIFIER}
NAVIGATION_PAIRS = (
    ('plain',
     'ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862',
     '3f5972d2e5c1d7277e71b4f716b79a405d152ca5bbab25b614dbc1fc67c5fd7f'),
    ('observer',
     'cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6',
     '304e176d646edf62e8767a9f986879abc74753a3b2a8812c1a8cec8447a20b78'),
)
NAVIGATION_FROZEN_SERVING_DEFINITIONS = {
    'assignment:NATIVE_GROUP_PROCESS_PHASE_PAIRS': '601fae5af5d922be4a3c9775201b61f5b5563fb4fd954f0df24c37184b3fdca3',
    'assignment:NATIVE_GROUP_PROCESS_CAPTURE_SHA256': 'fc721e25c27c269920aa17c53e99b5f0cb0bf0bb739101bf662473c9cfff7a8a',
    'function:native_group_process_custody_inputs': 'e02f3c425b19ce5cbc6240fa8140a05cf0132dbba0454da16044afc8b0c72307',
    'function:native_group_process_state_query': '5bd08193d425e4a91848c931e2bd9bddcd1cb211a2bc840372f46007f4e9ba2e',
    'function:native_group_process_finalizer_sql': 'baea0f254e05d1fcf8d52abc99dd3e152807750fec2f41bc86c11f48f8a33636',
    'function:native_group_process_custody_files': '61bd48e6529973d4abf60b231f6df5b52b5460065b43a94ece7492677d5f887c',
}
NAVIGATION_FROZEN_SERVING_OUTPUTS = {
    CLASSIFIER: 'c44e239958fd5e716ddaec66459aba45df783fedf63b1d4d7939425fd018491d',
    APP_CLASSIFIER: 'c44e239958fd5e716ddaec66459aba45df783fedf63b1d4d7939425fd018491d',
    FINALIZER: '2f7d9463df35c8f5ffa679ffacff869b8724e2fe042e2bdeadd5498a4e0e8f65',
}


class NativeGroupNavigationCustodyGeneration(unittest.TestCase):
    def custody(self):
        entry = getattr(GENERATOR, 'native_group_process_navigation_custody_files', None)
        self.assertTrue(callable(entry), 'NATIVE_GROUP_NAVIGATION_CUSTODY_FUNCTION_MISSING')
        files = entry()
        self.assertEqual(set(files), NAVIGATION_ARTIFACTS,
                         'this bounded stage generates only the two successor classifiers')
        self.assertTrue(all(isinstance(value, str) and value for value in files.values()))
        self.assertEqual(files[NAVIGATION_CLASSIFIER].encode(), files[NAVIGATION_APP_CLASSIFIER].encode())
        return files

    @staticmethod
    def normalized_predicate(text):
        # Only the finite generated predicate grammar below. This does not
        # parse or establish semantic equivalence for arbitrary SQL.
        return re.sub(r'\s*(->>|->|::|,|=)\s*', r'\1', compact(text))

    def test_navigation_correction_is_one_exact_function_and_inverse_recovers_original(self):
        self.custody()  # Missing source is not the admitted missing-generator RED.
        path = ROOT / NAVIGATION_SOURCE
        self.assertTrue(path.is_file() and not path.is_symlink(), 'reviewed correction input required')
        raw = path.read_bytes()
        self.assertEqual(sha(raw), NAVIGATION_SOURCE_SHA256)
        owner = (ROOT / OWNER).read_text()
        self.assertEqual(sha(owner.encode()), INPUT_SHA256[OWNER])
        header = 'CREATE FUNCTION public.identity_native_group_process_navigation_candidates_v1(\n'
        terminator = '\n$body$;\n'
        self.assertEqual(owner.count(header), 1)
        remainder = owner.split(header, 1)[1]
        original = header + remainder[:remainder.index(terminator) + len(terminator)]
        self.assertEqual(sha(original.encode()),
                         '2e58ddbe2c6d4ee8b82f6a96aec9d8343a97a6db9d7074759d17c9d8bf38c1db')
        self.assertNotIn('v_navigation_head_revision', original)
        expected = original.replace('CREATE FUNCTION', 'CREATE OR REPLACE FUNCTION', 1)
        for before, after, count in (
            ('head_revision bigint', 'v_navigation_head_revision bigint', 1),
            ('head_revision:=', 'v_navigation_head_revision:=', 5),
            ('<>head_revision::numeric', '<>v_navigation_head_revision::numeric', 2),
            ('IF head_revision>0', 'IF v_navigation_head_revision>0', 2),
            ('h.head_revision<=head_revision ORDER BY h.head_revision',
             'h.head_revision<=v_navigation_head_revision ORDER BY h.head_revision', 1),
        ):
            self.assertEqual(expected.count(before), count, 'finite independently reviewed rename frame')
            expected = expected.replace(before, after)
        self.assertEqual(expected.count('v_navigation_head_revision'), 11)
        self.assertEqual(raw, expected.encode(), 'no other function, SQL, comments or source bytes')
        inverse = raw.decode().replace('v_navigation_head_revision', 'head_revision').replace(
            'CREATE OR REPLACE FUNCTION', 'CREATE FUNCTION', 1)
        self.assertEqual(inverse, original)
        self.assertEqual(raw.decode().count('CREATE OR REPLACE FUNCTION'), 1)
        self.assertEqual(raw.decode().count('AS $body$'), 1)
        self.assertEqual(raw.decode().count(terminator), 1)

    def test_navigation_two_artifacts_bind_exact_measured_same_variant_pairs_and_raw_capture(self):
        query = self.custody()[NAVIGATION_CLASSIFIER]
        self.assertEqual(tuple(GENERATOR.NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS), NAVIGATION_PAIRS)
        self.assertEqual(tuple((row[0], row[1]) for row in NAVIGATION_PAIRS),
                         tuple((row[0], row[2]) for row in EXPECTED_PAIRS))
        self.assertEqual(len({digest for row in NAVIGATION_PAIRS for digest in row[1:]}), 4)
        capture = (ROOT / CAPTURE).read_text().removesuffix(';\n')
        self.assertEqual(sha((ROOT / CAPTURE).read_bytes()), INPUT_SHA256[CAPTURE])
        self.assertEqual(query.count(capture), 1, 'one complete frozen raw83 capture, no alternate serializer')
        self.assertEqual(query.count('WITH wanted(name) AS (VALUES\n'), 1,
                         'a second independently truncated/modified capture cannot coexist')
        phases = re.search(
            r'phase_pairs\s*\(variant\s*,\s*predecessor83\s*,\s*installed83\)'
            r'\s+AS\s*\((.*?)\n\)', query, re.S)
        self.assertIsNotNone(phases)
        rows = re.findall(r"\('([^']+)','([0-9a-f]{64})','([0-9a-f]{64})'\)", compact(phases.group(1)))
        self.assertEqual(tuple(rows), NAVIGATION_PAIRS)
        self.assertRegex(compact(phases.group(1)),
                         r"^VALUES \('[^']+','[0-9a-f]{64}','[0-9a-f]{64}'\), \('[^']+','[0-9a-f]{64}','[0-9a-f]{64}'\)$")
        for forbidden in (NAVIGATION_SOURCE_SHA256,):
            self.assertNotIn(forbidden, compact(phases.group(1)),
                             'a source digest is not an observed database fingerprint')

    def test_navigation_both_phases_require_true_effective_rights(self):
        query = self.custody()[NAVIGATION_CLASSIFIER]
        matching = re.search(
            r'matching_phase(?:\s*\(variant\s*,\s*phase\))?\s+AS\s*\((.*?)\n\),', query, re.S)
        self.assertIsNotNone(matching)
        branches = re.split(r'\bUNION ALL\b', compact(matching.group(1)))
        self.assertEqual(len(branches), 2)
        observed = set()
        for branch in branches:
            aliases = re.search(r'FROM phase_pairs (\w+) CROSS JOIN full83 (\w+)', branch)
            self.assertIsNotNone(aliases)
            pair, snapshot = aliases.groups()
            phase = re.search(r"SELECT " + re.escape(pair)
                              + r"\.variant\s*,\s*'(installed|predecessor)'(?:\s*::\s*text)?", branch)
            self.assertIsNotNone(phase)
            phase = phase.group(1)
            observed.add(phase)
            column = 'installed83' if phase == 'installed' else 'predecessor83'
            expected = (f"SELECT {pair}.variant,'{phase}'::text "
                        f"FROM phase_pairs {pair} CROSS JOIN full83 {snapshot} "
                        f"WHERE {snapshot}.snapshot_sha256={pair}.{column} "
                        f"AND {snapshot}.native_group_process_startup_rights_valid IS TRUE")
            permitted = {self.normalized_predicate(expected),
                         self.normalized_predicate(expected.replace(f"'{phase}'::text", f"'{phase}'", 1))}
            self.assertIn(self.normalized_predicate(branch), permitted,
                          'the whole match is positive digest equality AND TRUE rights; no negation or alternate condition')
        self.assertEqual(observed, {'installed', 'predecessor'})

    def test_navigation_unique_match_and_complete_absence_are_only_disclosure_paths(self):
        query = self.custody()[NAVIGATION_CLASSIFIER]
        absent = re.search(r'namespace_absence\s+AS\s*\((.*?)\n\)', query, re.S)
        self.assertIsNotNone(absent)
        namespaces = contract()['reserved_namespace_expressions']
        expected_absence = 'SELECT ' + ' AND '.join(
            "snapshot->>'" + key + "' IS NULL" for key in namespaces) + ' AS valid FROM full83'
        self.assertEqual(self.normalized_predicate(absent.group(1)),
                         self.normalized_predicate(expected_absence),
                         'all four exact namespace-null predicates must be positively AND-conjoined')
        for expression in namespaces.values():
            self.assertIn(expression, query)
        cases = list(re.finditer(r'\bSELECT\s+CASE\b', query))
        self.assertTrue(cases)
        tail = compact(query[cases[-1].start():])
        expected_tail = """SELECT CASE
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='installed'
 THEN 'native_group_process_navigation.finalized'
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='predecessor'
 THEN 'native_group_process_navigation.head_revision_required'
 WHEN (SELECT count(*) FROM matching_phase)=0
  AND (SELECT valid FROM namespace_absence) IS TRUE
 THEN 'native_group_process_navigation.absent'
 ELSE 'native_group_process_navigation.profile_mismatch' END AS state;"""
        self.assertEqual(self.normalized_predicate(tail), self.normalized_predicate(expected_tail),
                         'complete dispatch requires exact positive phase equality, unique matching and bounded absence fallback')

    def test_navigation_preserves_all_existing_definitions_defaults_sources_pairs_and_outputs(self):
        files = self.custody()
        expected = contract()
        frozen = {**expected['historical_generator_definitions'], **FROZEN_GROUP_DEFINITIONS,
                  **NAVIGATION_FROZEN_SERVING_DEFINITIONS}
        self.assertEqual(len(frozen), 146)
        actual = definition_hashes(SCRIPT.read_text())
        for key, digest in frozen.items():
            self.assertEqual(actual.get(key), digest, key)
        defaults = GENERATOR.generated_files()
        self.assertEqual(set(defaults), set(expected['historical_default_outputs']))
        self.assertFalse(set(defaults) & set(files), 'successor is opt-in, never a default rewrite')
        for name, digest in expected['historical_default_outputs'].items():
            self.assertEqual(sha(defaults[name].encode()), digest, name)
            self.assertEqual(sha((ROOT / name).read_bytes()), digest, name)
        self.assertEqual(tuple(GENERATOR.NATIVE_GROUP_PROCESS_PHASE_PAIRS), EXPECTED_PAIRS)
        current = GENERATOR.native_group_process_custody_files()
        self.assertEqual(set(current), set(NAVIGATION_FROZEN_SERVING_OUTPUTS))
        for name, digest in NAVIGATION_FROZEN_SERVING_OUTPUTS.items():
            self.assertEqual(sha(current[name].encode()), digest, name)
            self.assertEqual(sha((ROOT / name).read_bytes()), digest, name)
        for row in expected['group_sql_sources'] + expected['compiled_source_identity']:
            self.assertEqual(sha((ROOT / row['path']).read_bytes()), row['sha256'], row['path'])
        ledger = (ROOT / LEDGER).read_bytes()
        self.assertEqual(len(ledger.splitlines()), 231)
        self.assertEqual(sha(ledger), INPUT_SHA256[LEDGER])
        self.assertEqual(sha(b''.join(ledger.splitlines(keepends=True)[:230])),
                         '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325')

    def test_navigation_malformed_duplicate_mixed_and_unmeasured_pairs_are_refused(self):
        self.custody()
        plain, observer = NAVIGATION_PAIRS
        cases = {
            'missing': (), 'one_variant': (plain,), 'wrong_width': (plain[:2], observer),
            'duplicate_variant': (plain, ('plain', *observer[1:])), 'wrong_order': (observer, plain),
            'duplicate_hash': (plain, ('observer', plain[1], observer[2])),
            'null_hash': (('plain', None, plain[2]), observer),
            'short_hash': (('plain', plain[1][:-1], plain[2]), observer),
            'uppercase_hash': (('plain', plain[1].upper(), plain[2]), observer),
            'nonhex_hash': (('plain', 'g' * 64, plain[2]), observer),
            'reversed_phases': (('plain', plain[2], plain[1]), observer),
            'mixed_corrected_variant': (('plain', plain[1], observer[2]), ('observer', observer[1], plain[2])),
            'mixed_predecessor_variant': (('plain', observer[1], plain[2]), ('observer', plain[1], observer[2])),
            'unmeasured_valid_hash': (('plain', plain[1], '0' * 64), observer),
            'old_absent_predecessor': (('plain', EXPECTED_PAIRS[0][1], plain[2]), observer),
            'equal_phases': (('plain', plain[1], plain[1]), observer),
            'list_instead_of_tuple': [plain, observer],
        }
        for name, value in cases.items():
            with self.subTest(corruption=name), patch.object(
                    GENERATOR, 'NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS', value):
                with self.assertRaises((SystemExit, ValueError), msg=name):
                    GENERATOR.native_group_process_navigation_custody_files()

    def test_navigation_corrupted_inputs_and_pins_are_refused_after_positive_generation(self):
        self.custody()
        expected = contract()
        migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
        self.assertEqual(len(migrations), 231)
        names = [row['path'] for row in expected['group_sql_sources'] + expected['compiled_source_identity']]
        names += [OWNER, CAPTURE, LEDGER, str(migrations[0].relative_to(ROOT)),
                  str(migrations[-1].relative_to(ROOT)), NAVIGATION_SOURCE]
        self.assertEqual((len(names), len(set(names))), (23, 23))
        original = GENERATOR.company_provenance_regular_path
        for name in names:
            for corruption in ('changed', 'missing', 'symlink'):
                with self.subTest(source=name, corruption=corruption), tempfile.TemporaryDirectory() as temporary:
                    target = Path(temporary) / 'mutated-input'
                    raw = (ROOT / name).read_bytes()
                    if corruption == 'changed':
                        target.write_bytes(raw + b'\n')
                    elif corruption == 'symlink':
                        retained = Path(temporary) / 'retained-exact-input'
                        retained.write_bytes(raw)
                        target.symlink_to(retained)
                    def regular(requested, *, required):
                        if requested == name:
                            self.assertTrue(required, 'all correction and original inputs are mandatory')
                            if target.is_symlink() or not target.is_file():
                                raise SystemExit('mutated input is not a regular file')
                            return target
                        return original(requested, required=required)
                    with patch.object(GENERATOR, 'company_provenance_regular_path', side_effect=regular):
                        with self.assertRaises((SystemExit, ValueError), msg=name + '/' + corruption):
                            GENERATOR.native_group_process_navigation_custody_files()
        for attribute in ('NATIVE_GROUP_PROCESS_SOURCE_SHA256',
                          'NATIVE_GROUP_PROCESS_COMPILED_SOURCE_SHA256', 'NATIVE_GROUP_PROCESS_CAPTURE_SHA256'):
            pins = getattr(GENERATOR, attribute)
            for name in pins:
                with self.subTest(pin=attribute, source=name), patch.dict(pins, {name: '0' * 64}):
                    with self.assertRaises((SystemExit, ValueError), msg=attribute + '/' + name):
                        GENERATOR.native_group_process_navigation_custody_files()
        self.assertEqual(GENERATOR.NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256, NAVIGATION_SOURCE_SHA256)
        with patch.object(GENERATOR, 'NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256', '0' * 64):
            with self.assertRaises((SystemExit, ValueError)):
                GENERATOR.native_group_process_navigation_custody_files()

    def test_navigation_cli_is_explicit_two_artifact_opt_in_with_exact_check(self):
        files = self.custody()
        with tempfile.TemporaryDirectory() as temporary:
            paths = {name: Path(temporary) / Path(name).name for name in files}
            self.assertEqual(len(set(paths.values())), 2)
            def output_path(name, *, required):
                self.assertIn(name, files)
                self.assertFalse(required)
                return paths[name]
            with patch.object(GENERATOR, 'native_group_process_navigation_custody_files', return_value=files) as selected, \
                    patch.object(GENERATOR, 'company_provenance_regular_path', side_effect=output_path), \
                    patch.object(GENERATOR, 'generated_files', side_effect=AssertionError('opt-in may not select defaults')):
                with patch.object(GENERATOR.sys, 'argv', [str(SCRIPT), '--native-group-process-navigation-custody']):
                    GENERATOR.main()
                self.assertEqual(selected.call_count, 1)
                for name, path in paths.items():
                    self.assertEqual(path.read_bytes(), files[name].encode())
                with patch.object(GENERATOR.sys, 'argv',
                                  [str(SCRIPT), '--native-group-process-navigation-custody', '--check']):
                    GENERATOR.main()
                self.assertEqual(selected.call_count, 2)
                damaged = paths[NAVIGATION_CLASSIFIER]
                damaged.write_bytes(damaged.read_bytes() + b'\n')
                with patch.object(GENERATOR.sys, 'argv',
                                  [str(SCRIPT), '--native-group-process-navigation-custody', '--check']):
                    with self.assertRaises(SystemExit):
                        GENERATOR.main()
                self.assertEqual(selected.call_count, 3)
                damaged.write_bytes(files[NAVIGATION_CLASSIFIER].encode())
                paths[NAVIGATION_APP_CLASSIFIER].unlink()
                with patch.object(GENERATOR.sys, 'argv',
                                  [str(SCRIPT), '--native-group-process-navigation-custody', '--check']):
                    with self.assertRaises(SystemExit):
                        GENERATOR.main()
                self.assertEqual(selected.call_count, 4)
                paths[NAVIGATION_APP_CLASSIFIER].write_bytes(files[NAVIGATION_APP_CLASSIFIER].encode())
                with patch.object(GENERATOR.sys, 'argv',
                                  [str(SCRIPT), '--native-group-process-navigation-custody', '--check']):
                    GENERATOR.main()
                self.assertEqual(selected.call_count, 5, 'positive check follows each independently repaired fault')


if __name__ == '__main__':
    unittest.main()
