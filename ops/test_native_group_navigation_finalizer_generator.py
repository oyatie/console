"""Bounded Group navigation finalizer source contract, not PostgreSQL proof.

Replay/rollback/cancellation, target custody and concurrent catalog histories
require the separately admitted native database tests. No rows are populated.
"""
from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch

import test_native_group_process_serving_custody_generator as serving
from test_company_provenance_capture_generator import inventory

ROOT, SCRIPT, GENERATOR = serving.ROOT, serving.SCRIPT, serving.GENERATOR
sha, contract, compact = serving.sha, serving.contract, serving.compact
FINALIZER = 'ops/postgres-finalize-native-group-process-navigation-v1.sql'
CLASSIFIER_SHA256 = '6a7a721c434486582ffba11d79a818088c3f3a41b93248b4bb1504ecd484dc39'
FROZEN_NAVIGATION_DEFINITIONS = {
    'assignment:NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE':
        'da397904a0881c8b3c40f9d79076c17e59babef43b6784b5421e9f0346e64128',
    'assignment:NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256':
        'd314460f88ef9041b6e28c836ba8c21b681928fe8c24e062ee4c9d885c15f6c2',
    'assignment:NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS':
        'caea717c55ad24b360b2d9645ca6ab68c550b381eef32cde1af2b95793b501a6',
    'function:native_group_process_navigation_custody_files':
        '43580f5a88ee5ee6de9e88147ef796a4d7bf7e22865a0508134d3968b9ac8511',
}

# An explicit finite outer protocol, with separately exact-pinned SQL bodies
# replaced by markers. Equality checks the complete predicates and sequencing;
# finding a reassuring substring is insufficient. This is not a SQL parser.
ENTRY = """DO $native_group_process_navigation_custody$
DECLARE observed_phase text; variant_name text; expected_variant text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user IS DISTINCT FROM current_user OR current_user<>'console_buck_admin'
  OR (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) IS NOT TRUE
  OR starts_with(current_database(),'_sqlx_test_') IS NOT TRUE
  OR pg_catalog.current_setting('console.sqlx_test_bootstrap',true)
      IS DISTINCT FROM 'buck-sqlx-superuser-v1' THEN
  RAISE EXCEPTION 'native_group_process_navigation.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR pg_catalog.current_setting('search_path') IS DISTINCT FROM 'pg_catalog, pg_temp'
  OR pg_catalog.current_setting('jit') IS DISTINCT FROM 'off' THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_settings_mismatch'; END IF;
"""
BOUNDS = '\n'.join(""" IF (pg_catalog.current_setting('%s') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='%s')
      BETWEEN 1 AND %d) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_bounds_mismatch'; END IF;"""
    % (name, name, bound) for name, bound in (
        ('lock_timeout', 1000), ('statement_timeout', 60000),
        ('idle_in_transaction_session_timeout', 30000), ('transaction_timeout', 120000)))
LEDGER_LOCK = """IF (SELECT count(*)=1 AND bool_and(c.oid IS NOT NULL AND c.oid>0
      AND c.relkind='r' AND NOT c.relispartition AND r.rolname='console_app') IS TRUE
     FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
     JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
     WHERE n.nspname='public' AND c.relname='_sqlx_migrations') IS NOT TRUE
  OR (SELECT count(*)=1 FROM pg_catalog.pg_locks
      WHERE pid=pg_backend_pid() AND locktype='relation'
       AND relation=pg_catalog.to_regclass('public._sqlx_migrations')
       AND mode='ShareLock' AND granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_lock_missing'; END IF;"""
RELATION_SHAPE = """IF (WITH required_relations(name) AS (
__exact83__
 ) SELECT count(*)=83 AND count(DISTINCT c.oid)=83 AND count(DISTINCT n.oid)=1
    AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
        AND NOT c.relispartition AND n.nspname='public') IS TRUE
   FROM required_relations required
   LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;"""
LOCK_SCAN = """locked_relations:=0;
 FOR relation_name IN WITH required_relations(name) AS (
__exact83__
 ) SELECT c.relname::text FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;"""
RETAINED_LOCKS = """IF locked_relations<>83 OR (WITH required_relations(name) AS (
__exact83__
 ) SELECT count(*)=83 AND count(DISTINCT l.relation)=83
   FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   JOIN pg_catalog.pg_locks l ON l.relation=c.oid
   WHERE l.pid=pg_backend_pid() AND l.locktype='relation'
    AND l.mode='AccessExclusiveLock' AND l.granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.relation_locks_missing'; END IF;"""
INSPECT = """SELECT classified.state,classified.variant INTO observed_phase,variant_name FROM (
__exact_classifier__ AS state,(SELECT variant FROM matching_phase) AS variant
) classified;"""
DECIDE_AND_EXECUTE = """IF (observed_phase IS DISTINCT FROM 'native_group_process_navigation.head_revision_required'
     AND observed_phase IS DISTINCT FROM 'native_group_process_navigation.finalized') OR variant_name IS NULL THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 expected_variant:=variant_name;
 IF observed_phase='native_group_process_navigation.head_revision_required' THEN
  EXECUTE $native_group_process_navigation_source$__exact_correction__$native_group_process_navigation_source$;
 END IF;
 SET CONSTRAINTS ALL IMMEDIATE;"""
POSTCHECK = """IF observed_phase IS DISTINCT FROM 'native_group_process_navigation.finalized'
  OR variant_name IS DISTINCT FROM expected_variant THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;"""
EXPECTED_OUTER = '\n'.join((ENTRY, BOUNDS, LEDGER_LOCK, '__exact231__', RELATION_SHAPE,
    LOCK_SCAN, RETAINED_LOCKS, INSPECT, DECIDE_AND_EXECUTE, INSPECT, POSTCHECK,
    '__exact231__', RETAINED_LOCKS, 'END\n$native_group_process_navigation_custody$;'))


class NativeGroupNavigationFinalizerGeneration(unittest.TestCase):
    def finalizer(self):
        entry = getattr(GENERATOR, 'native_group_process_navigation_finalizer_files', None)
        self.assertTrue(callable(entry), 'NATIVE_GROUP_NAVIGATION_FINALIZER_FUNCTION_MISSING')
        files = entry()
        self.assertEqual(set(files), {FINALIZER}, 'one distinct opt-in corrective finalizer only')
        self.assertTrue(isinstance(files[FINALIZER], str) and files[FINALIZER])
        return files

    @staticmethod
    def normalized(text):
        # Finite protocol grammar only; identifiers/formatting are not custody.
        return serving.NativeGroupNavigationCustodyGeneration.normalized_predicate(
            text.replace('pg_catalog.', ''))

    def assert_outer(self, outer):
        sql = compact(outer)
        guard_pattern = (r"\bIF\s+((?:(?!\b(?:IF|THEN|END)\b).)*?)\s+THEN\s+"
                         r"RAISE EXCEPTION '([^']+)';\s*END IF;")
        guards = list(re.finditer(guard_pattern, sql))
        def expected_guard(fragment):
            found = list(re.finditer(guard_pattern, compact(fragment)))
            self.assertEqual(len(found), 1)
            return found[0].groups()
        def matches(fragment):
            predicate, error = expected_guard(fragment)
            return [item for item in guards if item.group(2) == error
                    and self.normalized(item.group(1)) == self.normalized(predicate)]
        # Full finite predicates rather than checks for isolated reassuring
        # terms. The wrapper can reuse/repeat the exact lock checks as needed.
        required = [fragment.group(0) for fragment in re.finditer(guard_pattern, compact(ENTRY))]
        required += [fragment.group(0) for fragment in re.finditer(guard_pattern, compact(BOUNDS))]
        required += [LEDGER_LOCK, RELATION_SHAPE, RETAINED_LOCKS, DECIDE_AND_EXECUTE.split(' expected_variant:', 1)[0], POSTCHECK]
        allowed = set()
        for fragment in required:
            predicate, error = expected_guard(fragment)
            allowed.add((self.normalized(predicate), error))
        for guard in guards:
            self.assertIn((self.normalized(guard.group(1)), guard.group(2)), allowed,
                          'each whole predicate must enforce a known positive boundary')
        for fragment in required[:7]:
            self.assertEqual(len(matches(fragment)), 1, 'one complete entry/operator/bounds/ledger-lock guard')
        source = compact("EXECUTE $native_group_process_navigation_source$__exact_correction__$native_group_process_navigation_source$;")
        self.assertEqual(sql.count(source), 1)
        source_at = sql.index(source)
        self.assertTrue(all(matches(fragment)[0].end() < source_at for fragment in required[:7]))
        inspections = list(re.finditer(re.escape(compact(INSPECT)), sql))
        self.assertEqual(len(inspections), 2, 'same complete classified state and paired variant twice')
        before_at, after_at = inspections[0].start(), inspections[1].start()
        self.assertLess(before_at, source_at)
        self.assertLess(source_at, after_at)
        shapes, retained = matches(RELATION_SHAPE), matches(RETAINED_LOCKS)
        self.assertTrue(shapes and min(item.start() for item in shapes) < before_at,
                        'exact83 ordinary-relation metadata census precedes the first classifier')
        self.assertTrue(any(item.end() < before_at for item in retained), 'all83 locks verified before classification')
        self.assertTrue(any(item.start() > after_at for item in retained), 'all83 retained locks checked on common tail')
        pre = matches(required[-2])
        post = matches(POSTCHECK)
        self.assertEqual((len(pre), len(post)), (1, 1))
        self.assertLess(inspections[0].end(), pre[0].start())
        self.assertLess(pre[0].end(), source_at)
        self.assertLess(inspections[1].end(), post[0].start())
        scan = compact(LOCK_SCAN)
        scans = list(re.finditer(re.escape(scan), sql))
        self.assertTrue(scans and any(item.end() < before_at for item in scans),
                        'deterministic exact83 C-ordered AccessExclusive scan precedes mutation')
        decision = compact(DECIDE_AND_EXECUTE)
        self.assertEqual(sql.count(decision), 1, 'only head-revision-required executes correction; no replay early return')
        ledgers = list(re.finditer('__exact231__', sql))
        self.assertEqual(len(ledgers), 2)
        self.assertLess(ledgers[0].end(), before_at)
        self.assertGreater(ledgers[1].start(), post[0].end())
        # Strip independently checked statements and permit only a non-effecting
        # DO/DECLARE/BEGIN/END wrapper. No hidden function call or second writer
        # is admitted merely because the expected statements are also present.
        residue = sql
        for fragment in [guard.group(0) for guard in guards] + [compact(INSPECT), scan,
                compact(DECIDE_AND_EXECUTE[len(DECIDE_AND_EXECUTE.split(' expected_variant:', 1)[0]):]),
                '__exact231__']:
            residue = residue.replace(fragment, '')
        residue = compact(residue)
        self.assertRegex(residue,
            r'^DO (\$[a-z_0-9]+\$) DECLARE (?:[a-z_0-9]+ (?:text|integer\s*:=\s*0);\s*)+ BEGIN\s+END \1;$',
            'no executable remainder outside checked protocol statements')

    def protocol(self, sql):
        source = (ROOT / serving.NAVIGATION_SOURCE).read_text()
        self.assertEqual(sha(source.encode()), serving.NAVIGATION_SOURCE_SHA256)
        query = (ROOT / serving.NAVIGATION_CLASSIFIER).read_text()
        self.assertEqual(sha(query.encode()), CLASSIFIER_SHA256)
        self.assertTrue(query.endswith(' AS state;\n'))
        inspection = query.removesuffix(' AS state;\n')
        self.assertEqual(sql.count(source), 1, 'one exact correction, no broader owner bundle')
        self.assertEqual(sql.count(inspection), 2, 'the same pinned complete classifier before and after')
        self.assertIn('EXECUTE $native_group_process_navigation_source$' + source
                      + '$native_group_process_navigation_source$;', sql)
        outer = sql.replace(source, '__exact_correction__').replace(inspection, '__exact_classifier__')
        ledger = (ROOT / serving.LEDGER).read_bytes()
        self.assertEqual(sha(ledger), serving.INPUT_SHA256[serving.LEDGER])
        rows = [line.split('\t') for line in ledger.decode().splitlines()]
        self.assertEqual(len(rows), 231)
        values = 'VALUES\n ' + ',\n '.join('(' + version + ",'" + checksum + "')" for version, checksum in rows)
        ledger_guard = """IF (WITH expected_migrations(version,checksum) AS (
%s
 ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
    AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
   FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_mismatch'; END IF;""" % values
        self.assertEqual(outer.count(ledger_guard), 2, 'exact successful231 full-join ledger before and after')
        outer = outer.replace(ledger_guard, '__exact231__')
        capture = (ROOT / serving.CAPTURE).read_text()
        self.assertEqual(sha(capture.encode()), serving.INPUT_SHA256[serving.CAPTURE])
        roster = re.search(r'WITH wanted\(name\) AS \(VALUES\n(.*?)\n\), relations AS \(', capture, re.S)
        self.assertIsNotNone(roster)
        names = sorted(re.findall(r"\('([a-z_0-9]+)'\)", roster.group(1)))
        self.assertEqual((len(names), len(set(names))), (83, 83))
        rosters = serving.relation_rosters(outer)
        self.assertGreaterEqual(len(rosters), 4, 'shape, scan, initial and common final lock census')
        self.assertTrue(all(roster == names for roster in rosters), 'every roster is exactly the frozen83')
        values = 'VALUES\n ' + ',\n '.join("('" + name + "')" for name in names)
        self.assertEqual(outer.count(values), len(rosters))
        outer = outer.replace(values, '__exact83__')
        self.assert_outer(outer)
        return outer

    def test_pinned_execution_entry_locks_replay_and_common_postchecks(self):
        sql = self.finalizer()[FINALIZER]
        outer = compact(self.protocol(sql))
        # The SQL cannot establish these caller obligations through fake tuple
        # locks. Native target/role custody tests must exercise the real caller.
        header = sql.split('DO $native_group_process_navigation_custody$', 1)[0]
        documentation = re.sub(r'\s+', ' ', re.sub(r'^\s*--\s?', '', header, flags=re.M))
        for obligation in ('actual database name/OID/system_identifier',
                           'cluster schema/role maintenance lease', 'BEGINs READ COMMITTED',
                           'locks the ledger SHARE and actual console_account_owner',
                           'pg_authid row FOR UPDATE', 'Retain all locks through COMMIT/ROLLBACK',
                           'Role-row custody is a trusted caller obligation',
                           'no fabricated pg_locks tuple proof',
                           'does not authorize production DDL or client exposure'):
            self.assertIn(obligation, documentation, obligation)
        self.assertNotRegex(outer, r"locktype\s*=\s*'tuple'|\bRETURN\b")
        for forbidden in (r'\b(?:INSERT|UPDATE|DELETE|TRUNCATE|COPY)\b',
                          r'\b(?:GRANT|REVOKE|ALTER\s+(?:ROLE|OWNER)|SET\s+ROLE)\b',
                          r'\b(?:COMMIT|ROLLBACK)\b'):
            self.assertNotRegex(outer, forbidden,
                'outer maintenance wrapper adds no ledger/business DML, privileges or transaction boundary')

    def test_preserves150_existing_definitions_and_all_historical_outputs(self):
        self.finalizer()
        expected = contract()
        frozen = {**expected['historical_generator_definitions'], **serving.FROZEN_GROUP_DEFINITIONS,
                  **serving.NAVIGATION_FROZEN_SERVING_DEFINITIONS, **FROZEN_NAVIGATION_DEFINITIONS}
        self.assertEqual(len(frozen), 150)
        definitions = serving.definition_hashes(SCRIPT.read_text())
        for name, digest in frozen.items():
            self.assertEqual(definitions.get(name), digest, name)
        defaults = GENERATOR.generated_files()
        self.assertEqual(len(defaults), 33)
        self.assertEqual(set(defaults), set(expected['historical_default_outputs']))
        for name, digest in expected['historical_default_outputs'].items():
            self.assertEqual(sha(defaults[name].encode()), digest, name)
            self.assertEqual(sha((ROOT / name).read_bytes()), digest, name)
        old = GENERATOR.native_group_process_custody_files()
        self.assertEqual(set(old), set(serving.NAVIGATION_FROZEN_SERVING_OUTPUTS))
        for name, digest in serving.NAVIGATION_FROZEN_SERVING_OUTPUTS.items():
            self.assertEqual(sha(old[name].encode()), digest, name)
            self.assertEqual(sha((ROOT / name).read_bytes()), digest, name)
        navigation = GENERATOR.native_group_process_navigation_custody_files()
        self.assertEqual(set(navigation), serving.NAVIGATION_ARTIFACTS)
        for name, value in navigation.items():
            self.assertEqual(sha(value.encode()), CLASSIFIER_SHA256, name)
            self.assertEqual(sha((ROOT / name).read_bytes()), CLASSIFIER_SHA256, name)
        pins = {row['path']: row for row in expected['all54_original_pins']}
        for name in expected['frozen48_paths']:
            raw = (ROOT / name).read_bytes()
            self.assertEqual((len(raw), sha(raw)), (pins[name]['size'], pins[name]['sha256']), name)
        for row in expected['group_sql_sources'] + expected['compiled_source_identity']:
            self.assertEqual(sha((ROOT / row['path']).read_bytes()), row['sha256'], row['path'])
        ledger = (ROOT / serving.LEDGER).read_bytes()
        self.assertEqual((len(ledger.splitlines()), sha(ledger)),
                         (231, serving.INPUT_SHA256[serving.LEDGER]))
        self.assertEqual(sha(b''.join(ledger.splitlines(keepends=True)[:230])),
                         '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325')

    def test_refuses_source_ledger_pin_and_pair_corruption_after_positive_generation(self):
        self.finalizer()
        expected = contract()
        migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
        self.assertEqual(len(migrations), 231)
        names = [row['path'] for row in expected['group_sql_sources'] + expected['compiled_source_identity']]
        names += [serving.OWNER, serving.CAPTURE, serving.LEDGER, serving.NAVIGATION_SOURCE,
                  str(migrations[0].relative_to(ROOT)), str(migrations[-1].relative_to(ROOT))]
        self.assertEqual((len(names), len(set(names))), (23, 23))
        regular = GENERATOR.company_provenance_regular_path
        for name in names:
            for corruption in ('changed', 'missing', 'symlink'):
                with self.subTest(input=name, corruption=corruption), tempfile.TemporaryDirectory() as temporary:
                    target = Path(temporary) / 'input'
                    raw = (ROOT / name).read_bytes()
                    if corruption == 'changed':
                        target.write_bytes(raw + b'\n')
                    elif corruption == 'symlink':
                        retained = Path(temporary) / 'exact'
                        retained.write_bytes(raw)
                        target.symlink_to(retained)
                    def altered(requested, *, required):
                        if requested == name:
                            self.assertTrue(required)
                            if target.is_symlink() or not target.is_file():
                                raise SystemExit('not a regular mandatory input')
                            return target
                        return regular(requested, required=required)
                    with patch.object(GENERATOR, 'company_provenance_regular_path', side_effect=altered):
                        with self.assertRaises((SystemExit, ValueError)):
                            GENERATOR.native_group_process_navigation_finalizer_files()
        for attribute in ('NATIVE_GROUP_PROCESS_SOURCE_SHA256', 'NATIVE_GROUP_PROCESS_COMPILED_SOURCE_SHA256',
                          'NATIVE_GROUP_PROCESS_CAPTURE_SHA256'):
            pins = getattr(GENERATOR, attribute)
            for name in pins:
                with self.subTest(pin=attribute, input=name), patch.dict(pins, {name: '0' * 64}):
                    with self.assertRaises((SystemExit, ValueError)):
                        GENERATOR.native_group_process_navigation_finalizer_files()
        with patch.object(GENERATOR, 'NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256', '0' * 64):
            with self.assertRaises((SystemExit, ValueError)):
                GENERATOR.native_group_process_navigation_finalizer_files()
        plain, observer = serving.NAVIGATION_PAIRS
        for name, pairs in {
            'missing': (), 'wrong_order': (observer, plain),
            'duplicate_variant': (plain, ('plain', *observer[1:])),
            'wrong_type': [plain, observer], 'wrong_width': (plain[:2], observer),
            'old_absent': (('plain', serving.EXPECTED_PAIRS[0][1], plain[2]), observer),
            'mixed_corrected': (('plain', plain[1], observer[2]), ('observer', observer[1], plain[2])),
            'reversed': (('plain', plain[2], plain[1]), observer),
            'unmeasured': (('plain', plain[1], '0' * 64), observer),
        }.items():
            with self.subTest(pair=name), patch.object(GENERATOR, 'NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS', pairs):
                with self.assertRaises((SystemExit, ValueError)):
                    GENERATOR.native_group_process_navigation_finalizer_files()
        self.finalizer()  # restoration must recover positive generation

    def test_cli_single_artifact_opt_in_check_and_independent_restoration(self):
        files = self.finalizer()
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary) / FINALIZER
            target.parent.mkdir(parents=True)
            regular = GENERATOR.company_provenance_regular_path
            with patch.object(GENERATOR, 'native_group_process_navigation_finalizer_files', return_value=files) as selected, \
                    patch.object(GENERATOR, 'ROOT', Path(temporary)), \
                    patch.object(GENERATOR, 'company_provenance_regular_path', wraps=regular) as output_paths, \
                    patch.object(GENERATOR, 'generated_files', side_effect=AssertionError('no default selection')), \
                    patch.object(GENERATOR, 'native_group_process_navigation_custody_files',
                                 side_effect=AssertionError('no classifier-output selection')):
                argv = [str(SCRIPT), '--native-group-process-navigation-finalizer']
                with patch.object(GENERATOR.sys, 'argv', argv):
                    GENERATOR.main()
                self.assertEqual(target.read_bytes(), files[FINALIZER].encode())
                before = inventory(Path(temporary))
                with patch.object(GENERATOR.sys, 'argv', argv + ['--check']):
                    GENERATOR.main()
                self.assertEqual(inventory(Path(temporary)), before, '--check success must not mutate the private tree')
                for fault in ('changed', 'missing', 'symlink'):
                    if fault == 'changed':
                        target.write_bytes(files[FINALIZER].encode() + b'\n')
                    else:
                        target.unlink()
                        if fault == 'symlink':
                            retained = Path(temporary) / 'exact.sql'
                            retained.write_bytes(files[FINALIZER].encode())
                            target.symlink_to(retained)
                    before = inventory(Path(temporary))
                    with self.subTest(fault=fault), patch.object(GENERATOR.sys, 'argv', argv + ['--check']):
                        with self.assertRaises(SystemExit):
                            GENERATOR.main()
                        self.assertEqual(inventory(Path(temporary)), before,
                                         '--check refusal must not mutate the private tree before restoration')
                    if target.is_symlink():
                        target.unlink()
                    target.write_bytes(files[FINALIZER].encode())
                    before = inventory(Path(temporary))
                    with patch.object(GENERATOR.sys, 'argv', argv + ['--check']):
                        GENERATOR.main()
                    self.assertEqual(inventory(Path(temporary)), before, '--check after restoration must not mutate the private tree')
                self.assertEqual(selected.call_count, 8)
                self.assertEqual(output_paths.call_count, 8)
                for call in output_paths.call_args_list:
                    self.assertEqual(call.args, (FINALIZER,))
                    self.assertEqual(call.kwargs, {'required': False})

    def test_complete_outer_oracle_rejects_inverted_or_omitted_boundaries(self):
        outer = compact(self.protocol(self.finalizer()[FINALIZER]))
        self.assert_outer(outer)  # positive control precedes every refusal
        for name, before, after in outer_corruptions():
            before, after = compact(before), compact(after)
            with self.subTest(boundary=name):
                self.assertEqual(outer.count(before), 1, name)
                with self.assertRaises(AssertionError):
                    self.assert_outer(outer.replace(before, after, 1))
        self.assert_outer(outer)


def outer_corruptions():
    # Finite source-format controls, not generated product SQL or native cases.
    return (
        ('operator_bypass', "current_user<>'console_buck_admin'", "current_user='console_buck_admin'"),
        ('target_negated', "starts_with(current_database(),'_sqlx_test_') IS NOT TRUE",
         "NOT(starts_with(current_database(),'_sqlx_test_')) IS NOT TRUE"),
        ('bootstrap_missing', "OR pg_catalog.current_setting('console.sqlx_test_bootstrap',true)\n      IS DISTINCT FROM 'buck-sqlx-superuser-v1'", ''),
        ('bounds_or', "AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='lock_timeout')",
         "OR (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='lock_timeout')"),
        ('ledger_share_missing', "AND mode='ShareLock' AND granted", "AND mode='ShareLock'"),
        ('ordinary_negated', "count(*)=83 AND count(DISTINCT c.oid)=83 AND count(DISTINCT n.oid)=1\n    AND bool_and",
         "count(*)=83 AND count(DISTINCT c.oid)=83 AND count(DISTINCT n.oid)=1\n    AND NOT bool_and"),
        ('scan_not_C', 'ORDER BY c.relname COLLATE "C"', 'ORDER BY c.relname'),
        ('wrong_relation_count', 'locked_relations:=0;', 'locked_relations:=1;'),
        ('null_variant_allowed', 'OR variant_name IS NULL THEN', 'OR variant_name IS NOT NULL THEN'),
        ('corrected_replay_executes', "IF observed_phase='native_group_process_navigation.head_revision_required' THEN",
         "IF observed_phase='native_group_process_navigation.finalized' THEN"),
        ('early_replay_return', 'expected_variant:=variant_name;', 'expected_variant:=variant_name; RETURN;'),
        ('constraints_missing', 'SET CONSTRAINTS ALL IMMEDIATE;', ''),
        ('postcheck_variant_inverted', 'OR variant_name IS DISTINCT FROM expected_variant THEN',
         'OR variant_name IS NOT DISTINCT FROM expected_variant THEN'),
        ('common_retained_locks_missing', '__exact231__\n' + RETAINED_LOCKS + '\nEND', '__exact231__\nEND'),
        ('extra_ledger_DML', 'END\n$native_group_process_navigation_custody$;',
         'DELETE FROM public._sqlx_migrations; END\n$native_group_process_navigation_custody$;'),
    )


if __name__ == '__main__':
    unittest.main()
