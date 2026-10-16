"""LC02 wrapper acceptance. No fabricated SQLx ledger or fixture ALTER OWNER.

--list discovers cases without Docker/build prerequisites. Normal invocation
builds the current console-app and uses its actual migrate run mode. Missing
production assets are prerequisites (exit 77), never deeper behavioral RED.
Only disposable, randomly named test containers are removed by this runner.
"""
from pathlib import Path
from decimal import Decimal
import ast
import hashlib
import gzip
import json
import os
import re
import secrets
import shutil
import subprocess
import sys
import tempfile
import time
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[1]
IMAGE = 'postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'
TABLES = ('accounts', 'account_security', 'account_security_events',
          'account_terms_acceptances', 'account_terms_head', 'account_terms_release_receipts')
OWNERS = {t: 'console_terms_owner' if t in TABLES[-2:] else 'console_account_owner' for t in TABLES}
CASES = ('valid_finalize', 'already_finalized', 'wrong_expected_operator',
         'ordinary_migration_login', 'missing_descriptor', 'wrong_database_descriptor', 'wrong_database_oid',
         'identically_migrated_wrong_database', 'wrong_cluster', 'missing_ledger',
         'release_checksum_mismatch', 'wrong_tls_ca', 'wrong_tls_hostname',
         'composed_default_installer_order', 'composed_observer_installer_order',
         'missing_credential_artifact_rolls_back', 'observer_failure_rolls_back_composed_custody',
         'missing_both_native_catalogs_with_exact_ledger',
         'native_extension_exact_final9_upgrade_preserves_all15_state',
         'native_extension_replay_preserves_all15_state',
         'native_extension_partial_helper_refuses_without_repair',
         'native_extension_foreign_helpers_refuse_without_repair',
         'native_extension_late_failure_rolls_back_then_retries',
         'operator_jit_default_child_settings', 'operator_jit_observer_child_settings')
ASSETS = ('postgres-finalize-account-custody.sh', 'postgres-finalize-account-custody.sql',
          'account-custody-migrations.sha384', 'postgres-finalize-account-credentials.sql',
          'postgres-install-durability-observer.sql', 'postgres-verify-account-native.sql')
CREDENTIAL_TABLES = ('auth_webauthn_credentials', 'auth_webauthn_ceremonies',
                    'auth_refresh_token_families', 'auth_refresh_tokens',
                    'auth_bootstrap_credentials', 'auth_webauthn_ceremony_bindings',
                    'auth_device_login_handoffs')

# Independent relation roster and state oracle; NULL/empty result cannot pass.
SNAPSHOT = """SELECT jsonb_agg(jsonb_build_object(
 'name',c.relname,'owner',pg_get_userbyid(c.relowner),'kind',c.relkind,
 'acl',c.relacl::text,'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attname,format_type(a.atttypid,a.atttypmod),
 a.attnotnull,a.attacl::text,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
 WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.convalidated,
 pg_get_constraintdef(k.oid)) ORDER BY k.conname) FROM pg_constraint k WHERE k.conrelid=c.oid),
 'triggers',(SELECT jsonb_agg(pg_get_triggerdef(t.oid) ORDER BY t.tgname)
 FROM pg_trigger t WHERE t.tgrelid=c.oid),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename)
 FROM pg_rewrite r WHERE r.ev_class=c.oid)) ORDER BY c.relname)
 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='public' AND c.relname IN (%s)""" % ','.join("'"+t+"'" for t in TABLES)
CREDENTIAL_SNAPSHOT = SNAPSHOT.replace(
    ','.join("'"+t+"'" for t in TABLES),
    ','.join("'"+t+"'" for t in CREDENTIAL_TABLES))


# Exact immutable predecessor; never infer it from the current database.
NATIVE_FINAL9_BASE = '9de5c7676c04e762c2c223cf34406b5b5c5449a2'
NATIVE_FINAL9_FIXTURE = 'ops/fixtures/account-custody-native-final9-9de5c767.json.gz'
NATIVE_FINAL9_FIXTURE_SHA256 = 'b65cf0f4dd7399c073acfbd335c0bdfa57fd62271b16bd4d6457f356f61413e6'
NATIVE_FINAL9_GENERATOR_SHA256 = 'e622978651edc0da867d39647e9d44d9fe0579fd57d1e8238ed236a9665fe710'
# Historical extension assets/classifiers remain frozen when current228 evolves.
NATIVE_EXTENDED227_BASE = '17fba595e6494da812c593fd1591dd50f770ab78'
NATIVE_EXTENDED227_FIXTURE = 'ops/fixtures/account-custody-native-extended227-17fba595.json.gz'
NATIVE_EXTENDED227_FIXTURE_SHA256 = '8124bc85ff844ff0dd6bdd3de28c13987dbf7de1ef245c3cdc05691b7fe3c6ac'
NATIVE_EXTENDED227_GENERATOR_SHA256 = '535ff78e66c9ed12a26823f47b8059f6af30b1402a1efaf889c9f8d479987027'
NATIVE_EXTENDED227_CLASSIFIERS = {
    'account_custody_state.sql': ('ops/fixtures/account_custody_state-extended227-17fba595.sql', '0eac017777279bd6afa0b04885c8fc634ac9a9d909bb50c5f4268db00a2281c8'),
    'account_credential_custody_state.sql': ('ops/fixtures/account_credential_custody_state-extended227-17fba595.sql', '129858239e492ddd8785ebd370b7660361eff57b9270ba04345eec85d9bbc290'),
}
NATIVE_STAGED_SHA256 = 'd148c9adeaa0f22286816d9bdb8fe8be3b899b9dfd1f5831d42fb1f41bc24200'
NATIVE_FINAL9_SHA256 = 'a453c9f30950a3f3ff9f545d6fb40bfc35ba0f79b97b94902be19cce2736d481'
# Independently approved actual declared-source capture: operator review
# ac59d8578185814d36c392986324e88e21b62279c48fd2df3b4d103c92578c47.
# This fixed expectation is never learned from the current target database.
NATIVE_EXTENSION_SHA256 = 'dc8e947c67171c1e10c268d4a1d54f18c1b5b39947d13b20a604d7e6f8de6766'
NATIVE_EXTENSION_HELPERS_SHA256 = 'ce4a703ec38deef76eae13efd197b748c981d2f1dfb81500228dcd38b22fdcd0'
NATIVE_TABLES = TABLES + CREDENTIAL_TABLES + ('company_actors', 'account_context_candidates')
NATIVE_HELPERS = {
    'account_login_consent_v1': ('p_account uuid', 'console_account_owner', 'console_auth_rt'),
    'auth_account_refresh_reuse_revoke_v1': (
        'p_account uuid, p_family uuid, p_token uuid, p_token_hash bytea, p_generation bigint, p_absolute_ttl interval',
        'console_credential_owner', 'console_account_owner'),
    'account_session_refresh_reuse_v1': (
        'p_account uuid, p_family uuid, p_token uuid, p_token_hash bytea, p_generation bigint, p_absolute_ttl interval',
        'console_account_owner', 'console_auth_rt'),
}


def decode_native_json(text):
    def reject_constant(value):
        raise ValueError('non-finite native JSON number')
    def object_pairs(pairs):
        result = {}
        for name, value in pairs:
            if name in result:
                raise ValueError('duplicate native JSON key')
            result[name] = value
        return result
    return json.loads(text, parse_float=Decimal, parse_constant=reject_constant, object_pairs_hook=object_pairs)


def exact_json_key(value):
    # Python considers True == 1 and rounds JSON floats. PostgreSQL catalog/row
    # preservation requires exact kinds and arbitrary precision, including scale.
    if value is None: return ('null',)
    if type(value) is bool: return ('boolean', value)
    if type(value) is int: return ('integer', str(value))
    if type(value) is str: return ('string', value)
    if type(value) is Decimal:
        assert value.is_finite(), 'non-finite observation'
        parts = value.as_tuple()
        return ('decimal', parts.sign, parts.digits, parts.exponent)
    if type(value) is list: return ('array', tuple(exact_json_key(item) for item in value))
    if type(value) is dict:
        assert all(type(name) is str for name in value), 'non-string observation key'
        return ('object', tuple((name,exact_json_key(value[name])) for name in sorted(value)))
    raise AssertionError('unsupported or lossy native observation value')


def exact_json_equal(left, right):
    return exact_json_key(left) == exact_json_key(right)


def source_literal(source, name):
    """Read frozen literal data, never import/execute a generator."""
    values = []
    for node in ast.parse(source).body:
        targets = node.targets if isinstance(node, ast.Assign) else [node.target] if isinstance(node, ast.AnnAssign) else []
        if any(isinstance(target, ast.Name) and target.id == name for target in targets):
            values.append(ast.literal_eval(node.value))
    assert len(values) == 1, 'missing or duplicate source literal: '+name
    return values[0]


def historical227_success(output):
    marker = 'HISTORICAL227_PRODUCER_PASS'
    prefix = 'test historical227::historical227_operator_owned_database_producer ... '
    lines = output.splitlines()
    witnesses = [line for line in lines if marker in line]
    leaves = [line for line in lines if line.startswith('test ') and not line.startswith('test result:')]
    runs = [line for line in lines if line.startswith('running ')]
    summaries = [line for line in lines if line.startswith('test result:')]
    if len(witnesses) != 1 or len(leaves) != 1:
        return False
    # libtest --nocapture prints the test prefix before the marker and completes
    # on the next line. Also accept a standalone marker and complete named line.
    if leaves[0] == prefix+marker:
        position = lines.index(leaves[0])
        complete = position+1 < len(lines) and lines[position+1] == 'ok'
    else:
        complete = leaves[0] == prefix+'ok' and witnesses == [marker]
    return (complete and runs == ['running 1 test'] and len(summaries) == 1
        and re.fullmatch(r'test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out;.*', summaries[0]) is not None)


def native_predecessor_bundle(root):
    """Read hash-bound tracked history; shallow clones need no old Git objects."""
    path = root / NATIVE_FINAL9_FIXTURE
    if path.is_symlink() or not path.is_file() or not path.resolve().is_relative_to(root.resolve()):
        raise AssertionError('nonregular native predecessor fixture')
    raw = path.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == NATIVE_FINAL9_FIXTURE_SHA256, 'native predecessor fixture digest mismatch'
    bundle = decode_native_json(gzip.decompress(raw).decode('utf-8'))
    assert set(bundle) == {'source_base', 'files', 'migration_sha256'}
    assert bundle['source_base'] == NATIVE_FINAL9_BASE
    files = bundle['files']
    assert set(files) == {'ops/'+name for name in ASSETS} | {'ops/generate-account-custody.py'}
    assert all(type(text) is str for text in files.values())
    generator = files['ops/generate-account-custody.py']
    assert hashlib.sha256(generator.encode('utf-8')).hexdigest() == NATIVE_FINAL9_GENERATOR_SHA256
    assert source_literal(generator, 'NATIVE_FINALIZED_SHA256') == NATIVE_FINAL9_SHA256
    assert source_literal(generator, 'NATIVE_STAGED_SHA256') == NATIVE_STAGED_SHA256
    roster = bundle['migration_sha256']
    assert type(roster) is dict and len(roster) == 227
    assert all(re.fullmatch(r'backend/crates/platform/db/migrations/[0-9]{4}_[a-z0-9_]+\.sql', name)
               and type(digest) is str and re.fullmatch('[0-9a-f]{64}', digest) for name, digest in roster.items())
    assert sorted(int(Path(name).name.split('_', 1)[0]) for name in roster) == list(range(1, 228))
    return bundle


def native_extended227_bundle(root):
    path = root / NATIVE_EXTENDED227_FIXTURE
    assert path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(root.resolve()), 'nonregular extended227 fixture'
    raw = path.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == NATIVE_EXTENDED227_FIXTURE_SHA256, 'extended227 fixture digest mismatch'
    bundle = decode_native_json(gzip.decompress(raw).decode('utf-8'))
    assert set(bundle) == {'source_base', 'files', 'migration_sha256'}
    assert bundle['source_base'] == NATIVE_EXTENDED227_BASE
    assert set(bundle['files']) == {'ops/'+name for name in ASSETS} | {'ops/generate-account-custody.py'}
    assert all(type(text) is str for text in bundle['files'].values())
    generator = bundle['files']['ops/generate-account-custody.py']
    assert hashlib.sha256(generator.encode()).hexdigest() == NATIVE_EXTENDED227_GENERATOR_SHA256
    assert bundle['migration_sha256'] == native_predecessor_bundle(root)['migration_sha256']
    return bundle


def assert_current_migration_ledger(root, ledger, frozen_files):
    """Certify current SQLx history from frozen regular sources, not its last row."""
    prefix = 'backend/crates/platform/db/migrations/'
    paths = sorted((root/prefix).glob('*.sql'))
    names = {str(path.relative_to(root)) for path in paths}
    frozen_names = {name for name in frozen_files if name.startswith(prefix) and name.endswith('.sql')}
    assert names and names == frozen_names, 'current migration source roster changed'
    expected = {}
    for path in paths:
        assert path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(root.resolve()), 'nonregular current migration source'
        match = re.fullmatch(r'([0-9]{4})_[a-z0-9_]+\.sql', path.name)
        assert match and int(match[1]) > 0, 'invalid current migration filename'
        version = int(match[1])
        assert version not in expected, 'duplicate current migration version'
        raw = path.read_bytes()
        assert hashlib.sha256(raw).hexdigest() == frozen_files[str(path.relative_to(root))], 'current migration bytes changed after freeze'
        expected[version] = [version, True, hashlib.sha384(raw).hexdigest()]
    assert exact_json_equal(ledger, [expected[version] for version in sorted(expected)]), 'real SQLx ledger differs from exact current migration sources'


def historical227_source_prefix(root, roster):
    # Suffix migrations may evolve; old227 names and bytes remain exact. Reject
    # duplicate numbered old files rather than silently selecting one by name.
    prefix = {}
    for path in (root/'backend/crates/platform/db/migrations').glob('*.sql'):
        assert path.is_file() and not path.is_symlink(), 'nonregular historical migration source'
        match = re.fullmatch(r'([0-9]{4})_[a-z0-9_]+\.sql', path.name)
        assert match and int(match[1]) > 0, 'invalid migration filename'
        if int(match[1]) <= 227:
            assert int(match[1]) not in prefix, 'duplicate historical migration version'
            prefix[int(match[1])] = str(path.relative_to(root))
    assert sorted(prefix) == list(range(1,228)) and set(prefix.values()) == set(roster), 'incomplete or renamed historical migration prefix'
    for name in prefix.values():
        assert hashlib.sha256((root/name).read_bytes()).hexdigest() == roster[name], 'historical migration bytes changed'


def native_extended227_classifiers(root):
    result = {}
    for name, (relative, digest) in NATIVE_EXTENDED227_CLASSIFIERS.items():
        path = root/relative
        assert path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(root.resolve()), 'nonregular extended227 classifier'
        raw = path.read_bytes()
        assert hashlib.sha256(raw).hexdigest() == digest, 'extended227 classifier digest mismatch'
        result[name] = raw.decode('utf-8')
    return result


def exact_named_rows(rows, expected, key='name'):
    assert isinstance(rows, list) and len(rows) == len(expected), 'missing/null/extra snapshot records'
    assert all(isinstance(row, dict) and isinstance(row.get(key), str) for row in rows), 'invalid snapshot row'
    names = [row[key] for row in rows]
    assert len(set(names)) == len(names) and set(names) == set(expected), 'duplicate or omitted snapshot record'
    return {row[key]: row for row in rows}


def validate_native_state(state):
    assert isinstance(state, dict) and set(state) == {'data', 'users', 'ledger', 'routines', 'roles', 'memberships', 'metadata'}, 'incomplete native observation'
    rows = exact_named_rows(state['data'], NATIVE_TABLES)
    for item in rows.values():
        assert set(item) == {'name', 'count', 'rows'} and type(item['count']) is int and item['count'] >= 0
        assert isinstance(item['rows'], list) and len(item['rows']) == item['count']
        assert all(isinstance(row, dict) for row in item['rows']), 'null/nonobject data row'
        encoded = [exact_json_key(row) for row in item['rows']]
        assert len(set(encoded)) == len(encoded), 'duplicate row cannot stand in for omitted data'
    for name in ('users', 'ledger', 'routines', 'roles'):
        assert isinstance(state[name], list) and state[name], 'missing positive observation: '+name
    assert isinstance(state['memberships'], list)
    assert all(type(row['version']) is int for row in state['ledger']), 'invalid ledger version type'
    versions = [row['version'] for row in state['ledger']]
    assert len(set(versions)) == len(versions) and all(row['success'] is True for row in state['ledger'])
    assert versions == list(range(1,228)), 'complete actual SQLx1..227 ledger required'
    assert isinstance(state['metadata'], dict) and set(state['metadata']) == {'snapshot', 'snapshot_text', 'sha256'}
    snapshot = state['metadata']['snapshot']
    raw = state['metadata']['snapshot_text']
    assert type(raw) is str and hashlib.sha256(raw.encode('utf-8')).hexdigest() == state['metadata']['sha256'], 'metadata digest is not bound to raw capture'
    assert exact_json_equal(decode_native_json(raw), snapshot), 'decoded metadata differs from raw capture'
    exact_named_rows(snapshot['tables'], NATIVE_TABLES)
    assert isinstance(snapshot['routines'], list) and snapshot['routines']
    keys = [routine_key(row['metadata']) for row in snapshot['routines']]
    assert len(set(keys)) == len(keys), 'duplicate routine observation'
    assert len({row['key'] for row in state['routines']}) == len(state['routines']), 'duplicate independent routine'
    assert re.fullmatch('[0-9a-f]{64}', state['metadata']['sha256'])
    return state


def assert_raw_native_migration(state):
    validate_native_state(state)
    tables = exact_named_rows(state['metadata']['snapshot']['tables'], NATIVE_TABLES)
    data = exact_named_rows(state['data'], NATIVE_TABLES)
    for name in TABLES:
        assert tables[name]['owner'] == 'console_app', 'native_raw_migration.owner_mismatch'
        assert data[name]['count'] == 0 and data[name]['rows'] == [], 'native_raw_migration.root_not_empty'
    assert state['metadata']['sha256'] not in (NATIVE_STAGED_SHA256, NATIVE_FINAL9_SHA256, NATIVE_EXTENSION_SHA256), 'native_raw_migration.already_in_custody'


def safe_failure_location(error):
    # Never serialize traceback locals, source lines, SQL or protected row values.
    location = 'external'
    trace = error.__traceback__
    while trace is not None:
        if trace.tb_frame.f_code.co_filename == __file__:
            location = trace.tb_frame.f_code.co_name+':'+str(trace.tb_lineno)
        trace = trace.tb_next
    return location


def routine_key(row):
    return row['schema']+'.'+row['name']+'('+row['identity_arguments']+')'


def assert_native_extension(before, after):
    validate_native_state(before); validate_native_state(after)
    assert before['metadata']['sha256'] == NATIVE_FINAL9_SHA256
    assert NATIVE_EXTENSION_SHA256 is not None and after['metadata']['sha256'] == NATIVE_EXTENSION_SHA256
    for name in ('data', 'users', 'ledger', 'roles', 'memberships'):
        assert exact_json_equal(before[name], after[name]), 'extension changed preserved '+name
    old, new = before['metadata']['snapshot'], after['metadata']['snapshot']
    assert set(old) == set(new)
    for name in old:
        if name != 'routines':
            assert exact_json_equal(old[name], new[name]), 'extension changed old metadata component: '+name
    old_routines = {routine_key(row['metadata']): row for row in old['routines']}
    new_routines = {routine_key(row['metadata']): row for row in new['routines']}
    added = {'public.'+name+'('+definition[0]+')' for name, definition in NATIVE_HELPERS.items()}
    assert not (set(old_routines) & added), 'predecessor already contains extension'
    assert set(new_routines) == set(old_routines) | added, 'unexpected/missing/overloaded routine'
    for key, row in old_routines.items():
        assert exact_json_equal(new_routines[key], row), 'old routine metadata changed'
    for name, (arguments, owner, caller) in NATIVE_HELPERS.items():
        row = new_routines['public.'+name+'('+arguments+')']
        record = row['metadata']
        assert row['extra_valid'] is True and record['owner'] == owner and record['security_definer'] is True
        assert exact_json_equal(sorted(record['acl']), sorted([[owner, owner, 'EXECUTE', False], [owner, caller, 'EXECUTE', False]])), 'new helper capability closure mismatch'
    old_public = {row['key']: row for row in before['routines']}
    new_public = {row['key']: row for row in after['routines']}
    assert set(new_public) == set(old_public) | added
    assert all(exact_json_equal(new_public[key], row) for key, row in old_public.items()), 'independent old pg_proc record/body changed'


def assert_composed_psql_trace(trace, assets, observer):
    """Actual shim argv only; this is not a SQL execution or transaction oracle."""
    assert trace.endswith('\0'), 'incomplete psql trace'
    words = trace.split('\0')[:-1]
    assert words and words[0] == 'BEGIN_PSQL' and words.count('BEGIN_PSQL') == 1, 'must use one psql process'
    argv = words[1:]
    assert argv.count('--single-transaction') == 1, 'one explicit transaction for all installers'
    assert '-X' in argv and '-w' in argv, 'preserve startup-file and password-prompt refusal'
    assert sum(argv[i:i+2] == ['--set', 'ON_ERROR_STOP=1'] for i in range(len(argv))) == 1
    flags = set()
    options = set()
    settings = {}
    files = []
    index = 0
    while index < len(argv):
        option = argv[index]
        index += 1
        if option in ('-X', '-w', '--quiet', '--single-transaction'):
            assert option not in flags, 'duplicate psql flag'
            flags.add(option)
            continue
        assert option in ('--host', '--port', '--username', '--dbname', '--set', '--file'), 'unexpected or attached psql option'
        assert index < len(argv) and argv[index] and not argv[index].startswith('-'), 'unbound psql option'
        value = argv[index]
        index += 1
        if option == '--set':
            name, separator, setting = value.partition('=')
            assert separator and setting and name in ('ON_ERROR_STOP', 'expected_operator', 'expected_database', 'expected_database_oid', 'expected_system_identifier'), 'unexpected psql setting'
            assert name not in settings, 'overridden psql setting'
            settings[name] = setting
        elif option == '--file':
            files.append(value)
        else:
            assert option not in options, 'overridden connection option'
            options.add(option)
    assert settings.get('ON_ERROR_STOP') == '1', 'error stop must remain enabled'
    expected = [assets+'/postgres-finalize-account-custody.sql',
                assets+'/postgres-finalize-account-credentials.sql']
    if observer:
        expected.append(assets+'/postgres-install-durability-observer.sql')
    expected.append(assets+'/postgres-verify-account-native.sql')
    assert len(files) == len(expected)+1, 'missing or extra installer file'
    assert re.fullmatch(r'/tmp/console-account-custody\.[A-Za-z0-9]+/preflight\.sql', files[0]), 'actual private preflight first'
    assert files[1:] == expected, 'root then credential then optional observer then native postcondition in the same transaction'


def assert_operator_child_environment(trace):
    """Only explicit nonsecret fields emitted by the actual psql child shim."""
    expected = ['BEGIN_OPERATOR_ENV', 'PGOPTIONS',
                '-c search_path=pg_catalog,pg_temp -c statement_timeout=60000 -c lock_timeout=5000 -c jit=off',
                'PGSSLMODE', 'verify-full', 'PGGSSENCMODE', 'disable',
                'PGCONNECT_TIMEOUT', '10', 'END_OPERATOR_ENV', '']
    assert trace.split('\0') == expected, 'actual psql child execution settings differ from fixed operator contract'


def complete_case_roster(results):
    names = [item['name'] for item in results]
    return (len(names) == len(CASES) and len(set(names)) == len(CASES)
            and set(names) == set(CASES) and all(item['status'] == 'PASS' for item in results))


def source_snapshot(root, paths):
    files = {}
    for name in sorted(set(paths)):
        path = root / name
        if path.is_symlink() or not path.is_file() or not path.resolve().is_relative_to(root.resolve()):
            raise AssertionError('nonregular source: '+name)
        files[name] = hashlib.sha256(path.read_bytes()).hexdigest()
    head = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
    return {'head': head, 'files': files}


def freeze_executable(source, destination):
    if source.is_symlink() or not source.is_file():
        raise AssertionError('nonregular migration executable')
    digest = hashlib.sha256(source.read_bytes()).hexdigest()
    shutil.copyfile(source, destination)
    destination.chmod(0o500)
    if hashlib.sha256(source.read_bytes()).hexdigest() != digest or hashlib.sha256(destination.read_bytes()).hexdigest() != digest:
        raise AssertionError('migration executable changed while freezing')
    return digest


def assert_executable_unchanged(path, expected):
    if path.is_symlink() or not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise AssertionError('frozen migration executable changed')


def cleanup_containers(containers, run):
    failures = []
    for container in reversed(containers):
        try:
            run(['docker', 'rm', '-f', container], required=False)
        except Exception:
            # A lost reply is not proof of absence. Still inspect this resource
            # and continue with every other pre-registered intended resource.
            pass
        try:
            code, text = run(['docker', 'container', 'inspect', container], required=False)
            absent = code != 0 and any(
                prefix.casefold() in ('error: no such container', 'error: no such object',
                    'error response from daemon: no such container',
                    'error response from daemon: no such object')
                and separator and name == container
                for prefix, separator, name in
                (line.strip().rpartition(': ') for line in text.splitlines()))
        except Exception:
            absent = False
        if not absent:
            failures.append(container)
    return failures


class Prerequisite(Exception):
    pass


class Suite:
    def __init__(self, output):
        self.output = output
        self.env = dict(os.environ)
        self.containers = []
        self.commands = []
        self.results = []
        self.temp = tempfile.TemporaryDirectory(prefix='console-account-wrapper-private-')
        self.private = Path(self.temp.name)
        self.passwords = {k: secrets.token_hex(24) for k in (
            'POSTGRES_ADMIN_PASSWORD', 'CONSOLE_APP_POSTGRES_PASSWORD',
            'CONSOLE_RT_POSTGRES_PASSWORD', 'CONSOLE_AUTH_POSTGRES_PASSWORD', 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD',
            'CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD', 'CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD')}
        self.sequence = 0
        self.provenance = {}
        self.frozen_sources = None

    def run(self, argv, *, env=None, stdin=None, required=True, timeout=240, private_output=False):
        self.sequence += 1
        log = self.output / f'{self.sequence:03d}.log'
        try:
            result = subprocess.run(argv, env=env or self.env, input=stdin,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout, cwd=ROOT)
        except subprocess.TimeoutExpired as error:
            data = error.stdout or b''
            log.write_bytes(b'[PRIVATE SQL OUTPUT OMITTED]\n' if private_output else self.redact(data))
            self.commands.append({'argv': argv, 'exit': 'TIMEOUT', 'log': log.name})
            raise AssertionError(f'command timed out: {argv[0]}') from error
        data = result.stdout
        leaked = any(value.encode() in data for value in self.passwords.values())
        log.write_bytes(b'[PRIVATE SQL OUTPUT OMITTED]\n' if private_output else self.redact(data))
        self.commands.append({'argv': argv, 'exit': result.returncode, 'log': log.name})
        if leaked:
            raise AssertionError('command emitted credential bytes; retained output redacted')
        if required and result.returncode:
            raise AssertionError(f'command failed ({result.returncode}); see {log.name}')
        return result.returncode, data.decode('utf-8', errors='strict')

    def redact(self, data):
        for secret in self.passwords.values():
            data = data.replace(secret.encode(), b'[CREDENTIAL_REDACTED]')
        return data

    def private_file(self, name, text):
        path = self.private / name
        path.write_text(text)
        path.chmod(0o600)
        return path

    def assert_sources_unchanged(self):
        if self.frozen_sources is not None:
            current = source_snapshot(ROOT, self.frozen_sources['files'])
            if current != self.frozen_sources:
                raise AssertionError('source custody changed after pre-build freeze')

    def copy(self, container, source, target):
        source = Path(source)
        if self.frozen_sources is not None and source.is_relative_to(ROOT):
            name = str(source.relative_to(ROOT))
            expected = self.frozen_sources['files'].get(name)
            if expected is None or source.is_symlink() or hashlib.sha256(source.read_bytes()).hexdigest() != expected:
                raise AssertionError('copy source differs from pre-build freeze: '+name)
        self.run(['docker', 'cp', str(source), f'{container}:{target}'])

    def psql(self, container, database, sql, *, role='wrapper_admin', password=None, required=True, private_output=False, verifying_tls=False):
        password = password or self.passwords['POSTGRES_ADMIN_PASSWORD']
        assert type(verifying_tls) is bool, 'TLS observation profile must be explicit'
        tls_options = ('PGSSLMODE=verify-full\nPGSSLROOTCERT=/wrapper-tls/ca.crt\nPGGSSENCMODE=disable\n'
                       if verifying_tls else '')
        envfile = self.private_file('psql.env', 'PGPASSWORD='+password+'\n'+tls_options)
        return self.run(['docker', 'exec', '-i', '--env-file', str(envfile), container,
                         'psql', '-X', '-w', '-h', 'localhost', '-U', role, '-d', database,
                         '-v', 'ON_ERROR_STOP=1', '-At', '-F', '|', *(['-q'] if private_output else []), '--file', '-'],
                        stdin=sql.encode('utf-8'), required=required, **({'private_output': True} if private_output else {}))

    def snapshot(self, container, database):
        _, text = self.psql(container, database, SNAPSHOT)
        rows = json.loads(text)
        assert isinstance(rows, list) and len(rows) == 6, 'missing/empty catalog snapshot'
        assert {r['name'] for r in rows} == set(TABLES), 'incomplete snapshot roster'
        counts = []
        for table in TABLES:
            _, count = self.psql(container, database, f'SELECT count(*) FROM public.{table}')
            counts.append(int(count.strip()))
        assert counts[1:] == [0]*5, 'finalization populated Account security or Terms rows'
        # Root finalization backfills only accounts(id,created_at) from users.
        # Preserve every Account/user field in refusal and replay comparisons.
        _, raw = self.psql(container, database, """SELECT jsonb_build_object(
            'accounts',(SELECT COALESCE(jsonb_agg(to_jsonb(a) ORDER BY a.id),'[]'::jsonb)
                        FROM public.accounts a),
            'users',(SELECT COALESCE(jsonb_agg(to_jsonb(u) ORDER BY u.id),'[]'::jsonb)
                     FROM public.users u))""")
        data = json.loads(raw)
        assert isinstance(data['accounts'], list) and len(data['accounts']) == counts[0]
        assert isinstance(data['users'], list), 'missing user-row snapshot'
        # Include actual ledger data in equality without ever writing its rows.
        _, exists = self.psql(container, database, "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
        ledger = None
        if exists.strip() == 't':
            _, raw = self.psql(container, database,
                "SELECT jsonb_agg(jsonb_build_array(version,success,encode(checksum,'hex')) ORDER BY version) FROM public._sqlx_migrations")
            ledger = json.loads(raw)
            assert ledger, 'empty ledger cannot be a migrated positive'
        return {'relations': rows, 'counts': counts, 'ledger': ledger,
                'account_rows': data['accounts'], 'user_rows': data['users']}

    def composed_snapshot(self, container, database):
        root = self.snapshot(container, database)
        _, text = self.psql(container, database, CREDENTIAL_SNAPSHOT)
        credentials = json.loads(text)
        assert isinstance(credentials, list) and len(credentials) == 7
        assert {row['name'] for row in credentials} == set(CREDENTIAL_TABLES)
        counts = []
        for table in CREDENTIAL_TABLES:
            _, text = self.psql(container, database, f'SELECT count(*) FROM public.{table}')
            counts.append(int(text.strip()))
        # Unchanged migrator produces these databases; no credential row seed.
        # Compare every observed count rather than assuming a fixture value.
        _, text = self.psql(container, database, """SELECT jsonb_agg(jsonb_build_array(
            n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
            pg_get_userbyid(p.proowner),p.proacl::text,pg_get_functiondef(p.oid))
            ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid))
            FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
            WHERE n.nspname='public' AND p.prokind IN ('f','p')""")
        return {'root': root, 'credentials': credentials, 'counts': counts,
                'public_routines': json.loads(text)}

    def native_observation(self, container, database, snapshot_query):
        # Full rows remain only in private process memory. Neither secrets nor
        # credential/hash/consent values are emitted to ordinary evidence logs.
        tables = []
        for table in NATIVE_TABLES:
            tables.append("jsonb_build_object('name','"+table+"','count',(SELECT count(*) FROM public."+table+"),'rows',"
                "(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text COLLATE \"C\"),'[]'::jsonb) FROM public."+table+" t))")
        query = """SELECT jsonb_build_object('data',jsonb_build_array(%s),
          'users',(SELECT jsonb_agg(to_jsonb(u) ORDER BY u.id) FROM public.users u),
          'ledger',(SELECT jsonb_agg(to_jsonb(m) ORDER BY m.version) FROM public._sqlx_migrations m),
          'roles',(SELECT jsonb_agg(to_jsonb(r) ORDER BY r.rolname) FROM pg_roles r),
          'memberships',(SELECT COALESCE(jsonb_agg(to_jsonb(m) ORDER BY to_jsonb(m)::text COLLATE "C"),'[]'::jsonb) FROM pg_auth_members m),
          'routines',(SELECT jsonb_agg(jsonb_build_object('key',n.nspname||'.'||p.proname||'('||pg_get_function_identity_arguments(p.oid)||')',
              'catalog',to_jsonb(p),'definition',pg_get_functiondef(p.oid)) ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid))
              FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.prokind IN ('f','p')),
          'metadata',(SELECT jsonb_build_object('snapshot',snapshot,'snapshot_text',snapshot::text,'sha256',snapshot_sha256) FROM (%s) capture))""" % (','.join(tables), snapshot_query)
        _, text = self.psql(container, database, 'BEGIN READ ONLY;\nSET LOCAL search_path=pg_catalog,pg_temp;\n'+query+';\nCOMMIT;', private_output=True)
        return validate_native_state(decode_native_json(text))

    def native_classifier(self, container, database, root_status, credential_status, *, historical227=False):
        for name, expected in (('account_custody_state.sql', root_status),
                               ('account_credential_custody_state.sql', credential_status)):
            source = self.historical227_classifiers[name] if historical227 else (ROOT/'backend/app/src'/name).read_text()
            _, text = self.psql(container, database, 'SET search_path=pg_catalog,pg_temp;\n'+source)
            assert text.strip().splitlines() in ([expected], ['SET', expected]), 'wrong native serving classifier'

    def native_extension_cases(self, container, binary):
        if not isinstance(NATIVE_EXTENSION_SHA256, str) or not re.fullmatch('[0-9a-f]{64}', NATIVE_EXTENSION_SHA256):
            raise Prerequisite('independently reviewed fixed extension digest not pinned in test source')
        extended = native_extended227_bundle(ROOT)
        self.historical227_classifiers = native_extended227_classifiers(ROOT)
        generator = extended['files']['ops/generate-account-custody.py']
        assert source_literal(generator, 'NATIVE_FINALIZED_SHA256') == NATIVE_EXTENSION_SHA256
        assert source_literal(generator, 'NATIVE_FINALIZED9_SHA256') == NATIVE_FINAL9_SHA256
        assert source_literal(generator, 'NATIVE_STAGED_SHA256') == NATIVE_STAGED_SHA256
        query = source_literal(generator, 'NATIVE_SNAPSHOT_QUERY')
        extension = source_literal(generator, 'NATIVE_EXTENSION_INSTALL')
        assert hashlib.sha256(extension.encode()).hexdigest() == NATIVE_EXTENSION_HELPERS_SHA256, 'extension source changed after reviewed test binding'
        historical = native_predecessor_bundle(ROOT)
        old_generator = historical['files']['ops/generate-account-custody.py']
        old_query = source_literal(old_generator, 'NATIVE_SNAPSHOT_QUERY')
        old_credentials = historical['files']['ops/postgres-finalize-account-credentials.sql']
        assert old_credentials.count("IF native_state IS DISTINCT FROM 'account_native.staged' THEN\n        RAISE EXCEPTION 'account_native.staging_profile_mismatch';") == 1, 'missing immutable predecessor intermediate-stage guard'
        assert ("WHEN snapshot_sha256='"+NATIVE_STAGED_SHA256+"' THEN 'account_native.staged'") in old_credentials, 'predecessor stage guard lost independently pinned profile'
        # Genuine run_to227 owns the ledger; suffix228 is never executed here.
        # Preserve every historical name/hash while allowing a later suffix.
        historical227_source_prefix(ROOT, historical['migration_sha256'])
        predecessor = self.private/'native-final9-assets'
        predecessor.mkdir()
        for name in ASSETS:
            (predecessor/name).write_bytes(historical['files']['ops/'+name].encode('utf-8'))
        extension_assets = self.private/'native-extended227-assets'
        extension_assets.mkdir()
        for name in ASSETS:
            (extension_assets/name).write_bytes(extended['files']['ops/'+name].encode('utf-8'))
        assert (predecessor/'account-custody-migrations.sha384').read_bytes() == (extension_assets/'account-custody-migrations.sha384').read_bytes()
        self.copy(container, predecessor, '/native-final9-assets')
        self.copy(container, extension_assets, '/native-extended227-assets')
        self.provenance['native_extension'] = {
            'predecessor_base': NATIVE_FINAL9_BASE, 'predecessor_generator_sha256': NATIVE_FINAL9_GENERATOR_SHA256,
            'predecessor_fixture_sha256': NATIVE_FINAL9_FIXTURE_SHA256,
            'intermediate_stage_sha256': NATIVE_STAGED_SHA256,
            'intermediate_stage_evidence': 'mandatory exact predecessor operator guard after root/Auth7; new/old intermediate serializer equality independently captured in approved capture receipt',
            'predecessor_assets': {name:hashlib.sha256((predecessor/name).read_bytes()).hexdigest() for name in ASSETS},
            'extended_sha256': NATIVE_EXTENSION_SHA256, 'helper_source_sha256': NATIVE_EXTENSION_HELPERS_SHA256,
            'extended_source_base': NATIVE_EXTENDED227_BASE, 'extended_fixture_sha256': NATIVE_EXTENDED227_FIXTURE_SHA256,
            'extended_assets': {name:hashlib.sha256((extension_assets/name).read_bytes()).hexdigest() for name in ASSETS},
            'historical_classifiers': {name:digest for name,(_,digest) in NATIVE_EXTENDED227_CLASSIFIERS.items()},
            'coverage': 'complete observed15-table rows; real migrator legacy rows and restricted-owner pending Account; active credential/consent/history NOT certified'}
        def final9(database):
            self.database(container, database, binary, historical227=True)
            raw = self.native_observation(container, database, query)
            assert_raw_native_migration(raw)
            assert exact_json_equal(self.native_observation(container, database, old_query)['metadata'], raw['metadata']), 'new serializer changed raw migration capture'
            # The fixed native-staged pin is AFTER root+Auth7 custody, inside the
            # unchanged predecessor package. Its mandatory staged-profile guard
            # executes before native installation. Raw SQLx migrations are not it.
            expected = self.descriptor(container, database)
            code, text = self.traced_wrapper(container, database, expected, assets='/native-final9-assets')
            assert code == 0 and 'account_custody.native_finalized' in text, 'actual predecessor package did not finalize'
            before = self.native_observation(container, database, query)
            assert before['metadata']['sha256'] == NATIVE_FINAL9_SHA256, 'native_final9.digest_mismatch expected='+NATIVE_FINAL9_SHA256+' actual='+before['metadata']['sha256']
            assert exact_json_equal(self.native_observation(container, database, old_query)['metadata'], before['metadata']), 'new serializer changed final9 capture'
            assert next(row for row in before['data'] if row['name'] == 'accounts')['count'] > 0, 'missing actual legacy backfill positive'
            self.native_classifier(container, database, 'account_custody.native_upgrade_required', 'account_credentials.native_upgrade_required', historical227=True)
            self.native_classifier(container, database, 'account_custody.native_upgrade_required', 'account_credentials.native_upgrade_required')
            # Populate a genuine pending Account through its existing owner as
            # actual Auth LOGIN, not raw table writes or SET ROLE. Full active
            # enrollment/credential/consent history remains a separate fixture.
            _, identity = self.psql(container, database, 'SELECT session_user,current_user',
                role='console_auth_rt', password=self.passwords['CONSOLE_AUTH_POSTGRES_PASSWORD'])
            assert identity.strip() == 'console_auth_rt|console_auth_rt'
            _, text = self.psql(container, database, 'SELECT to_jsonb(created) FROM public.account_registration_begin_v1() created',
                role='console_auth_rt', password=self.passwords['CONSOLE_AUTH_POSTGRES_PASSWORD'], private_output=True)
            created = decode_native_json(text)
            assert set(created) == {'account_id','security_generation','revision','context_generation','created_at'}
            assert all(type(created[name]) is int and created[name] == 1 for name in ('security_generation','revision','context_generation'))
            populated = self.native_observation(container, database, query)
            old_rows = exact_named_rows(before['data'], NATIVE_TABLES)
            new_rows = exact_named_rows(populated['data'], NATIVE_TABLES)
            for name in NATIVE_TABLES:
                if name in ('accounts','account_security'):
                    assert new_rows[name]['count'] == old_rows[name]['count']+1
                    key = 'id' if name == 'accounts' else 'account_id'
                    added = [row for row in new_rows[name]['rows'] if row[key] == created['account_id']]
                    assert len(added) == 1
                    assert exact_json_equal([row for row in new_rows[name]['rows'] if row[key] != created['account_id']], old_rows[name]['rows'])
                    if name == 'account_security':
                        assert added[0]['security_state'] == 'PENDING_ENROLLMENT'
                else:
                    assert exact_json_equal(new_rows[name], old_rows[name])
            for name in ('users','ledger','routines','roles','memberships','metadata'):
                assert exact_json_equal(before[name], populated[name]), 'ordinary pending creation changed unrelated state'
            return expected, populated
        database = 'wrapper_native_upgrade'
        def upgrade():
            expected, before = final9(database)
            code, text = self.traced_wrapper(container, database, expected, assets='/native-extended227-assets')
            assert code == 0 and 'account_custody.native_finalized' in text, 'exact final9 did not upgrade'
            after = self.native_observation(container, database, query)
            assert_native_extension(before, after)
            self.native_classifier(container, database, 'account_custody.native_finalized', 'account_credentials.native_finalized', historical227=True)
            self.native_classifier(container, database, 'account_custody.native_upgrade_required', 'account_credentials.native_upgrade_required')
            # Independently preserve the other genuine227 path: staged->extended.
            # The main valid_finalize case still exercises today's current assets.
            staged_installed_database = 'wrapper_native_staged227'
            self.database(container, staged_installed_database, binary, historical227=True)
            code, text = self.traced_wrapper(container, staged_installed_database,
                self.descriptor(container, staged_installed_database), assets='/native-extended227-assets')
            assert code == 0 and 'account_custody.native_finalized' in text, 'historical staged path did not finalize'
            fresh = self.native_observation(container, staged_installed_database, query)
            assert exact_json_equal(fresh['metadata'], after['metadata']), 'staged and predecessor paths disagree'
        self.case('native_extension_exact_final9_upgrade_preserves_all15_state', upgrade)
        def replay():
            before = self.native_observation(container, database, query)
            code, text = self.traced_wrapper(container, database, self.descriptor(container, database), assets='/native-extended227-assets')
            assert code == 0 and 'account_custody.native_finalized' in text
            assert exact_json_equal(self.native_observation(container, database, query), before), 'extension replay changed data or metadata'
        self.case('native_extension_replay_preserves_all15_state', replay)
        def refusal(database, fault, expected_name, expected_arguments, expected_owner):
            expected, original = final9(database)
            self.psql(container, database, fault)
            before = self.native_observation(container, database, query)
            matches = [row['metadata'] for row in before['metadata']['snapshot']['routines'] if row['metadata']['name'] == expected_name]
            assert len(matches) == 1 and matches[0]['identity_arguments'] == expected_arguments and matches[0]['owner'] == expected_owner, 'fault did not reach exact helper metadata'
            assert not exact_json_equal(before, original) and before['metadata']['sha256'] not in (NATIVE_FINAL9_SHA256, NATIVE_EXTENSION_SHA256)
            self.native_classifier(container, database, 'account_native.profile_mismatch', 'account_native.profile_mismatch', historical227=True)
            self.native_classifier(container, database, 'account_native.profile_mismatch', 'account_native.profile_mismatch')
            code, text = self.traced_wrapper(container, database, expected, assets='/native-extended227-assets')
            assert code != 0 and 'account_native.profile_mismatch' in text and 'account_custody.native_finalized' not in text
            assert exact_json_equal(self.native_observation(container, database, query), before), 'operator repaired foreign/partial metadata or changed rows'
        def partial():
            # Install the exact first reviewed production helper/ACL block only.
            marker = '\n-- Private root-owned proposal;'
            assert extension.count(marker) == 1
            first = extension.split(marker)[0]
            assert first.count('CREATE FUNCTION ') == 1 and 'GRANT EXECUTE ON FUNCTION public.account_login_consent_v1(uuid)' in first
            refusal('wrapper_native_partial', first, 'account_login_consent_v1', 'p_account uuid', 'console_account_owner')
        self.case('native_extension_partial_helper_refuses_without_repair', partial)
        def foreign():
            for suffix, argument in (('same', 'uuid'), ('overload', 'text')):
                # Fault under actual admin ownership; never ALTER OWNER to
                # fabricate a valid positive custody fixture.
                fault = 'CREATE FUNCTION public.account_login_consent_v1(p_account '+argument+") RETURNS boolean LANGUAGE sql AS 'SELECT false';"
                refusal('wrapper_native_foreign_'+suffix, fault, 'account_login_consent_v1', 'p_account '+argument, 'wrapper_admin')
        self.case('native_extension_foreign_helpers_refuse_without_repair', foreign)
        def rollback():
            database = 'wrapper_native_rollback'
            expected, before = final9(database)
            variant = self.private/'native-late-failure'
            variant.mkdir()
            for name in ASSETS:
                shutil.copyfile(extension_assets/name, variant/name)
            postcondition = variant/'postgres-verify-account-native.sql'
            original = postcondition.read_text()
            postcondition.write_text(original+"\nDO $native_extension_fault$ BEGIN\n"
                "IF (SELECT snapshot_sha256 FROM ("+query+") capture) IS DISTINCT FROM '"+NATIVE_EXTENSION_SHA256+"' THEN RAISE EXCEPTION 'test.extension_stage_not_reached'; END IF;\n"
                "RAISE NOTICE 'test.extension_stage_reached';\nRAISE EXCEPTION 'test.extension_late_failure';\nEND $native_extension_fault$;\n")
            self.copy(container, variant, '/native-late-failure')
            code, text = self.traced_wrapper(container, database, expected, assets='/native-late-failure')
            assert code != 0 and 'test.extension_stage_reached' in text and 'test.extension_late_failure' in text, 'did not fail after real extension/postcondition'
            assert 'account_custody.native_finalized' not in text
            assert exact_json_equal(self.native_observation(container, database, query), before), 'late failure committed helpers or changed preserved state'
            self.native_classifier(container, database, 'account_custody.native_upgrade_required', 'account_credentials.native_upgrade_required', historical227=True)
            self.native_classifier(container, database, 'account_custody.native_upgrade_required', 'account_credentials.native_upgrade_required')
            postcondition.write_text(original)
            self.copy(container, postcondition, '/native-late-failure/postgres-verify-account-native.sql')
            code, text = self.traced_wrapper(container, database, expected, assets='/native-late-failure')
            assert code == 0 and 'account_custody.native_finalized' in text, 'same packaged retry failed after injection removal'
            assert_native_extension(before, self.native_observation(container, database, query))
        self.case('native_extension_late_failure_rolls_back_then_retries', rollback)

    def traced_wrapper(self, container, database, expected, *, observer=False, assets='/wrapper-assets', execution_settings=False):
        # Test-only transport observation. The shim forwards every original
        # argument to the actual image psql; it changes no SQL, identity or TLS.
        number = self.sequence
        directory = f'/wrapper-trace-{number}'
        self.run(['docker', 'exec', container, 'mkdir', '-p', directory])
        _, selected = self.run(['docker', 'exec', container, 'sh', '-c', 'command -v psql'])
        actual_psql = selected.strip()
        assert re.fullmatch(r'/[A-Za-z0-9_./-]+/psql', actual_psql)
        trace_path = directory+'/argv'
        environment_path = directory+'/environment'
        environment_trace = (
            "printf '%s\\0' BEGIN_OPERATOR_ENV PGOPTIONS \"${PGOPTIONS-}\" "
            "PGSSLMODE \"${PGSSLMODE-}\" PGGSSENCMODE \"${PGGSSENCMODE-}\" "
            "PGCONNECT_TIMEOUT \"${PGCONNECT_TIMEOUT-}\" END_OPERATOR_ENV > "+environment_path+'\n'
        ) if execution_settings else ''
        shim = self.private_file(f'psql-shim-{number}',
            '#!/bin/sh\n{ printf \'BEGIN_PSQL\\0\'; printf \'%s\\0\' "$@"; } >> '+trace_path+'\n'
            +environment_trace+'exec '+actual_psql+' "$@"\n')
        self.copy(container, shim, directory+'/psql')
        self.run(['docker', 'exec', container, 'chmod', '500', directory+'/psql'])
        hostile_ambient = {'PGOPTIONS':'-c jit=on -c statement_timeout=0 -c lock_timeout=0',
                           'PGSSLMODE':'disable', 'PGGSSENCMODE':'prefer'} if execution_settings else {}
        result = self.wrapper(container, database, expected, observer=observer, assets=assets,
            PATH=directory+':/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
            **hostile_ambient)
        _, trace = self.run(['docker', 'exec', container, 'cat', trace_path])
        assert_composed_psql_trace(trace, assets, observer)
        if execution_settings:
            code, text = result
            assert code != 0 and 'account_custody.operator_identity_mismatch' in text, 'actual production preflight identity refusal required before configuration oracle'
            assert 'account_custody.native_finalized' not in text
            _, environment = self.run(['docker', 'exec', container, 'cat', environment_path])
            assert_operator_child_environment(environment)
        return result

    def operator_jit_configuration_cases(self, container, *, collect_failures=False):
        # Real TLS psql executes the original production preflight. Deliberately
        # wrong expected identity stops before any schema/installer dependency.
        assert type(collect_failures) is bool, 'case collection must be explicit'
        failures = False
        descriptor = self.descriptor(container, 'postgres')
        descriptor['ACCOUNT_CUSTODY_EXPECTED_OPERATOR'] = 'not_the_actual_operator'
        for observer, name in ((False, 'operator_jit_default_child_settings'),
                               (True, 'operator_jit_observer_child_settings')):
            def observe(observer=observer):
                sql = "SELECT current_setting('jit'),session_user,current_user,current_database(),(SELECT ssl FROM pg_stat_ssl WHERE pid=pg_backend_pid())"
                _, before = self.psql(container, 'postgres', sql, verifying_tls=True)
                assert before.strip() == 'on|wrapper_admin|wrapper_admin|postgres|t', 'real TLS administrator and default JIT-on positive control required'
                code, text = self.traced_wrapper(container, 'postgres', descriptor,
                    observer=observer, execution_settings=True)
                assert code != 0 and 'account_custody.operator_identity_mismatch' in text, 'actual production preflight identity refusal required'
                assert 'account_custody.native_finalized' not in text
                _, after = self.psql(container, 'postgres', sql, verifying_tls=True)
                assert after == before, 'one-shot options escaped into later sessions'
            try:
                self.case(name, observe)
            except Exception:
                if not collect_failures:
                    raise
                failures = True
        if failures:
            raise AssertionError('operator configuration cases failed; each exact failure is retained')

    def certificate(self):
        def openssl(*args):
            self.run(['openssl', *map(str, args)])
        openssl('req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '2',
                '-subj', '/CN=Wrapper Test CA', '-keyout', self.private/'ca.key', '-out', self.private/'ca.crt')
        openssl('req', '-newkey', 'rsa:2048', '-nodes', '-subj', '/CN=localhost',
                '-keyout', self.private/'server.key', '-out', self.private/'server.csr')
        ext = self.private_file('server.ext', 'subjectAltName=DNS:localhost\nextendedKeyUsage=serverAuth\n')
        openssl('x509', '-req', '-in', self.private/'server.csr', '-CA', self.private/'ca.crt',
                '-CAkey', self.private/'ca.key', '-CAcreateserial', '-days', '2',
                '-extfile', ext, '-out', self.private/'server.crt')
        openssl('req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '2',
                '-subj', '/CN=Wrong Test CA', '-keyout', self.private/'wrong-ca.key', '-out', self.private/'wrong-ca.crt')
        for name in ('ca.key', 'server.key', 'wrong-ca.key'):
            (self.private/name).chmod(0o600)

    def cluster(self):
        container = 'console-wrapper-'+secrets.token_hex(6)
        env = dict(self.passwords, POSTGRES_USER='wrapper_admin', POSTGRES_ADMIN_USER='wrapper_admin',
                   POSTGRES_PASSWORD=self.passwords['POSTGRES_ADMIN_PASSWORD'], POSTGRES_DB='postgres',
                   POSTGRES_HOST='localhost')
        file = self.private_file('cluster.env', ''.join(k+'='+v+'\n' for k,v in env.items()))
        # Register intent before Docker can create a resource and lose its ACK.
        self.containers.append(container)
        self.run(['docker','run','-d','--rm','--name',container,'--env-file',str(file),
                  '-p','127.0.0.1::5432',IMAGE])
        for _ in range(60):
            code, _ = self.run(['docker','exec',container,'sh','-c',
                'test "$(cat /proc/1/comm)" = postgres && pg_isready -U wrapper_admin -d postgres'], required=False)
            if code == 0:
                break
            time.sleep(1)
        else:
            raise Prerequisite('PostgreSQL durable process not ready')
        self.run(['docker','exec',container,'mkdir','-p','/wrapper-tls','/wrapper-assets'])
        for name in ('server.key','server.crt','ca.crt','wrong-ca.crt'):
            self.copy(container,self.private/name,'/wrapper-tls/'+name)
        self.run(['docker','exec',container,'chown','postgres:postgres','/wrapper-tls/server.key'])
        self.run(['docker','exec',container,'chmod','600','/wrapper-tls/server.key'])
        self.psql(container,'postgres',"ALTER SYSTEM SET ssl='on'")
        self.psql(container,'postgres',"ALTER SYSTEM SET ssl_cert_file='/wrapper-tls/server.crt'")
        self.psql(container,'postgres',"ALTER SYSTEM SET ssl_key_file='/wrapper-tls/server.key'")
        self.psql(container,'postgres','SELECT pg_reload_conf()')
        for name in ASSETS:
            self.copy(container,ROOT/'ops'/name,'/wrapper-assets/'+name)
        self.copy(container,ROOT/'ops/postgres-reconcile-topology.sh','/topology.sh')
        password = self.private_file('operator.password',self.passwords['POSTGRES_ADMIN_PASSWORD']+'\n')
        self.copy(container,password,'/operator.password')
        self.run(['docker','exec',container,'chmod','600','/operator.password'])
        return container

    def build_historical227_producer(self, build_env, owned_target):
        # Existing auth_rest integration executable; no alternate migration app.
        _, output = self.run(['cargo','test','--locked','--manifest-path',str(ROOT/'backend/Cargo.toml'),
            '-p','console-app','--test','auth_rest','--features','test-historical227-producer',
            '--no-run','--message-format=json'], env=build_env, timeout=900)
        artifacts = []
        for line in output.splitlines():
            try:
                value = json.loads(line)
            except ValueError:
                continue
            if (isinstance(value, dict) and value.get('reason') == 'compiler-artifact'
                    and value.get('target', {}).get('name') == 'auth_rest'
                    and value.get('target', {}).get('kind') == ['test']
                    and value.get('profile', {}).get('test') is True and value.get('executable')):
                artifacts.append(Path(value['executable']))
        assert len(artifacts) == 1 and artifacts[0].resolve().is_relative_to(owned_target.resolve()), 'historical227 exact owned test executable required'
        self.assert_sources_unchanged()
        self.historical227_binary = self.private/'auth-rest-historical227-frozen'
        self.provenance['historical227_binary_sha256'] = freeze_executable(artifacts[0], self.historical227_binary)

    def historical227_producer(self, container, database, url):
        # Parent owns the exact disposable container/port/TLS certificate and
        # independently reads cluster identity; console_app gains no monitor role.
        expected = self.descriptor(container, database)
        env = {k:v for k,v in self.env.items() if not k.startswith(('PG','CONSOLE_','DATABASE_'))}
        env.update(CONSOLE_HISTORICAL227_DATABASE_URL=url,
            CONSOLE_HISTORICAL227_EXPECTED_DATABASE=expected['ACCOUNT_CUSTODY_EXPECTED_DATABASE'],
            CONSOLE_HISTORICAL227_EXPECTED_DATABASE_OID=expected['ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID'])
        binary = self.historical227_binary
        digest = self.provenance['historical227_binary_sha256']
        assert_executable_unchanged(binary, digest)
        _, output = self.run([str(binary), '--exact',
            'historical227::historical227_operator_owned_database_producer',
            '--test-threads=1', '--nocapture', '--color=never'], env=env, timeout=300)
        assert_executable_unchanged(binary, digest)
        assert historical227_success(output), 'historical227 exact one-test completion required'
        assert self.descriptor(container, database) == expected, 'historical227 owned target changed during production'

    def database(self, container, name, binary, *, raw=False, historical227=False):
        assert not (raw and historical227), 'raw SQL is not a historical SQLx producer'
        self.psql(container,'postgres',f'CREATE DATABASE {name}')
        self.run(['docker','exec','-e','POSTGRES_DB='+name,container,'bash','/topology.sh'])
        if raw:
            # Same real source scripts, deliberately NO synthetic SQLx ledger.
            for source in sorted((ROOT/'backend/crates/platform/db/migrations').glob('*.sql')):
                self.copy(container,source,'/raw-migration.sql')
                envfile = self.private_file('raw.env','PGPASSWORD='+self.passwords['CONSOLE_APP_POSTGRES_PASSWORD']+'\n')
                self.run(['docker','exec','--env-file',str(envfile),container,'psql','-X','-w',
                    '-h','localhost','-U','console_app','-d',name,'-v','ON_ERROR_STOP=1','-f','/raw-migration.sql'])
        else:
            _, bindings = self.run(['docker','port',container,'5432/tcp'])
            port = bindings.strip().rsplit(':',1)[1]
            url = ('postgresql://console_app:'+self.passwords['CONSOLE_APP_POSTGRES_PASSWORD']+
                   '@localhost:'+port+'/'+name+'?sslmode=verify-full&sslrootcert='+quote(str(self.private/'ca.crt'),safe=''))
            env = {k:v for k,v in self.env.items() if not k.startswith(('PG','CONSOLE_','DATABASE_'))}
            env.update(CONSOLE_APP_ROLE='migrate', DATABASE_URL=url)
            if historical227:
                self.historical227_producer(container, name, url)
            else:
                assert_executable_unchanged(binary,self.provenance['binary_sha256'])
                self.run([str(binary)],env=env,timeout=300)
                assert_executable_unchanged(binary,self.provenance['binary_sha256'])
        state = self.snapshot(container,name)
        assert all(r['owner']=='console_app' for r in state['relations']), 'fixture not staging'
        assert state['counts'] == [0]*6, 'dormant staging catalog contains rows'
        return state

    def descriptor(self, container, database):
        # A protected fixture admin captures identity BEFORE configuring wrapper;
        # no production wrapper obtains its expected descriptor from target reads.
        _, text = self.psql(container,database,
            "SELECT system_identifier::text,current_database(),(SELECT oid FROM pg_database WHERE datname=current_database()) FROM pg_control_system()")
        systemid, name, oid = text.strip().split('|')
        return dict(ACCOUNT_CUSTODY_EXPECTED_OPERATOR='wrapper_admin',
            ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER=systemid,
            ACCOUNT_CUSTODY_EXPECTED_DATABASE=name, ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=oid,
            ACCOUNT_CUSTODY_EXPECTED_TLS_HOST='localhost')

    def wrapper(self, container, database, expected, **changes):
        values = dict(expected, POSTGRES_HOST='localhost',POSTGRES_PORT='5432',POSTGRES_DB=database,
            POSTGRES_ADMIN_USER='wrapper_admin',POSTGRES_ADMIN_PASSWORD_FILE='/operator.password',
            PGSSLROOTCERT='/wrapper-tls/ca.crt',PGSSLMODE='verify-full')
        assets = changes.pop('assets','/wrapper-assets')
        observer = changes.pop('observer', False)
        assert type(observer) is bool, 'test profile must be explicit boolean'
        for key,value in changes.items():
            if value is None:
                values.pop(key,None)
            else:
                values[key]=value
        file=self.private_file('wrapper.env',''.join(k+'='+v+'\n' for k,v in values.items()))
        command = ['docker','exec','--env-file',str(file),container,'bash',assets+'/'+ASSETS[0]]
        if observer:
            command.append('--with-durability-observer')
        return self.run(command,required=False)

    def rejection(self, name, container, database, descriptor, error, **changes):
        before=self.snapshot(container,database)
        # Successful real administrator observation prevents transport errors
        # from masquerading as target/catalog refusals.
        code,text=self.wrapper(container,database,descriptor,**changes)
        assert code != 0 and re.search(error,text,re.M), 'wrong refusal cause: '+name
        assert self.snapshot(container,database)==before, 'refusal mutated custody/ledger: '+name

    def case(self, name, operation):
        start=len(self.commands)
        try:
            operation()
        except Exception as error:
            self.results.append({'name':name,'status':'FAIL','error':str(error),'location':safe_failure_location(error),'commands':[start,len(self.commands)]})
            raise
        self.results.append({'name':name,'status':'PASS','commands':[start,len(self.commands)]})

    def execute(self):
        tracked = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '-z',
            'backend', 'ops', 'rust-toolchain.toml', '.cargo']).decode().split('\0')
        paths = [name for name in tracked if name]
        paths += ['ops/'+name for name in ASSETS]
        paths += [NATIVE_FINAL9_FIXTURE, NATIVE_EXTENDED227_FIXTURE, *[path for path,_ in NATIVE_EXTENDED227_CLASSIFIERS.values()],
                  'backend/app/tests/auth_rest/historical227.rs',
                  'backend/app/tests/auth_rest/fixtures/account-native-extended227-root-17fba595.sql',
                  'backend/app/tests/auth_rest/fixtures/account-native-extended227-credentials-17fba595.sql',
                  'backend/app/tests/auth_rest/fixtures/account-native-extended227-verify-17fba595.sql',
                  'backend/app/tests/auth_rest/fixtures/historical-migrations227-9de5c767.json']
        paths += [str(path.relative_to(ROOT)) for path in (ROOT/'backend/crates/platform/db/migrations').glob('*.sql')]
        self.frozen_sources = source_snapshot(ROOT, paths)
        self.provenance['prebuild_sources'] = self.frozen_sources
        self.run(['docker','image','inspect',IMAGE])
        # No other lane may replace this executable through a shared Cargo cache.
        owned_target=self.private/'cargo-target'
        build_env=dict(self.env,RUSTUP_TOOLCHAIN='1.98.1',RUSTC_WRAPPER='',CARGO_INCREMENTAL='1',
                       CARGO_TARGET_DIR=str(owned_target))
        self.run(['cargo','build','--locked','--manifest-path',str(ROOT/'backend/Cargo.toml'),
                  '-p','console-app','--bin','console-app'],env=build_env,timeout=900)
        self.assert_sources_unchanged()
        binary=self.private/'console-app-frozen'
        self.provenance['binary_sha256']=freeze_executable(owned_target/'debug/console-app',binary)
        self.provenance['build_target_ownership']='unique private target; no inherited shared executable'
        self.build_historical227_producer(build_env, owned_target)
        self.provenance['production_assets']={name:hashlib.sha256((ROOT/'ops'/name).read_bytes()).hexdigest() for name in ASSETS}
        self.provenance['migrations']={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((ROOT/'backend/crates/platform/db/migrations').glob('*.sql'))}
        self.certificate()
        first,second=self.cluster(),self.cluster()
        self.operator_jit_configuration_cases(first)
        for container,name,raw in ((first,'wrapper_positive',False),(first,'wrapper_subject',False),
            (first,'wrapper_other',False),(second,'wrapper_subject',False),(first,'wrapper_raw',True)):
            self.database(container,name,binary,raw=raw)
        expected=self.descriptor(first,'wrapper_subject')
        positive_expected=self.descriptor(first,'wrapper_positive')
        positive_before=self.snapshot(first,'wrapper_positive')
        expected_roots=[{'id':row['id'],'created_at':row['created_at']}
                        for row in positive_before['user_rows']]
        assert expected_roots, 'actual migrator must supply a nonempty root-backfill control'
        def positive():
            code,text=self.wrapper(first,'wrapper_positive',positive_expected)
            assert code==0 and 'account_custody.native_finalized' in text, 'normal wrapper did not finalize'
            state=self.snapshot(first,'wrapper_positive')
            assert all(r['owner']==OWNERS[r['name']] for r in state['relations'])
            assert state['account_rows']==expected_roots, 'finalization did not copy exact user UUIDs and timestamps'
            assert state['user_rows']==positive_before['user_rows'], 'finalization changed pre-existing users'
            assert state['ledger']==positive_before['ledger'], 'finalization changed the real SQLx ledger'
            # Read the existing serving classifiers independently after the
            # packaged transaction. A printed label alone cannot prove readiness.
            for artifact, expected_state in (
                    ('account_custody_state.sql', 'account_custody.native_finalized'),
                    ('account_credential_custody_state.sql', 'account_credentials.native_finalized')):
                query = (ROOT/'backend/app/src'/artifact).read_text()
                _, actual = self.psql(first, 'wrapper_positive',
                    'SET search_path=pg_catalog,pg_temp;\n'+query)
                assert actual.strip().splitlines() in ([expected_state], ['SET', expected_state]), 'packaged success is not a native serving profile'
        self.case('valid_finalize',positive)
        def repeat():
            before=self.snapshot(first,'wrapper_positive')
            positive()
            assert self.snapshot(first,'wrapper_positive')==before, 'retry changed state'
        self.case('already_finalized',repeat)
        def reject(name, error, *, container=first, database='wrapper_subject', descriptor=expected, **changes):
            self.case(name,lambda:self.rejection(name,container,database,descriptor,error,**changes))
        reject('wrong_expected_operator',r'account_custody.operator_identity_mismatch',
               ACCOUNT_CUSTODY_EXPECTED_OPERATOR='wrong_admin')
        app_password=self.private_file('app.password',self.passwords['CONSOLE_APP_POSTGRES_PASSWORD']+'\n')
        self.copy(first,app_password,'/app.password')
        code,text=self.psql(first,'wrapper_subject','SELECT current_user,current_database(),1',
                           role='console_app',password=self.passwords['CONSOLE_APP_POSTGRES_PASSWORD'])
        assert code==0 and text.strip()=='console_app|wrapper_subject|1'
        reject('ordinary_migration_login',r'account_custody.operator_identity_mismatch',
            POSTGRES_ADMIN_USER='console_app',POSTGRES_ADMIN_PASSWORD_FILE='/app.password',
            ACCOUNT_CUSTODY_EXPECTED_OPERATOR='console_app')
        reject('missing_descriptor',r'account_custody.descriptor_invalid',ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER=None)
        reject('wrong_database_descriptor',r'account_custody.target_mismatch',ACCOUNT_CUSTODY_EXPECTED_DATABASE='wrong_db')
        reject('wrong_database_oid',r'account_custody.target_mismatch',
               ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=str(int(expected['ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID'])+1))
        reject('identically_migrated_wrong_database',r'account_custody.target_mismatch',database='wrapper_other')
        second_identity=self.descriptor(second,'wrapper_subject')
        assert second_identity['ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER'] != expected['ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER']
        reject('wrong_cluster',r'account_custody.target_mismatch',container=second,
               ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID=second_identity['ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID'])
        reject('missing_ledger',r'account_custody.migration_ledger_missing',database='wrapper_raw',
               descriptor=self.descriptor(first,'wrapper_raw'))
        variant=self.private/'wrong-release'
        variant.mkdir()
        for name in ASSETS:
            shutil.copyfile(ROOT/'ops'/name,variant/name)
        manifest=variant/ASSETS[2]
        lines=manifest.read_text().splitlines()
        assert lines and re.fullmatch(r'[1-9][0-9]*\t[0-9a-f]{96}',lines[-1]), 'invalid release manifest fixture'
        version,digest=lines[-1].split('\t')
        lines[-1]=version+'\t'+('0' if digest[0]!='0' else '1')+digest[1:]
        manifest.write_text('\n'.join(lines)+'\n')
        self.copy(first,variant,'/wrong-release')
        reject('release_checksum_mismatch',r'account_custody.migration_checksum_mismatch',assets='/wrong-release')
        reject('wrong_tls_ca',r'(certificate verify failed|certificate verification failed)',PGSSLROOTCERT='/wrapper-tls/wrong-ca.crt')
        reject('wrong_tls_hostname',r'(does not match host name|certificate verify failed)',
               POSTGRES_HOST='127.0.0.1',ACCOUNT_CUSTODY_EXPECTED_TLS_HOST='127.0.0.1')

        def default_order():
            before = self.composed_snapshot(first, 'wrapper_positive')
            code, text = self.traced_wrapper(first, 'wrapper_positive', positive_expected)
            assert code == 0 and 'account_custody.native_finalized' in text
            assert self.composed_snapshot(first, 'wrapper_positive') == before, 'composed retry changed state'
        self.case('composed_default_installer_order', default_order)

        def observer_order():
            code, text = self.traced_wrapper(first, 'wrapper_positive', positive_expected, observer=True)
            assert code == 0 and 'account_custody.native_finalized' in text
            _, text = self.psql(first, 'wrapper_positive',
                "SELECT count(*) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname='public' AND p.proname='console_durability_observation_v1'")
            assert text.strip() == '1', 'optional observer was not actually installed'
        self.case('composed_observer_installer_order', observer_order)

        def missing_credentials():
            variant = self.private/'missing-credential'
            variant.mkdir()
            for name in ASSETS:
                if name != 'postgres-finalize-account-credentials.sql':
                    shutil.copyfile(ROOT/'ops'/name, variant/name)
            self.copy(first, variant, '/missing-credential')
            before = self.composed_snapshot(first, 'wrapper_subject')
            code, text = self.traced_wrapper(first, 'wrapper_subject', expected, assets='/missing-credential')
            assert code != 0 and re.search(r'postgres-finalize-account-credentials\.sql: No such file or directory', text), 'missing credential artifact did not cause the expected file refusal'
            assert self.composed_snapshot(first, 'wrapper_subject') == before, 'missing credential artifact committed partial custody'
        self.case('missing_credential_artifact_rolls_back', missing_credentials)

        def observer_rollback():
            # The previous real observer installation created the cluster role
            # in another database. Its absence here is an actual partial state,
            # refused by the unchanged observer installer without fake SQL.
            _, text = self.psql(first, 'wrapper_subject',
                "SELECT EXISTS(SELECT 1 FROM pg_roles WHERE rolname='console_durability_observer'),to_regprocedure('public.console_durability_observation_v1(name,oid)') IS NULL")
            assert text.strip() == 't|t', 'observer partial-state prerequisite missing'
            before = self.composed_snapshot(first, 'wrapper_subject')
            code, text = self.traced_wrapper(first, 'wrapper_subject', expected, observer=True)
            assert code != 0 and 'durability_observer.partial_installation' in text, 'wrong optional-observer refusal'
            assert self.composed_snapshot(first, 'wrapper_subject') == before, 'observer refusal committed root or credential custody'
        self.case('observer_failure_rolls_back_composed_custody', observer_rollback)

        def missing_native_catalogs():
            database = 'wrapper_missing_native'
            self.database(first, database, binary)
            descriptor = self.descriptor(first, database)
            before_fault = self.composed_snapshot(first, database)
            assert_current_migration_ledger(ROOT, before_fault['root']['ledger'], self.frozen_sources['files'])
            # Fault only this disposable database. Preserve the real successful
            # ledger; never fabricate migration evidence or run fixture owners.
            self.psql(first, database,
                'DROP TABLE public.account_context_candidates,public.company_actors')
            _, absent = self.psql(first, database,
                "SELECT to_regclass('public.company_actors') IS NULL,"
                "to_regclass('public.account_context_candidates') IS NULL")
            assert absent.strip() == 't|t', 'both native catalogs must actually be absent'
            before = self.composed_snapshot(first, database)
            assert before['root']['ledger'] == before_fault['root']['ledger'], 'fault changed SQLx ledger'
            code, text = self.traced_wrapper(first, database, descriptor)
            assert code != 0 and 'account_native.finalization_incomplete' in text, 'missing native catalogs did not cause exact postcondition refusal'
            assert 'account_custody.native_finalized' not in text, 'refusal advertised native success'
            assert self.composed_snapshot(first, database) == before, 'postcondition refusal committed partial root or credential custody'
            _, absent = self.psql(first, database,
                "SELECT to_regclass('public.company_actors') IS NULL,"
                "to_regclass('public.account_context_candidates') IS NULL")
            assert absent.strip() == 't|t', 'operator repaired missing catalogs'
        self.case('missing_both_native_catalogs_with_exact_ledger', missing_native_catalogs)
        self.native_extension_cases(first, binary)

    def cleanup(self):
        try:
            return cleanup_containers(self.containers, self.run)
        finally:
            self.temp.cleanup()


def machinery_tests():
    """Tests this evidence machinery only, not fake application implementations."""
    import unittest

    class Machinery(unittest.TestCase):
        def test_historical227_producer_requires_complete_unique_success(self):
            witness = 'HISTORICAL227_PRODUCER_PASS'
            leaf = 'test historical227::historical227_operator_owned_database_producer ... '
            summary = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.1s'
            positive = 'running 1 test\n'+witness+'\n'+leaf+'ok\n'+summary+'\n'
            split = 'running 1 test\n'+leaf+witness+'\nok\n'+summary+'\n'
            self.assertTrue(historical227_success(positive))
            self.assertTrue(historical227_success(split))
            for bad in (
                positive.replace(witness, ''), positive+witness+'\n', positive+summary+'\n',
                positive+'test result: FAILED. 0 passed; 1 failed;\n',
                positive.replace('1 passed', '0 passed'), positive.replace('0 failed', '1 failed'),
                positive.replace('0 ignored', '1 ignored'), positive.replace('running 1 test', 'running 0 tests'),
                positive.replace(witness, 'untrusted prefix '+witness),
                positive+'running 0 tests\n', positive+'untrusted prefix '+witness+'\n',
                positive.replace(leaf+'ok\n', ''), positive.replace(leaf, 'test wrong ... '),
                positive+leaf+'ok\n', positive.replace(leaf+'ok', leaf+'FAILED'),
                positive.replace(leaf+'ok', leaf+'ignored'), split.replace('\nok\n', '\n'),
                split.replace('\nok\n', '\nFAILED\n'),
            ):
                self.assertFalse(historical227_success(bad))

        def test_psql_streams_large_utf8_sql_without_command_argument(self):
            suite = object.__new__(Suite)
            suite.passwords = {'POSTGRES_ADMIN_PASSWORD': 'fixture-default-password'}
            files = []
            calls = []
            def private_file(name, text):
                files.append((name, text))
                return Path('/private/fixture-psql.env')
            def execute(argv, **options):
                calls.append((argv, options))
                return 9, 'fixture-result'
            suite.private_file = private_file
            suite.run = execute
            sql = "SELECT '한글';\n--" + ('x' * 140000)
            actual = suite.psql('fixture-container', 'fixture-database', sql,
                role='fixture-role', password='fixture-explicit-password', required=False)
            self.assertEqual(actual, (9, 'fixture-result'))
            self.assertEqual(files, [('psql.env', 'PGPASSWORD=fixture-explicit-password\n')])
            self.assertEqual(len(calls), 1)
            argv, options = calls[0]
            self.assertEqual(argv, ['docker','exec','-i','--env-file',
                '/private/fixture-psql.env','fixture-container','psql','-X','-w',
                '-h','localhost','-U','fixture-role','-d','fixture-database',
                '-v','ON_ERROR_STOP=1','-At','-F','|','--file','-'])
            self.assertEqual(options, {'stdin': sql.encode('utf-8'), 'required': False})
            for value in (sql, *suite.passwords.values(), 'fixture-explicit-password'):
                self.assertNotIn(value, argv)

        def test_composed_psql_order_accepts_both_fixed_profiles(self):
            for observer in (False, True):
                argv = ['-X', '-w', '--set', 'ON_ERROR_STOP=1', '--single-transaction',
                        '--file', '/tmp/console-account-custody.Abc123/preflight.sql',
                        '--file', '/assets/postgres-finalize-account-custody.sql',
                        '--file', '/assets/postgres-finalize-account-credentials.sql']
                if observer:
                    argv += ['--file', '/assets/postgres-install-durability-observer.sql']
                argv += ['--file', '/assets/postgres-verify-account-native.sql']
                assert_composed_psql_trace('BEGIN_PSQL\0'+'\0'.join(argv)+'\0', '/assets', observer)

        def test_composed_psql_order_rejects_omission_extra_substitution_reorder_and_split(self):
            argv = ['-X', '-w', '--set', 'ON_ERROR_STOP=1', '--single-transaction',
                    '--file', '/tmp/console-account-custody.Abc123/preflight.sql',
                    '--file', '/assets/postgres-finalize-account-custody.sql',
                    '--file', '/assets/postgres-finalize-account-credentials.sql',
                    '--file', '/assets/postgres-install-durability-observer.sql',
                    '--file', '/assets/postgres-verify-account-native.sql']
            mutations = [argv[:-2], argv[:-4]+argv[-2:], argv+['--file', '/assets/extra.sql'],
                         [v.replace('postgres-finalize-account-credentials.sql', 'substituted.sql') for v in argv],
                         argv[:-4]+argv[-2:]+argv[-4:-2],
                         [v for v in argv if v != '--single-transaction'],
                         argv+['BEGIN_PSQL'], argv+['-c', 'COMMIT'],
                         [v for v in argv if v != '-X'],
                         [v.replace('ON_ERROR_STOP=1', 'ON_ERROR_STOP=0') for v in argv],
                         argv+['-cCOMMIT'], argv+['-f/tmp/extra.sql'],
                         argv+['--set', 'ON_ERROR_STOP=0'],
                         argv+['--variable=ON_ERROR_STOP=0'],
                         argv+['--file=/tmp/extra.sql']]
            for mutation in mutations:
                with self.subTest(argv=mutation), self.assertRaises(AssertionError):
                    assert_composed_psql_trace('BEGIN_PSQL\0'+'\0'.join(mutation)+'\0', '/assets', True)

        def test_operator_child_execution_settings_positive(self):
            trace = '\0'.join(['BEGIN_OPERATOR_ENV', 'PGOPTIONS',
                '-c search_path=pg_catalog,pg_temp -c statement_timeout=60000 -c lock_timeout=5000 -c jit=off',
                'PGSSLMODE', 'verify-full', 'PGGSSENCMODE', 'disable',
                'PGCONNECT_TIMEOUT', '10', 'END_OPERATOR_ENV', ''])
            assert_operator_child_environment(trace)

        def test_operator_child_execution_settings_reject_corruption(self):
            trace = '\0'.join(['BEGIN_OPERATOR_ENV', 'PGOPTIONS',
                '-c search_path=pg_catalog,pg_temp -c statement_timeout=60000 -c lock_timeout=5000 -c jit=off',
                'PGSSLMODE', 'verify-full', 'PGGSSENCMODE', 'disable',
                'PGCONNECT_TIMEOUT', '10', 'END_OPERATOR_ENV', ''])
            corruptions = [trace.replace(' -c jit=off', ''), trace.replace('jit=off', 'jit=on'),
                trace.replace('jit=off', 'jit=off -c jit=on'), trace.replace('jit=off', 'jit=off -c jit=off'),
                trace.replace('60000', '120000'), trace.replace('60000', '1000'),
                trace.replace('5000', '10000'), trace.replace('5000', '100'),
                trace.replace('pg_catalog,pg_temp', 'public'), trace.replace('verify-full', 'require'),
                trace.replace('disable', 'prefer'), trace.replace('\0'+'10'+'\0', '\0'+'0'+'\0'),
                trace.replace('END_OPERATOR_ENV', ''), trace[:-1], trace+trace,
                trace.replace('BEGIN_OPERATOR_ENV', 'FORGED_OPERATOR_ENV')]
            for malformed in corruptions:
                with self.subTest(trace=malformed), self.assertRaises(AssertionError):
                    assert_operator_child_environment(malformed)

        def test_psql_verifying_tls_uses_fixed_ca_without_changing_sql(self):
            suite = object.__new__(Suite)
            suite.passwords = {'POSTGRES_ADMIN_PASSWORD':'fixture-private-password'}
            files = []
            calls = []
            suite.private_file = lambda name,text: (files.append((name,text)) or Path('/private/psql.env'))
            suite.run = lambda argv,**options: (calls.append((argv,options)) or (0,'fixture-observed'))
            self.assertEqual(suite.psql('fixture-container','postgres','SELECT 1',verifying_tls=True), (0,'fixture-observed'))
            self.assertEqual(files, [('psql.env','PGPASSWORD=fixture-private-password\nPGSSLMODE=verify-full\nPGSSLROOTCERT=/wrapper-tls/ca.crt\nPGGSSENCMODE=disable\n')])
            self.assertEqual(calls[0][1]['stdin'], b'SELECT 1')
            self.assertNotIn('fixture-private-password', calls[0][0])
            self.assertIn('--env-file', calls[0][0])

        def test_operator_configuration_collects_both_real_case_failures(self):
            suite = object.__new__(Suite)
            suite.commands = []
            suite.results = []
            suite.descriptor = lambda *args: {}
            suite.psql = lambda *args,**kwargs: (0,'on|wrapper_admin|wrapper_admin|postgres|t\n')
            observed_profiles = []
            def fail_wrapper(*args,**kwargs):
                observed_profiles.append(kwargs['observer'])
                raise AssertionError('TEST_ONLY injected child configuration failure')
            suite.traced_wrapper = fail_wrapper
            with self.assertRaisesRegex(AssertionError, 'each exact failure is retained'):
                suite.operator_jit_configuration_cases('fixture-container', collect_failures=True)
            self.assertEqual(observed_profiles, [False,True])
            self.assertEqual([(r['name'],r['status']) for r in suite.results],
                [('operator_jit_default_child_settings','FAIL'),('operator_jit_observer_child_settings','FAIL')])

        def test_operator_assets_have_exact_six_without_losing_manifest_index(self):
            self.assertEqual(len(ASSETS), 6)
            self.assertEqual(set(ASSETS), {'postgres-finalize-account-custody.sh',
                'postgres-finalize-account-custody.sql', 'postgres-finalize-account-credentials.sql',
                'postgres-install-durability-observer.sql', 'account-custody-migrations.sha384',
                'postgres-verify-account-native.sql'})
            self.assertEqual(ASSETS[2], 'account-custody-migrations.sha384')

        def test_exact_unique_roster_positive(self):
            self.assertTrue(complete_case_roster([{'name':n,'status':'PASS'} for n in CASES]))

        def test_duplicate_cannot_replace_omitted_case(self):
            rows=[{'name':n,'status':'PASS'} for n in CASES]
            rows[-1]=dict(rows[0])
            self.assertFalse(complete_case_roster(rows))

        def test_missing_unknown_and_failed_case_refuse(self):
            rows=[{'name':n,'status':'PASS'} for n in CASES]
            self.assertFalse(complete_case_roster(rows[:-1]))
            for replacement in ({'name':'unregistered','status':'PASS'},
                                {'name':CASES[-1],'status':'FAIL'}):
                self.assertFalse(complete_case_roster(rows[:-1]+[replacement]))

        def test_lost_create_reply_still_registers_cleanup_intent(self):
            with tempfile.TemporaryDirectory() as directory:
                suite=Suite(Path(directory))
                def lost_reply(argv, **kwargs):
                    self.assertEqual(argv[:2], ['docker','run'])
                    self.assertEqual(len(suite.containers),1)
                    raise TimeoutError('created resource, reply lost')
                suite.run=lost_reply
                try:
                    with self.assertRaises(TimeoutError):
                        suite.cluster()
                    self.assertEqual(len(suite.containers),1)
                finally:
                    suite.temp.cleanup()

        def test_cleanup_attempts_all_after_lost_reply(self):
            calls=[]
            def execute(argv, **kwargs):
                calls.append(argv)
                if argv[:3]==['docker','rm','-f']:
                    raise TimeoutError('reply lost')
                return 1,'Error: No such container: '+argv[-1]+'\n'
            self.assertEqual(cleanup_containers(['first','second'],execute),[])
            self.assertEqual([a[-1] for a in calls],['second','second','first','first'])

        def test_unavailable_inspection_is_not_absence(self):
            calls=[]
            def execute(argv, **kwargs):
                calls.append(argv)
                return 1,'Cannot connect to Docker daemon'
            self.assertEqual(cleanup_containers(['first','second'],execute),['second','first'])
            self.assertEqual(len(calls),4)

        def test_docker28_lowercase_absence_is_recognized(self):
            def execute(argv, **kwargs):
                return 1,'[]\nerror: no such object: '+argv[-1]+'\n'
            self.assertEqual(cleanup_containers(['Intended'],execute),[])

        def test_docker29_daemon_absence_is_recognized(self):
            for kind in ('container', 'object'):
                def execute(argv, **kwargs):
                    return 1, '[]\nError response from daemon: No such '+kind+': '+argv[-1]+'\n'
                self.assertEqual(cleanup_containers(['Intended'],execute),[])

        def test_absence_requires_failure_and_exact_resource_name(self):
            for code, message in (
                    (0,'error: no such object: Intended'),
                    (1,'error: no such object: intended'),
                    (1,'error: no such object: Intended-other'),
                    (1,'error: no such object: Other'),
                    (1,'daemon error: no such object: Intended'),
                    (0,'Error response from daemon: No such container: Intended'),
                    (1,'Error response from daemon: No such container: Other'),
                    (1,'Error response from daemon: No such container: intended'),
                    (1,'Error response from daemon: No such container: Intended-other')):
                with self.subTest(code=code,message=message):
                    def execute(argv, **kwargs):
                        return code,message+'\n'
                    self.assertEqual(cleanup_containers(['Intended'],execute),['Intended'])

        def test_private_cleanup_runs_despite_inspection_exception(self):
            with tempfile.TemporaryDirectory() as directory:
                suite=Suite(Path(directory))
                private=suite.private
                suite.containers=['intended']
                def fail(argv, **kwargs):
                    raise OSError('unavailable')
                suite.run=fail
                self.assertEqual(suite.cleanup(),['intended'])
                self.assertFalse(private.exists())

        def test_frozen_executable_survives_build_output_replacement(self):
            with tempfile.TemporaryDirectory() as directory:
                root=Path(directory); built=root/'built'; frozen=root/'frozen'
                built.write_text('#!/bin/sh\nprintf reviewed\n');built.chmod(0o700)
                digest=freeze_executable(built,frozen)
                built.write_text('#!/bin/sh\nprintf replaced\n')
                assert_executable_unchanged(frozen,digest)
                self.assertEqual(subprocess.check_output([str(frozen)]),b'reviewed')

        def test_frozen_executable_mutation_refuses_before_execution(self):
            with tempfile.TemporaryDirectory() as directory:
                root=Path(directory);built=root/'built';frozen=root/'frozen'
                built.write_text('#!/bin/sh\nexit 0\n');built.chmod(0o700)
                digest=freeze_executable(built,frozen)
                frozen.chmod(0o700);frozen.write_text('#!/bin/sh\nexit 1\n')
                with self.assertRaises(AssertionError):
                    assert_executable_unchanged(frozen,digest)

        def test_real_source_change_missing_and_symlink_refuse(self):
            with tempfile.TemporaryDirectory() as directory:
                root=Path(directory)
                def git(*args):
                    return subprocess.check_output(['git','-C',str(root),
                        '-c','user.name=Wrapper Machinery','-c','user.email=wrapper@test.invalid',
                        '-c','core.hooksPath=/dev/null','-c','commit.gpgsign=false',*args],stderr=subprocess.STDOUT)
                git('init','-q')
                source=root/'source'
                source.write_text('original source\n')
                git('add','source');git('commit','-qm','fixture')
                before=source_snapshot(root,['source'])
                self.assertEqual(source_snapshot(root,['source']),before)
                source.write_text('changed source\n')
                self.assertNotEqual(source_snapshot(root,['source']),before)
                source.unlink()
                with self.assertRaises(AssertionError): source_snapshot(root,['source'])
                other=root/'other';other.write_text('original source\n');source.symlink_to(other)
                with self.assertRaises(AssertionError): source_snapshot(root,['source'])

        def test_real_head_change_refuses_even_when_selected_bytes_same(self):
            with tempfile.TemporaryDirectory() as directory:
                root=Path(directory)
                def git(*args):
                    return subprocess.check_output(['git','-C',str(root),
                        '-c','user.name=Wrapper Machinery','-c','user.email=wrapper@test.invalid',
                        '-c','core.hooksPath=/dev/null','-c','commit.gpgsign=false',*args],stderr=subprocess.STDOUT)
                git('init','-q');(root/'source').write_text('fixed\n')
                git('add','source');git('commit','-qm','first')
                before=source_snapshot(root,['source'])
                git('commit','--allow-empty','-qm','second')
                after=source_snapshot(root,['source'])
                self.assertEqual(before['files'],after['files'])
                self.assertNotEqual(before,after)

        def seal_native_metadata(self, state):
            # Test-only encoder for plain checker observations, never production
            # canonicalization or an authorization/profile issuer.
            def encoded(value):
                if type(value) is Decimal: return str(value)
                if type(value) is dict: return '{'+','.join(json.dumps(k)+':'+encoded(v) for k,v in sorted(value.items()))+'}'
                if type(value) is list: return '['+','.join(encoded(v) for v in value)+']'
                return json.dumps(value,allow_nan=False)
            raw = encoded(state['metadata']['snapshot'])
            state['metadata']['snapshot_text'] = raw
            state['metadata']['sha256'] = hashlib.sha256(raw.encode()).hexdigest()

        def native_oracle_fixture(self):
            # Synthetic plain observations test only this checker. They never
            # issue a credential or replace a real owner/database fixture.
            import copy
            row = {'metadata': {'schema':'public','name':'old','identity_arguments':'',
                'owner':'console_account_owner','security_definer':True,'acl':[], 'source_sha256':'old'}, 'extra_valid':True}
            snapshot = {'tables':[{'name':name} for name in NATIVE_TABLES], 'routines':[row], 'roles':[['original']]}
            before = {'data':[{'name':name,'count':1,'rows':[{'id':name,'value':'preserved'}]} for name in NATIVE_TABLES],
                'users':[{'id':'legacy'}], 'ledger':[{'version':version,'success':True,'checksum':'checker-only-placeholder'} for version in range(1,228)],
                'roles':[{'rolname':'owner'}], 'memberships':[],
                'routines':[{'key':'public.old()','catalog':{'prosrc':'old'},'definition':'old'}],
                'metadata':{'snapshot':snapshot,'sha256':NATIVE_FINAL9_SHA256}}
            after = copy.deepcopy(before); after['metadata']['sha256'] = 'e'*64
            for name, (arguments, owner, caller) in NATIVE_HELPERS.items():
                after['metadata']['snapshot']['routines'].append({'metadata':{
                    'schema':'public','name':name,'identity_arguments':arguments,'owner':owner,'security_definer':True,
                    'acl':[[owner,owner,'EXECUTE',False],[owner,caller,'EXECUTE',False]], 'source_sha256':'new'},'extra_valid':True})
                after['routines'].append({'key':'public.'+name+'('+arguments+')','catalog':{'prosrc':'new'},'definition':'new'})
            self.seal_native_metadata(before); self.seal_native_metadata(after)
            return before, after

        def test_native_oracle_accepts_only_exact_additive_transition(self):
            from unittest.mock import patch
            before, after = self.native_oracle_fixture()
            with patch.dict(globals(),NATIVE_FINAL9_SHA256=before['metadata']['sha256'],NATIVE_EXTENSION_SHA256=after['metadata']['sha256']):
                assert_native_extension(before, after)

        def test_native_oracle_rejects_each_missing_duplicate_null_table(self):
            import copy
            before, _ = self.native_oracle_fixture()
            for index in range(15):
                for fault in ('missing','duplicate','null'):
                    damaged = copy.deepcopy(before)
                    if fault == 'missing': damaged['data'].pop(index)
                    elif fault == 'duplicate': damaged['data'][index] = damaged['data'][(index+1)%15]
                    else: damaged['data'][index] = None
                    with self.subTest(index=index,fault=fault), self.assertRaises((AssertionError,KeyError,TypeError)):
                        validate_native_state(damaged)

        def test_native_oracle_rejects_changed_or_omitted_row_in_every_table(self):
            import copy
            from unittest.mock import patch
            before, after = self.native_oracle_fixture()
            for index in range(15):
                for fault in ('changed','omitted','null','duplicate'):
                    damaged = copy.deepcopy(after); table = damaged['data'][index]
                    if fault == 'changed': table['rows'][0]['value'] = 'lost original'
                    elif fault == 'omitted': table['rows'] = []; table['count'] = 0
                    elif fault == 'null': table['rows'][0] = None
                    else: table['rows'] *= 2; table['count'] = 2
                    with self.subTest(index=index,fault=fault), patch.dict(globals(),NATIVE_FINAL9_SHA256=before['metadata']['sha256'],NATIVE_EXTENSION_SHA256=after['metadata']['sha256']), self.assertRaises(AssertionError):
                        assert_native_extension(before, damaged)

        def test_native_oracle_rejects_old_routine_ledger_and_role_mutation(self):
            import copy
            from unittest.mock import patch
            before, after = self.native_oracle_fixture()
            for field in ('native_routine','independent_routine','ledger','roles','memberships','users'):
                damaged = copy.deepcopy(after)
                if field == 'native_routine': damaged['metadata']['snapshot']['routines'][0]['metadata']['source_sha256'] = 'changed'
                elif field == 'independent_routine': damaged['routines'][0]['definition'] = 'changed'
                elif field == 'memberships': damaged[field].append({'member':'unexpected'})
                else: damaged[field][0]['unexpected'] = 'changed'
                self.seal_native_metadata(damaged)
                with self.subTest(field=field), patch.dict(globals(),NATIVE_FINAL9_SHA256=before['metadata']['sha256'],NATIVE_EXTENSION_SHA256=damaged['metadata']['sha256']), self.assertRaises(AssertionError):
                    assert_native_extension(before, damaged)

        def test_native_oracle_rejects_missing_duplicate_and_overloaded_helper(self):
            import copy
            from unittest.mock import patch
            before, after = self.native_oracle_fixture()
            for fault in ('missing','duplicate','overload'):
                damaged = copy.deepcopy(after); rows = damaged['metadata']['snapshot']['routines']
                if fault == 'missing': rows.pop()
                elif fault == 'duplicate': rows[-1] = copy.deepcopy(rows[-2])
                else: rows[-1]['metadata']['identity_arguments'] = 'p_account text'
                self.seal_native_metadata(damaged)
                with self.subTest(fault=fault), patch.dict(globals(),NATIVE_FINAL9_SHA256=before['metadata']['sha256'],NATIVE_EXTENSION_SHA256=damaged['metadata']['sha256']), self.assertRaises(AssertionError):
                    assert_native_extension(before, damaged)

        def test_native_oracle_rejects_helper_capability_and_profile_drift(self):
            import copy
            from unittest.mock import patch
            before, after = self.native_oracle_fixture()
            for fault in ('public','grant_option','grant_boolean_integer','owner','invoker','pin','old_pin'):
                damaged = copy.deepcopy(after); row = damaged['metadata']['snapshot']['routines'][-1]['metadata']
                if fault == 'public': row['acl'].append([row['owner'],'PUBLIC','EXECUTE',False])
                elif fault == 'grant_option': row['acl'][-1][-1] = True
                elif fault == 'grant_boolean_integer': row['acl'][-1][-1] = 0
                elif fault == 'owner': row['owner'] = 'wrapper_admin'
                elif fault == 'invoker': row['security_definer'] = False
                elif fault == 'pin': damaged['metadata']['sha256'] = 'f'*64
                else: damaged['metadata']['sha256'] = NATIVE_FINAL9_SHA256
                expected = after['metadata']['sha256']
                if fault not in ('pin','old_pin'):
                    # Test-only expected profile admits this deliberately wrong
                    # body so the explicit owner/EXECUTE/definer oracle must fail.
                    self.seal_native_metadata(damaged)
                    expected = damaged['metadata']['sha256']
                with self.subTest(fault=fault), patch.dict(globals(),NATIVE_FINAL9_SHA256=before['metadata']['sha256'],NATIVE_EXTENSION_SHA256=expected), self.assertRaises(AssertionError):
                    assert_native_extension(before, damaged)

        def test_native_literal_reader_never_executes_and_rejects_duplicates(self):
            self.assertEqual(source_literal("raise RuntimeError('must not execute')\nVALUE: str | None = None\n", 'VALUE'), None)
            for source in ('OTHER=1', 'VALUE=1\nVALUE=2', 'VALUE=compute()'):
                with self.assertRaises((AssertionError, ValueError)):
                    source_literal(source, 'VALUE')

        def test_native_private_observation_never_enters_command_log(self):
            from unittest.mock import patch
            from types import SimpleNamespace
            with tempfile.TemporaryDirectory() as directory:
                suite = Suite(Path(directory))
                private = b'{"credential_hash":"PRIVATE-ROW-CANARY"}\n'
                try:
                    with patch.object(subprocess, 'run', return_value=SimpleNamespace(stdout=private,returncode=0)):
                        _, actual = suite.run(['private-observation'],private_output=True)
                    self.assertEqual(actual, private.decode())
                    self.assertEqual((Path(directory)/'001.log').read_text(), '[PRIVATE SQL OUTPUT OMITTED]\n')
                    self.assertNotIn('PRIVATE-ROW-CANARY', json.dumps(suite.commands))
                finally:
                    suite.temp.cleanup()

        def test_native_private_timeout_never_logs_partial_rows(self):
            from unittest.mock import patch
            with tempfile.TemporaryDirectory() as directory:
                suite = Suite(Path(directory))
                try:
                    with patch.object(subprocess, 'run', side_effect=subprocess.TimeoutExpired('private-observation',1,output=b'PRIVATE-ROW-CANARY')):
                        with self.assertRaises(AssertionError):
                            suite.run(['private-observation'],private_output=True)
                    self.assertEqual((Path(directory)/'001.log').read_text(), '[PRIVATE SQL OUTPUT OMITTED]\n')
                finally:
                    suite.temp.cleanup()

        def test_native_lossless_json_distinguishes_boolean_integer_and_fraction(self):
            for left, right in (('true','1'), ('false','0'), ('1','1.0'), ('1.0','1.00'),
                    ('9007199254740992','9007199254740993'),
                    ('9007199254740993.1','9007199254740993.2'),
                    ('1.0000000000000000000000000000000001','1.0000000000000000000000000000000002')):
                with self.subTest(left=left,right=right):
                    a, b = decode_native_json('{"value":'+left+'}'), decode_native_json('{"value":'+right+'}')
                    self.assertFalse(exact_json_equal(a,b))
                    self.assertTrue(exact_json_equal(a,decode_native_json('{"value":'+left+'}')))
            for raw in ('{"same":1,"same":2}', 'NaN', 'Infinity', '-Infinity'):
                with self.assertRaises(ValueError): decode_native_json(raw)
            with self.assertRaises(AssertionError): exact_json_key(1.0)

        def test_native_lossless_row_and_routine_oracle_refuses_python_equal_drift(self):
            import copy
            from unittest.mock import patch
            for original, changed in ((False,0), (1,True),
                    (Decimal('9007199254740993.1'),Decimal('9007199254740993.2'))):
                for dimension in ('row','routine','acl'):
                    before, after = self.native_oracle_fixture()
                    if dimension == 'row':
                        before['data'][0]['rows'][0]['value'] = original
                        after['data'][0]['rows'][0]['value'] = changed
                    elif dimension == 'routine':
                        before['metadata']['snapshot']['routines'][0]['metadata']['cost'] = original
                        after['metadata']['snapshot']['routines'][0]['metadata']['cost'] = changed
                    else:
                        if original is not False: continue
                        after['metadata']['snapshot']['routines'][-1]['metadata']['acl'][-1][-1] = changed
                    with self.subTest(dimension=dimension), patch.dict(globals(),NATIVE_FINAL9_SHA256=before['metadata']['sha256'],NATIVE_EXTENSION_SHA256=after['metadata']['sha256']), self.assertRaises(AssertionError):
                        assert_native_extension(before, after)

        def test_native_observation_uses_fixed_read_only_search_path_and_private_output(self):
            before, _ = self.native_oracle_fixture()
            suite = object.__new__(Suite)
            calls = []
            def observed(container,database,sql,**options):
                calls.append((container,database,sql,options))
                return 0,json.dumps(before)
            suite.psql = observed
            actual = suite.native_observation('fixture-container','fixture-database','SELECT snapshot,snapshot_sha256 FROM fixed_capture')
            self.assertTrue(exact_json_equal(actual,before))
            self.assertEqual(len(calls),1)
            self.assertTrue(calls[0][2].startswith('BEGIN READ ONLY;\nSET LOCAL search_path=pg_catalog,pg_temp;\n'))
            self.assertTrue(calls[0][2].endswith(';\nCOMMIT;'))
            self.assertEqual(calls[0][3],{'private_output':True})

        def test_native_metadata_hash_label_cannot_hide_new_helper_body_or_config(self):
            import copy
            before, after = self.native_oracle_fixture()
            for field in ('source_sha256','config'):
                damaged = copy.deepcopy(after)
                damaged['metadata']['snapshot']['routines'][-1]['metadata'][field] = 'changed'
                with self.subTest(field=field), self.assertRaises(AssertionError):
                    validate_native_state(damaged)
                damaged['metadata']['snapshot_text'] += ' '
                with self.assertRaises(AssertionError): validate_native_state(damaged)

        def test_native_ledger_omission_duplicate_future_and_boolean_versions_refuse(self):
            import copy
            before, _ = self.native_oracle_fixture()
            for fault in ('missing','duplicate','future','boolean'):
                damaged = copy.deepcopy(before)
                if fault == 'missing': damaged['ledger'].pop(10)
                elif fault == 'duplicate': damaged['ledger'][10] = dict(damaged['ledger'][11])
                elif fault == 'future': damaged['ledger'][-1]['version'] = 228
                else: damaged['ledger'][0]['version'] = True
                with self.subTest(fault=fault), self.assertRaises(AssertionError): validate_native_state(damaged)

        def current_ledger_fixture(self, root):
            directory = root/'backend/crates/platform/db/migrations'
            directory.mkdir(parents=True)
            sources = {}
            for name, raw in [('0001_first.sql', b'SELECT 1;\n'),
                              ('0002_second.sql', "SELECT '근거';\n".encode('utf-8'))]:
                path = directory/name
                path.write_bytes(raw)
                sources[str(path.relative_to(root))] = hashlib.sha256(raw).hexdigest()
            # Fixed SHA384 values independently bind UTF-8 bytes and newlines.
            return sources, [
                [1, True, '24454095d8d7876ca8e842f34296ca645c83e0f7b50f048c99db4b2ff4da659267afc4ef3e994f3d74e18c741165d931'],
                [2, True, '63aadf0dbe5ca3e2eb59d412035df2229c9d99edadeb9f8c78219adec98592c2ca77294d92593506f63c1de7aec2fe1f'],
            ]

        def test_current_ledger_accepts_exact_frozen_sources_and_new_suffix(self):
            with tempfile.TemporaryDirectory(prefix='current-ledger-positive-') as directory:
                root = Path(directory)
                sources, ledger = self.current_ledger_fixture(root)
                assert_current_migration_ledger(root, ledger, sources)
                suffix = root/'backend/crates/platform/db/migrations/0003_suffix.sql'
                suffix.write_bytes(b'SELECT 3;\n')
                sources[str(suffix.relative_to(root))] = hashlib.sha256(suffix.read_bytes()).hexdigest()
                ledger.append([3, True, 'f5fd4905a1908ffc4db0c917a2728a80e580d8d7224935148266115d5b6a54c9fa9aae585dd6ed6be003e47e07f9cd3a'])
                assert_current_migration_ledger(root, ledger, sources)

        def test_current_ledger_rejects_omission_duplicate_extra_types_failure_and_checksum(self):
            import copy
            with tempfile.TemporaryDirectory(prefix='current-ledger-corruption-') as directory:
                root = Path(directory)
                sources, ledger = self.current_ledger_fixture(root)
                for fault in ('missing_first','missing_last','duplicate','extra','boolean_version',
                              'string_version','failed','integer_success','checksum','short_checksum',
                              'extra_cell','missing_cell','reordered','null','empty','wrong_row'):
                    damaged = copy.deepcopy(ledger)
                    if fault == 'missing_first': damaged.pop(0)
                    elif fault == 'missing_last': damaged.pop()
                    elif fault == 'duplicate': damaged[1] = copy.deepcopy(damaged[0])
                    elif fault == 'extra': damaged.append([3, True, ledger[1][2]])
                    elif fault == 'boolean_version': damaged[0][0] = True
                    elif fault == 'string_version': damaged[0][0] = '1'
                    elif fault == 'failed': damaged[0][1] = False
                    elif fault == 'integer_success': damaged[0][1] = 1
                    elif fault == 'checksum': damaged[0][2] = '0'*96
                    elif fault == 'short_checksum': damaged[0][2] = damaged[0][2][:-1]
                    elif fault == 'extra_cell': damaged[0].append('unexpected')
                    elif fault == 'missing_cell': damaged[0].pop()
                    elif fault == 'reordered': damaged.reverse()
                    elif fault == 'null': damaged = None
                    elif fault == 'empty': damaged = []
                    elif fault == 'wrong_row': damaged[0] = {'version': 1}
                    with self.subTest(fault=fault), self.assertRaises(AssertionError):
                        assert_current_migration_ledger(root, damaged, sources)

        def test_current_ledger_rejects_source_roster_drift_duplicates_and_nonregular_files(self):
            for fault in ('missing','extra','duplicate_version','invalid_name','changed_bytes',
                          'symlink','directory','empty','missing_frozen_source'):
                with self.subTest(fault=fault), tempfile.TemporaryDirectory(prefix='current-ledger-source-') as directory:
                    root = Path(directory)
                    sources, ledger = self.current_ledger_fixture(root)
                    original = root/'backend/crates/platform/db/migrations/0001_first.sql'
                    if fault == 'missing': original.unlink()
                    elif fault == 'extra': (original.parent/'0003_extra.sql').write_bytes(b'SELECT 3;\n')
                    elif fault in ('duplicate_version','invalid_name'):
                        extra = original.parent/('0001_duplicate.sql' if fault == 'duplicate_version' else 'invalid.sql')
                        extra.write_bytes(b'SELECT 3;\n')
                        sources[str(extra.relative_to(root))] = hashlib.sha256(extra.read_bytes()).hexdigest()
                    elif fault == 'changed_bytes': original.write_bytes(b'SELECT 0;\n')
                    elif fault == 'symlink':
                        original.unlink()
                        original.symlink_to(original.parent/'0002_second.sql')
                    elif fault == 'directory':
                        original.unlink()
                        original.mkdir()
                    elif fault == 'empty':
                        for path in original.parent.glob('*.sql'): path.unlink()
                        sources.clear()
                    elif fault == 'missing_frozen_source': sources.pop(str(original.relative_to(root)))
                    with self.assertRaises(AssertionError):
                        assert_current_migration_ledger(root, ledger, sources)

        def test_historical227_prefix_allows_suffix_but_rejects_old_source_drift(self):
            with tempfile.TemporaryDirectory(prefix='historical227-source-controls-') as directory:
                root = Path(directory)
                migrations = root/'backend/crates/platform/db/migrations'
                migrations.mkdir(parents=True)
                roster = {}
                for version in range(1,228):
                    path = migrations/f'{version:04}_oracle_control.sql'
                    path.write_bytes(b'SELECT 1;')
                    roster[str(path.relative_to(root))] = hashlib.sha256(path.read_bytes()).hexdigest()
                historical227_source_prefix(root, roster)
                (migrations/'0228_unexecuted_suffix.sql').write_bytes(b'SELECT 2;')
                historical227_source_prefix(root, roster)
                original = migrations/'0001_oracle_control.sql'
                original.write_bytes(b'SELECT 9;')
                with self.assertRaisesRegex(AssertionError, 'bytes changed'): historical227_source_prefix(root, roster)
                original.write_bytes(b'SELECT 1;')
                renamed = migrations/'0001_renamed.sql'
                original.rename(renamed)
                with self.assertRaisesRegex(AssertionError, 'renamed'): historical227_source_prefix(root, roster)
                renamed.rename(original)
                renamed.write_bytes(original.read_bytes())
                with self.assertRaisesRegex(AssertionError, 'duplicate'): historical227_source_prefix(root, roster)
                renamed.unlink()
                original.unlink()
                with self.assertRaisesRegex(AssertionError, 'incomplete'): historical227_source_prefix(root, roster)
                original.symlink_to(migrations/'0002_oracle_control.sql')
                with self.assertRaisesRegex(AssertionError, 'nonregular'): historical227_source_prefix(root, roster)

        def test_extended227_bundle_and_classifiers_are_hash_bound_without_history(self):
            from unittest.mock import patch
            with tempfile.TemporaryDirectory(prefix='extended227-frozen-controls-') as directory:
                root = Path(directory)
                for relative in (NATIVE_FINAL9_FIXTURE, NATIVE_EXTENDED227_FIXTURE,
                                  *[path for path,_ in NATIVE_EXTENDED227_CLASSIFIERS.values()]):
                    path = root/relative
                    path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(Path(__file__).parent/'fixtures'/Path(relative).name, path)
                self.assertFalse((root/'.git').exists())
                with patch.object(subprocess, 'check_output', side_effect=AssertionError('history or subprocess lookup')):
                    bundle = native_extended227_bundle(root)
                    classifiers = native_extended227_classifiers(root)
                self.assertEqual(bundle['source_base'], NATIVE_EXTENDED227_BASE)
                self.assertEqual(len(bundle['migration_sha256']), 227)
                self.assertEqual(set(bundle['files']), {'ops/'+name for name in ASSETS} | {'ops/generate-account-custody.py'})
                self.assertEqual(set(classifiers), set(NATIVE_EXTENDED227_CLASSIFIERS))
                path = root/NATIVE_EXTENDED227_FIXTURE
                original = path.read_bytes()
                path.write_bytes(original+b'changed')
                with self.assertRaisesRegex(AssertionError, 'digest mismatch'): native_extended227_bundle(root)
                path.unlink()
                with self.assertRaisesRegex(AssertionError, 'nonregular'): native_extended227_bundle(root)
                target = root/'real-extended-fixture'
                target.write_bytes(original)
                path.symlink_to(target)
                with self.assertRaisesRegex(AssertionError, 'nonregular'): native_extended227_bundle(root)
                for relative,_ in NATIVE_EXTENDED227_CLASSIFIERS.values():
                    classifier = root/relative
                    classifier.write_bytes(classifier.read_bytes()+b'changed')
                    with self.assertRaisesRegex(AssertionError, 'digest mismatch'): native_extended227_classifiers(root)
                    classifier.unlink()
                    with self.assertRaisesRegex(AssertionError, 'nonregular'): native_extended227_classifiers(root)
                    shutil.copyfile(Path(__file__).parent/'fixtures'/Path(relative).name, classifier)

        def test_native_predecessor_loads_without_git_history_or_subprocess(self):
            from unittest.mock import patch
            with tempfile.TemporaryDirectory(prefix='native-no-history-') as directory:
                root = Path(directory)
                path = root / NATIVE_FINAL9_FIXTURE
                path.parent.mkdir(parents=True)
                shutil.copyfile(Path(__file__).parent/'fixtures'/Path(NATIVE_FINAL9_FIXTURE).name, path)
                self.assertFalse((root/'.git').exists())
                with patch.object(subprocess, 'check_output', side_effect=AssertionError('history or subprocess lookup')):
                    bundle = native_predecessor_bundle(root)
                self.assertEqual(bundle['source_base'], NATIVE_FINAL9_BASE)
                self.assertEqual(len(bundle['migration_sha256']), 227)
                self.assertEqual(set(bundle['files']), {'ops/'+name for name in ASSETS} | {'ops/generate-account-custody.py'})
                self.assertEqual(hashlib.sha256(bundle['files']['ops/generate-account-custody.py'].encode()).hexdigest(),
                                 NATIVE_FINAL9_GENERATOR_SHA256)
                self.assertEqual(source_literal(bundle['files']['ops/generate-account-custody.py'], 'NATIVE_FINALIZED_SHA256'),
                                 NATIVE_FINAL9_SHA256)

        def test_native_predecessor_missing_corrupt_and_symlink_refuse(self):
            with tempfile.TemporaryDirectory(prefix='native-fixture-fault-') as directory:
                root = Path(directory)
                path = root / NATIVE_FINAL9_FIXTURE
                path.parent.mkdir(parents=True)
                with self.assertRaisesRegex(AssertionError, 'nonregular native predecessor fixture'):
                    native_predecessor_bundle(root)
                original = (Path(__file__).parent/'fixtures'/Path(NATIVE_FINAL9_FIXTURE).name).read_bytes()
                path.write_bytes(original+b'changed')
                with self.assertRaisesRegex(AssertionError, 'native predecessor fixture digest mismatch'):
                    native_predecessor_bundle(root)
                path.unlink()
                target = root/'real-fixture'
                target.write_bytes(original)
                path.symlink_to(target)
                with self.assertRaisesRegex(AssertionError, 'nonregular native predecessor fixture'):
                    native_predecessor_bundle(root)

        def test_native_raw_migration_phase_rejects_owned_or_populated_root(self):
            import copy
            from unittest.mock import patch
            before, _ = self.native_oracle_fixture()
            for row in before['data']:
                if row['name'] in TABLES:
                    row['count'] = 0; row['rows'] = []
            for row in before['metadata']['snapshot']['tables']:
                row['owner'] = 'console_app'
            self.seal_native_metadata(before)
            assert_raw_native_migration(before)
            for name in TABLES:
                damaged = copy.deepcopy(before)
                next(row for row in damaged['metadata']['snapshot']['tables'] if row['name'] == name)['owner'] = 'console_account_owner'
                self.seal_native_metadata(damaged)
                with self.subTest(table=name, fault='owner'), self.assertRaisesRegex(AssertionError, 'native_raw_migration.owner_mismatch'):
                    assert_raw_native_migration(damaged)
                damaged = copy.deepcopy(before)
                row = next(row for row in damaged['data'] if row['name'] == name)
                row['count'] = 1; row['rows'] = [{'id':'checker-only-row'}]
                with self.subTest(table=name, fault='populated'), self.assertRaisesRegex(AssertionError, 'native_raw_migration.root_not_empty'):
                    assert_raw_native_migration(damaged)
            for name in ('NATIVE_STAGED_SHA256', 'NATIVE_FINAL9_SHA256', 'NATIVE_EXTENSION_SHA256'):
                with patch.dict(globals(), {name:before['metadata']['sha256']}), self.assertRaisesRegex(AssertionError, 'native_raw_migration.already_in_custody'):
                    assert_raw_native_migration(before)

        def test_native_failure_location_omits_protected_locals(self):
            def observation_failure():
                protected_rows = 'TEST_ONLY_PRIVATE_ROW_CANARY'
                raise AssertionError()
            try:
                observation_failure()
            except AssertionError as error:
                location = safe_failure_location(error)
            self.assertRegex(location, r'^observation_failure:[1-9][0-9]*$')
            self.assertNotIn('PRIVATE_ROW', location)

    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Machinery))
    return 0 if result.wasSuccessful() else 1


def main():
    if sys.argv[1:]==['--self-test']:
        return machinery_tests()
    if sys.argv[1:]==['--list']:
        print('\n'.join(CASES))
        print(f'{len(CASES)} cases discovered; 0 executed')
        return 0
    if sys.argv[1:]:
        raise SystemExit('usage: integration.test.sh [--list|--self-test]')
    output=Path(tempfile.mkdtemp(prefix='console-account-wrapper-evidence-'))
    report={'discovered':len(CASES),'executed':0,'passed':0,'failed':0,'blocked':len(CASES),
            'status':'PREREQUISITE','cases':[],'commands':[],
            'scope':'LC02; not LC07 migration-job ordering, LC03 full shape or production exposure'}
    suite=None
    try:
        missing=[name for name in ASSETS if not (ROOT/'ops'/name).is_file()]
        if missing: raise Prerequisite('missing production assets: '+', '.join(missing))
        if not (ROOT/'backend/crates/platform/db/migrations/0226_account_terms_catalog.sql').is_file():
            raise Prerequisite('missing real additive Account migration')
        suite=Suite(output)
        suite.execute()
        suite.assert_sources_unchanged()
        report['status']='PASS'
    except Prerequisite as error:
        report['reason']=str(error)
    except Exception as error:
        report['status']='FAIL' if suite and suite.results else 'PREREQUISITE'
        report['reason']=str(error)
    finally:
        if suite:
            try:
                cleanup=suite.cleanup()
                if cleanup:
                    report['status']='FAIL'; report['cleanup_failed']=cleanup
            except Exception as error:
                report['status']='FAIL'; report['cleanup_error']=str(error)
            report['cases']=suite.results
            report['commands']=suite.commands
            report['production_provenance']=suite.provenance
        report.update(executed=len(report['cases']),passed=sum(x['status']=='PASS' for x in report['cases']),
                      failed=sum(x['status']=='FAIL' for x in report['cases']),
                      blocked=len(CASES)-len(report['cases']))
        if report['status']=='PASS' and not complete_case_roster(report['cases']):
            report['status']='FAIL';report['reason']='incomplete case roster'
        if suite and suite.frozen_sources is not None:
            try:
                suite.assert_sources_unchanged()
            except Exception as error:
                report['status']='FAIL'; report['reason']=str(error)
        report['candidate_head']=subprocess.check_output(['git','-C',str(ROOT),'rev-parse','HEAD'],text=True).strip()
        report['changed_paths']=subprocess.check_output(['git','-C',str(ROOT),'status','--porcelain=v1','-z']).decode('utf-8')
        report['source_sha256']={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
            for p in [Path(__file__),ROOT/'ops/postgres-finalize-account-custody.integration.test.sh']}
        (output/'result.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({'evidence':str(output),'status':report['status'],
                          'discovered':report['discovered'],'executed':report['executed'],
                          'passed':report['passed'],'failed':report['failed'],'blocked':report['blocked']}))
    return 0 if report['status']=='PASS' else 77 if report['status']=='PREREQUISITE' else 1


if __name__=='__main__':
    sys.exit(main())
