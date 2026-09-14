"""LC02 wrapper acceptance. No fabricated SQLx ledger or fixture ALTER OWNER.

--list discovers cases without Docker/build prerequisites. Normal invocation
builds the current console-app and uses its actual migrate run mode. Missing
production assets are prerequisites (exit 77), never deeper behavioral RED.
Only disposable, randomly named test containers are removed by this runner.
"""
from pathlib import Path
import hashlib
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
         'release_checksum_mismatch', 'wrong_tls_ca', 'wrong_tls_hostname')
ASSETS = ('postgres-finalize-account-custody.sh', 'postgres-finalize-account-custody.sql',
          'account-custody-migrations.sha384')

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
            absent = code != 0 and any(line.strip() in (
                'Error: No such container: '+container,
                'Error: No such object: '+container) for line in text.splitlines())
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
            'CONSOLE_RT_POSTGRES_PASSWORD', 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD',
            'CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD', 'CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD')}
        self.sequence = 0
        self.provenance = {}
        self.frozen_sources = None

    def run(self, argv, *, env=None, stdin=None, required=True, timeout=240):
        self.sequence += 1
        log = self.output / f'{self.sequence:03d}.log'
        try:
            result = subprocess.run(argv, env=env or self.env, input=stdin,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout, cwd=ROOT)
        except subprocess.TimeoutExpired as error:
            data = error.stdout or b''
            log.write_bytes(self.redact(data))
            self.commands.append({'argv': argv, 'exit': 'TIMEOUT', 'log': log.name})
            raise AssertionError(f'command timed out: {argv[0]}') from error
        data = result.stdout
        leaked = any(value.encode() in data for value in self.passwords.values())
        log.write_bytes(self.redact(data))
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

    def psql(self, container, database, sql, *, role='wrapper_admin', password=None, required=True):
        password = password or self.passwords['POSTGRES_ADMIN_PASSWORD']
        envfile = self.private_file('psql.env', 'PGPASSWORD='+password+'\n')
        return self.run(['docker', 'exec', '-i', '--env-file', str(envfile), container,
                         'psql', '-X', '-w', '-h', 'localhost', '-U', role, '-d', database,
                         '-v', 'ON_ERROR_STOP=1', '-At', '-F', '|', '-c', sql], required=required)

    def snapshot(self, container, database):
        _, text = self.psql(container, database, SNAPSHOT)
        rows = json.loads(text)
        assert isinstance(rows, list) and len(rows) == 6, 'missing/empty catalog snapshot'
        assert {r['name'] for r in rows} == set(TABLES), 'incomplete snapshot roster'
        counts = []
        for table in TABLES:
            _, count = self.psql(container, database, f'SELECT count(*) FROM public.{table}')
            counts.append(int(count.strip()))
        assert counts == [0]*6, 'dormant catalog contains rows'
        # Include actual ledger data in equality without ever writing its rows.
        _, exists = self.psql(container, database, "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
        ledger = None
        if exists.strip() == 't':
            _, raw = self.psql(container, database,
                "SELECT jsonb_agg(jsonb_build_array(version,success,encode(checksum,'hex')) ORDER BY version) FROM public._sqlx_migrations")
            ledger = json.loads(raw)
            assert ledger, 'empty ledger cannot be a migrated positive'
        return {'relations': rows, 'counts': counts, 'ledger': ledger}

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

    def database(self, container, name, binary, *, raw=False):
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
            self.run([str(binary)],env=env,timeout=300)
        state = self.snapshot(container,name)
        assert all(r['owner']=='console_app' for r in state['relations']), 'fixture not staging'
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
        for key,value in changes.items():
            if value is None:
                values.pop(key,None)
            else:
                values[key]=value
        file=self.private_file('wrapper.env',''.join(k+'='+v+'\n' for k,v in values.items()))
        return self.run(['docker','exec','--env-file',str(file),container,'bash',assets+'/'+ASSETS[0]],required=False)

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
            self.results.append({'name':name,'status':'FAIL','error':str(error),'commands':[start,len(self.commands)]})
            raise
        self.results.append({'name':name,'status':'PASS','commands':[start,len(self.commands)]})

    def execute(self):
        tracked = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '-z',
            'backend', 'ops', 'rust-toolchain.toml', '.cargo']).decode().split('\0')
        paths = [name for name in tracked if name]
        paths += ['ops/'+name for name in ASSETS]
        paths += [str(path.relative_to(ROOT)) for path in (ROOT/'backend/crates/platform/db/migrations').glob('*.sql')]
        self.frozen_sources = source_snapshot(ROOT, paths)
        self.provenance['prebuild_sources'] = self.frozen_sources
        self.run(['docker','image','inspect',IMAGE])
        build_env=dict(self.env,RUSTUP_TOOLCHAIN='1.98.1',RUSTC_WRAPPER='',CARGO_INCREMENTAL='1')
        self.run(['cargo','build','--locked','--manifest-path',str(ROOT/'backend/Cargo.toml'),
                  '-p','console-app','--bin','console-app'],env=build_env,timeout=900)
        self.assert_sources_unchanged()
        target=Path(build_env.get('CARGO_TARGET_DIR',str(ROOT/'backend/target')))
        if not target.is_absolute():
            target=ROOT/target
        binary=target/'debug/console-app'
        self.provenance['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
        self.provenance['production_assets']={name:hashlib.sha256((ROOT/'ops'/name).read_bytes()).hexdigest() for name in ASSETS}
        self.provenance['migrations']={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((ROOT/'backend/crates/platform/db/migrations').glob('*.sql'))}
        self.certificate()
        first,second=self.cluster(),self.cluster()
        for container,name,raw in ((first,'wrapper_positive',False),(first,'wrapper_subject',False),
            (first,'wrapper_other',False),(second,'wrapper_subject',False),(first,'wrapper_raw',True)):
            self.database(container,name,binary,raw=raw)
        expected=self.descriptor(first,'wrapper_subject')
        positive_expected=self.descriptor(first,'wrapper_positive')
        def positive():
            code,text=self.wrapper(first,'wrapper_positive',positive_expected)
            assert code==0 and 'account_custody.finalized' in text, 'normal wrapper did not finalize'
            state=self.snapshot(first,'wrapper_positive')
            assert all(r['owner']==OWNERS[r['name']] for r in state['relations'])
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

    def cleanup(self):
        try:
            return cleanup_containers(self.containers, self.run)
        finally:
            self.temp.cleanup()


def machinery_tests():
    """Tests this evidence machinery only, not fake application implementations."""
    import unittest

    class Machinery(unittest.TestCase):
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
