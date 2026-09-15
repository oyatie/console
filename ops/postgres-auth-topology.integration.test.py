#!/usr/bin/env python3
"""Real disposable PostgreSQL auth-role topology controls; no pool/activation claim."""
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import time

IMAGE = 'postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'
PASSWORD_KEYS = ['POSTGRES_ADMIN_PASSWORD', 'CONSOLE_APP_POSTGRES_PASSWORD', 'CONSOLE_RT_POSTGRES_PASSWORD', 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD', 'CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD', 'CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD']
SOURCE = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name('postgres-reconcile-topology.sh')
SNAPSHOT = """SELECT jsonb_build_object(
'roles',(SELECT jsonb_agg(to_jsonb(r) ORDER BY oid) FROM pg_authid r),
'members',(SELECT coalesce(jsonb_agg(to_jsonb(m) ORDER BY roleid,member,grantor),'[]') FROM pg_auth_members m),
'settings',(SELECT coalesce(jsonb_agg(to_jsonb(s) ORDER BY setdatabase,setrole),'[]') FROM pg_db_role_setting s),
'databases',(SELECT jsonb_agg(to_jsonb(d) ORDER BY oid) FROM pg_database d),
'namespaces',(SELECT jsonb_agg(to_jsonb(n) ORDER BY oid) FROM pg_namespace n),
'default_acl',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY oid),'[]') FROM pg_default_acl a));"""


def main():
    if not os.environ.get('DOCKER_CONTEXT'):
        raise SystemExit('select explicit disposable DOCKER_CONTEXT')
    if SOURCE.is_symlink() or not SOURCE.is_file():
        raise SystemExit('production source must be regular file, not symlink')
    source_bytes = SOURCE.read_bytes()
    source_hash = hashlib.sha256(source_bytes).hexdigest()
    name = 'console-auth-topology-' + secrets.token_hex(12)
    passwords = {k: secrets.token_hex(32) for k in PASSWORD_KEYS + ['CONSOLE_AUTH_POSTGRES_PASSWORD']}
    report = {'source': str(SOURCE), 'source_sha256': source_hash, 'image': IMAGE, 'container': name, 'cases': [], 'scope': 'Real PostgreSQL role topology, password authentication and refusal atomicity; no auth pool or production activation'}

    def require(ok, label):
        if not ok:
            raise AssertionError(label)

    def run(*args, data=None, timeout=30):
        result = subprocess.run(args, input=data, text=True, capture_output=True, timeout=timeout)
        # Neither child output nor argv are emitted. Refuse secret-bearing output
        # before a failed DDL could be copied into evidence by a caller.
        require(not any(secret in result.stdout + result.stderr for secret in passwords.values()), 'secret appeared in subprocess output')
        return result

    with tempfile.TemporaryDirectory(prefix='console-auth-topology-') as tmp:
        private = Path(tmp)
        env = private / 'postgres.env'
        env.write_text('POSTGRES_USER=operator\nPOSTGRES_DB=console\nPOSTGRES_INITDB_ARGS=--auth-host=scram-sha-256\nPOSTGRES_PASSWORD=' + passwords['POSTGRES_ADMIN_PASSWORD'] + '\n')
        env.chmod(0o600)
        script = private / 'topology.sh'
        script.write_bytes(source_bytes)
        script.chmod(0o600)

        def sql(query, *, fail=False):
            result = run('docker', 'exec', '-i', name, 'psql', '-XqAt', '-v', 'ON_ERROR_STOP=1', '-U', 'operator', '-d', 'console', data=query)
            require(fail or result.returncode == 0, 'fixture SQL failed')
            return result.stdout.strip()

        def snapshot():
            return sql(SNAPSHOT)  # Sensitive role password hashes stay only in memory.

        def topology(auth=None, *, readonly=False):
            values = dict(passwords)
            values.pop('CONSOLE_AUTH_POSTGRES_PASSWORD')
            values.update(POSTGRES_HOST='127.0.0.1', POSTGRES_PORT='5432', POSTGRES_DB='console', POSTGRES_ADMIN_USER='operator')
            if auth is not None:
                values['CONSOLE_AUTH_POSTGRES_PASSWORD'] = auth
            if readonly:
                values['PGOPTIONS'] = '-c default_transaction_read_only=on'
            target = private / 'topology.env'
            target.write_text(''.join(k + '=' + v + '\n' for k, v in values.items()))
            target.chmod(0o600)
            try:
                return run('docker', 'exec', '--env-file', str(target), name, 'bash', '/tmp/topology.sh', timeout=60)
            finally:
                require(run('docker', 'logs', name).returncode == 0, 'server logs unavailable for secret audit')

        def login(role, password, query='SELECT session_user;'):
            target = private / 'login.env'
            target.write_text('PGPASSWORD=' + password + '\n')
            target.chmod(0o600)
            return run('docker', 'exec', '--env-file', str(target), name, 'psql', '-XqAt', '-w', '-h', '127.0.0.1', '-U', role, '-d', 'console', '-c', query)

        def auth_login(password):
            return login('console_auth_rt', password)

        def reject_unchanged(invoke, label):
            before = snapshot()
            result = invoke()
            require(result.returncode != 0, label + ': expected refusal')
            require(snapshot() == before, label + ': role or catalog mutated on refusal')

        def fresh():
            require(topology().returncode == 0, 'fresh topology invocation failed')
            require(sql("SELECT count(*) FROM pg_authid WHERE rolname='console_auth_rt' AND rolpassword IS NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication AND NOT rolbypassrls;") == '1', 'missing safe auth role with NULL password')
            require(auth_login(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD']).returncode != 0, 'NULL password unexpectedly authenticated via TCP password route')

        def supplied():
            require(topology(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD']).returncode == 0, 'distinct auth credential provisioning failed')
            login = auth_login(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD'])
            require(login.returncode == 0 and login.stdout.strip() == 'console_auth_rt', 'auth password login did not authenticate exact role')
            require(sql("SELECT count(*) FROM pg_authid WHERE rolname='console_auth_rt' AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication AND NOT rolbypassrls;") == '1', 'supplied credential changed safe auth flags')
            require(sql("SELECT count(*) FROM pg_auth_members WHERE roleid='console_auth_rt'::regrole OR member='console_auth_rt'::regrole;") == '0', 'auth membership exists')

        def wrong_credential():
            result = auth_login('incorrect-' + passwords['CONSOLE_AUTH_POSTGRES_PASSWORD'])
            require(result.returncode != 0 and 'password authentication failed' in result.stderr, 'wrong credential did not prove SCRAM password refusal')

        def omission():
            before = sql("SELECT rolpassword FROM pg_authid WHERE rolname='console_auth_rt';")
            require(bool(before), 'expected retained password hash')
            require(topology().returncode == 0, 'omitted repeat failed')
            require(sql("SELECT rolpassword FROM pg_authid WHERE rolname='console_auth_rt';") == before, 'omitted repeat changed retained password hash')
            require(auth_login(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD']).returncode == 0, 'omitted repeat broke auth login')

        def old_roles():
            require(sql("SELECT count(*) FROM pg_roles WHERE rolname IN ('console_app','console_rt','console_leave_cmd','console_ontology_cmd','console_platform_force_cmd','console_leave_definer','console_ontology_writer');") == '7', 'existing seven-role topology not preserved')

        cases = [('fresh_omission_null_password', fresh), ('distinct_auth_login', supplied), ('wrong_auth_credential_refused', wrong_credential), ('omitted_repeat_preserves_hash', omission), ('old_seven_roles_preserved', old_roles)]
        for key in PASSWORD_KEYS:
            cases.append(('collision_' + key, lambda key=key: reject_unchanged(lambda: topology(passwords[key]), 'secret collision')))
        for enable, disable in [('SUPERUSER', 'NOSUPERUSER'), ('INHERIT', 'NOINHERIT'), ('CREATEDB', 'NOCREATEDB'), ('CREATEROLE', 'NOCREATEROLE'), ('REPLICATION', 'NOREPLICATION'), ('BYPASSRLS', 'NOBYPASSRLS'), ('NOLOGIN', 'LOGIN')]:
            def drift(enable=enable, disable=disable):
                sql('ALTER ROLE console_auth_rt ' + enable + ';')
                try:
                    reject_unchanged(lambda: topology(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD']), 'auth flag drift')
                finally:
                    sql('ALTER ROLE console_auth_rt ' + disable + ';')
            cases.append(('flag_drift_' + enable, drift))
        for grant, revoke in [('GRANT console_auth_rt TO console_rt', 'REVOKE console_auth_rt FROM console_rt'), ('GRANT console_rt TO console_auth_rt', 'REVOKE console_rt FROM console_auth_rt')]:
            def member(grant=grant, revoke=revoke):
                sql(grant + ';')
                try:
                    if grant == 'GRANT console_auth_rt TO console_rt':
                        inherited = login('console_rt', passwords['CONSOLE_RT_POSTGRES_PASSWORD'], 'SET ROLE console_auth_rt; SELECT current_user;')
                        require(inherited.returncode == 0 and inherited.stdout.strip() == 'console_auth_rt', 'inbound grant does not exercise actual SET ROLE path')
                    reject_unchanged(lambda: topology(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD']), 'inbound/outbound auth membership drift')
                finally:
                    sql(revoke + ';')
            cases.append(('membership_' + str(len(cases)), member))
        cases.append(('forced_readonly_ddl_failure_no_secret_or_mutation', lambda: reject_unchanged(lambda: topology(passwords['CONSOLE_AUTH_POSTGRES_PASSWORD'], readonly=True), 'forced DDL refusal')))
        report['discovered'] = len(cases)
        try:
            cached = run('docker', 'image', 'inspect', '--format', '{{.Id}}', IMAGE)
            require(cached.returncode == 0, 'pinned image must already be available; fixture never pulls')
            created = run('docker', 'create', '--name', name, '--label', 'console.auth-topology.owner=' + name, '--network', 'none', '--tmpfs', '/var/lib/postgresql:rw,size=1g', '--env-file', str(env), IMAGE, 'postgres', '-c', 'log_statement=all', '-c', 'log_min_error_statement=error')
            require(created.returncode == 0, 'container create failed')
            require(run('docker', 'start', name).returncode == 0, 'container start failed')
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                ready = run('docker', 'exec', name, 'pg_isready', '-U', 'operator', '-d', 'console')
                pid1 = run('docker', 'exec', name, 'cat', '/proc/1/comm')
                if ready.returncode == 0 and pid1.stdout.strip() == 'postgres':
                    break
                time.sleep(0.2)
            else:
                raise AssertionError('PostgreSQL readiness deadline exceeded')
            require(sql("SELECT current_setting('log_statement') || '|' || current_setting('log_min_error_statement');") == 'all|error', 'fixture logging is not adversarial')
            sql("SELECT 'auth_topology_log_positive_control';")
            log_control = run('docker', 'logs', name)
            require(log_control.returncode == 0 and 'auth_topology_log_positive_control' in log_control.stdout + log_control.stderr, 'server log positive control missing')
            require(run('docker', 'cp', str(script), name + ':/tmp/topology.sh').returncode == 0, 'exact script copy failed')
            copied = run('docker', 'exec', name, 'sha256sum', '/tmp/topology.sh')
            require(copied.returncode == 0 and copied.stdout.split()[0] == source_hash, 'copied production source bytes differ')
            for label, case in cases:
                report['executed'] = report.get('executed', 0) + 1
                try:
                    case()
                except Exception:
                    report['cases'].append({'name': label, 'status': 'FAIL'})
                    raise
                report['cases'].append({'name': label, 'status': 'PASS'})
            require(hashlib.sha256(SOURCE.read_bytes()).hexdigest() == source_hash, 'production source drift')
            report['status'] = 'PASS'
        except Exception as error:
            report['status'] = 'FAIL'
            report['failure_type'] = type(error).__name__
            # Assertions are fixed labels, never SQL, secret or subprocess argv.
            report['reason'] = str(error) if isinstance(error, AssertionError) else 'fixture command failed'
        finally:
            try:
                owner = run('docker', 'inspect', '--type', 'container', '--format', '{{json .Config.Labels}}', name)
                if owner.returncode == 0:
                    require(json.loads(owner.stdout).get('console.auth-topology.owner') == name, 'refuse unrelated container cleanup')
                    try:
                        removed = run('docker', 'rm', '-fv', name)
                    finally:
                        absent = run('docker', 'inspect', '--type', 'container', name)
                        require(absent.returncode != 0 and any(line.strip() in ['Error response from daemon: No such container: ' + name, 'Error: No such container: ' + name] for line in absent.stderr.splitlines()), 'owned container absence unconfirmed')
                    require(removed.returncode == 0, 'owned container removal failed')
                else:
                    require(any(line.strip() in ['Error response from daemon: No such container: ' + name, 'Error: No such container: ' + name] for line in owner.stderr.splitlines()), 'owned container state unknown')
                report['cleanup'] = 'owned container absent'
            except Exception:
                report['status'] = 'FAIL'
                report['cleanup'] = 'owned container cleanup unconfirmed'
    report['private_directory_cleanup'] = 'confirmed'
    report['passed'] = sum(c['status'] == 'PASS' for c in report['cases'])
    report['blocked_after_first_failure'] = report['discovered'] - report.get('executed', 0)
    print(json.dumps(report, indent=2))
    return 0 if report['status'] == 'PASS' else 1


if __name__ == '__main__':
    sys.exit(main())
