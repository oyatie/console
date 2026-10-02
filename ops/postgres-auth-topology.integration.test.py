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
    passwords = {k: secrets.token_hex(32) for k in PASSWORD_KEYS + ['CONSOLE_AUTH_POSTGRES_PASSWORD', 'CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD']}
    report = {'source': str(SOURCE), 'source_sha256': source_hash, 'image': IMAGE, 'container': name, 'cases': [], 'scope': 'Real PostgreSQL auth/startup role topology, password authentication, timeout defaults, session drain and refusal atomicity; no serving pool or production activation'}

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

        def topology(auth=None, *, startup=None, readonly=False, require_canonical=False):
            values = dict(passwords)
            values.pop('CONSOLE_AUTH_POSTGRES_PASSWORD')
            values.pop('CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD')
            values.update(POSTGRES_HOST='127.0.0.1', POSTGRES_PORT='5432', POSTGRES_DB='console', POSTGRES_ADMIN_USER='operator')
            if auth is not None:
                values['CONSOLE_AUTH_POSTGRES_PASSWORD'] = auth
            if startup is not None:
                values['CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD'] = startup
            if require_canonical:
                values['CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES'] = '1'
            if readonly:
                values['PGOPTIONS'] = '-c default_transaction_read_only=on'
            target = private / 'topology.env'
            target.write_text(''.join(k + '=' + v + '\n' for k, v in values.items()))
            target.chmod(0o600)
            try:
                return run('docker', 'exec', '--env-file', str(target), name, 'bash', '/tmp/topology.sh', timeout=60)
            finally:
                require(run('docker', 'logs', name).returncode == 0, 'server logs unavailable for secret audit')

        def login(role, password, query='SELECT session_user;', *, database='console'):
            target = private / 'login.env'
            target.write_text('PGPASSWORD=' + password + '\n')
            target.chmod(0o600)
            return run('docker', 'exec', '--env-file', str(target), name, 'psql', '-XqAt', '-w', '-h', '127.0.0.1', '-U', role, '-d', database, '-c', query)

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

        # Preserve the preceding 21 Auth controls exactly. Startup has its own real
        # topology producer; no positive fixture CREATE ROLE, GRANT or SET ROLE.
        startup_role = 'console_auth_startup'
        startup_password = passwords['CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD']
        auth_password = passwords['CONSOLE_AUTH_POSTGRES_PASSWORD']

        def startup_fresh():
            require(topology().returncode == 0, 'startup omitted topology failed')
            require(sql("SELECT count(*) FROM pg_authid WHERE rolname='console_auth_startup' AND rolpassword IS NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication AND NOT rolbypassrls;") == '1', 'missing safe startup role with NULL password')
            require(sql("SELECT count(*) FROM pg_auth_members WHERE roleid='console_auth_startup'::regrole OR member='console_auth_startup'::regrole;") == '0', 'startup role has membership')
            refused = login(startup_role, startup_password)
            require(refused.returncode != 0 and 'password authentication failed' in refused.stderr, 'NULL startup password did not refuse actual TCP SCRAM login')

        def startup_supplied():
            require(topology(auth_password, startup=startup_password).returncode == 0, 'startup distinct credential provisioning failed')
            result = login(startup_role, startup_password, 'SELECT session_user,current_user;')
            require(result.returncode == 0 and result.stdout.strip() == 'console_auth_startup|console_auth_startup', 'startup actual LOGIN identity differs')
            require(sql("SELECT count(*) FROM pg_authid WHERE rolname='console_auth_startup' AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication AND NOT rolbypassrls;") == '1', 'startup supplied credential changed safe flags')
            require(sql("SELECT count(*) FROM pg_auth_members WHERE roleid='console_auth_startup'::regrole OR member='console_auth_startup'::regrole;") == '0', 'startup supplied credential created membership')

        def startup_wrong_credential():
            refused = login(startup_role, 'incorrect-' + startup_password)
            require(refused.returncode != 0 and 'password authentication failed' in refused.stderr, 'startup wrong credential did not refuse actual SCRAM login')

        def startup_omission():
            before = sql("SELECT rolpassword FROM pg_authid WHERE rolname='console_auth_startup';")
            require(bool(before), 'startup retained password hash missing')
            require(topology().returncode == 0, 'startup omitted repeat failed')
            require(sql("SELECT rolpassword FROM pg_authid WHERE rolname='console_auth_startup';") == before, 'startup omission changed retained password hash')
            require(login(startup_role, startup_password).returncode == 0, 'startup omission broke actual LOGIN')

        cases.extend([
            ('startup_fresh_omission_null_password', startup_fresh),
            ('startup_distinct_login', startup_supplied),
            ('startup_wrong_credential_refused', startup_wrong_credential),
            ('startup_omitted_repeat_preserves_hash', startup_omission),
            ('startup_explicit_empty_refused', lambda: reject_unchanged(lambda: topology(auth_password, startup=''), 'empty startup secret')),
        ])
        for key in PASSWORD_KEYS + ['CONSOLE_AUTH_POSTGRES_PASSWORD']:
            cases.append(('startup_collision_' + key, lambda key=key: reject_unchanged(lambda: topology(auth_password, startup=passwords[key]), 'startup secret collision')))
        for enable, disable in [('SUPERUSER', 'NOSUPERUSER'), ('INHERIT', 'NOINHERIT'), ('CREATEDB', 'NOCREATEDB'), ('CREATEROLE', 'NOCREATEROLE'), ('REPLICATION', 'NOREPLICATION'), ('BYPASSRLS', 'NOBYPASSRLS'), ('NOLOGIN', 'LOGIN')]:
            def startup_drift(enable=enable, disable=disable):
                sql('ALTER ROLE console_auth_startup ' + enable + ';')
                try:
                    reject_unchanged(lambda: topology(auth_password, startup=startup_password), 'startup flag drift')
                finally:
                    sql('ALTER ROLE console_auth_startup ' + disable + ';')
            cases.append(('startup_flag_drift_' + enable, startup_drift))
        for peer, peer_password in [('console_rt', passwords['CONSOLE_RT_POSTGRES_PASSWORD']), ('console_auth_rt', auth_password)]:
            for inbound in [True, False]:
                def startup_member(peer=peer, peer_password=peer_password, inbound=inbound):
                    granted, member = (startup_role, peer) if inbound else (peer, startup_role)
                    member_password = peer_password if inbound else startup_password
                    sql('GRANT ' + granted + ' TO ' + member + ';')
                    try:
                        # Prove the planted edge reaches the forbidden role before
                        # expecting topology refusal, including the NOINHERIT case.
                        inherited = login(member, member_password, 'SET ROLE ' + granted + '; SELECT current_user;')
                        require(inherited.returncode == 0 and inherited.stdout.strip() == granted, 'startup membership positive control did not reach planted role')
                        reject_unchanged(lambda: topology(auth_password, startup=startup_password), 'startup inbound/outbound membership drift')
                    finally:
                        sql('REVOKE ' + granted + ' FROM ' + member + ';')
                cases.append(('startup_membership_' + peer + ('_inbound' if inbound else '_outbound'), startup_member))

        def startup_defaults():
            query = "SELECT current_setting('statement_timeout'),current_setting('idle_in_transaction_session_timeout'),current_setting('transaction_timeout'),current_setting('lock_timeout');"
            sql('CREATE DATABASE startup_topology_overrides;')
            try:
                sql("ALTER ROLE console_auth_startup SET lock_timeout='1234ms'; ALTER ROLE console_auth_startup SET statement_timeout='0'; ALTER ROLE console_auth_startup SET idle_in_transaction_session_timeout='0'; ALTER ROLE console_auth_startup SET transaction_timeout='0';")
                for database in ['console', 'startup_topology_overrides']:
                    sql('ALTER ROLE console_auth_startup IN DATABASE ' + database + " SET statement_timeout='1ms'; ALTER ROLE console_auth_startup IN DATABASE " + database + " SET idle_in_transaction_session_timeout='1ms'; ALTER ROLE console_auth_startup IN DATABASE " + database + " SET transaction_timeout='1ms'; ALTER ROLE console_auth_startup IN DATABASE " + database + " SET lock_timeout='2345ms';")
                # Exact catalog control avoids a 1ms LOGIN racing its own query.
                require(sql("SELECT count(*) FROM pg_db_role_setting s CROSS JOIN LATERAL unnest(s.setconfig) setting WHERE s.setrole='console_auth_startup'::regrole AND s.setdatabase<>0 AND split_part(setting,'=',1) IN ('statement_timeout','idle_in_transaction_session_timeout','transaction_timeout');") == '6', 'startup timeout override positive control missing')
                require(topology(auth_password, startup=startup_password).returncode == 0, 'startup timeout default reconciliation failed')
                require(sql("SELECT count(*) FROM pg_db_role_setting s CROSS JOIN LATERAL unnest(s.setconfig) setting WHERE s.setrole='console_auth_startup'::regrole AND s.setdatabase<>0 AND split_part(setting,'=',1) IN ('statement_timeout','idle_in_transaction_session_timeout','transaction_timeout');") == '0', 'startup database timeout override survived')
                for database in ['console', 'startup_topology_overrides']:
                    result = login(startup_role, startup_password, query, database=database)
                    require(result.returncode == 0 and result.stdout.strip() == '30s|30s|45s|2345ms', 'startup actual LOGIN defaults or unrelated database setting changed')
                require(sql("SELECT count(*) FROM pg_db_role_setting WHERE setrole='console_auth_startup'::regrole AND setdatabase=0 AND 'lock_timeout=1234ms'=ANY(setconfig);") == '1', 'startup unrelated global setting changed')
            finally:
                sql('ALTER ROLE console_auth_startup RESET lock_timeout; ALTER ROLE console_auth_startup IN DATABASE console RESET lock_timeout;')
                sql('DROP DATABASE startup_topology_overrides WITH (FORCE);')

        def startup_session_drain():
            envfile = private / 'startup-drain.env'
            envfile.write_text('PGPASSWORD=' + startup_password + '\n')
            envfile.chmod(0o600)
            # Intentionally leave an old session with stale/unbounded settings.
            # Source must terminate this exact active PID after its commit.
            child = subprocess.Popen(['docker', 'exec', '--env-file', str(envfile), name, 'psql', '-XqAt', '-w', '-h', '127.0.0.1', '-U', startup_role, '-d', 'console', '-c', "SET application_name='startup_topology_drain'; SET statement_timeout=0; SET transaction_timeout=0; SELECT pg_sleep(120);"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            try:
                deadline = time.monotonic() + 10
                pid = ''
                while time.monotonic() < deadline:
                    pid = sql("SELECT pid FROM pg_stat_activity WHERE usename='console_auth_startup' AND application_name='startup_topology_drain' AND wait_event='PgSleep';")
                    if pid:
                        break
                    require(child.poll() is None, 'startup drain positive-control process exited early')
                    time.sleep(0.05)
                require(pid.isdigit(), 'startup drain positive-control active PID missing')
                require(topology(auth_password, startup=startup_password).returncode == 0, 'startup drain topology failed')
                require(sql('SELECT count(*) FROM pg_stat_activity WHERE pid=' + pid + ';') == '0', 'startup exact captured PID survived topology')
                stdout, stderr = child.communicate(timeout=10)
                require(not any(secret in stdout + stderr for secret in passwords.values()), 'startup drain child emitted secret')
                require(child.returncode != 0 and 'terminating connection due to administrator command' in stderr, 'startup connection did not observe administrator termination')
            finally:
                if child.poll() is None:
                    child.terminate()
                    try:
                        stdout, stderr = child.communicate(timeout=5)
                    except subprocess.TimeoutExpired:
                        child.kill()
                        stdout, stderr = child.communicate(timeout=5)
                else:
                    stdout, stderr = child.communicate(timeout=5)
                require(not any(secret in stdout + stderr for secret in passwords.values()), 'startup drain cleanup emitted secret')
                if pid.isdigit():
                    sql("SELECT pg_terminate_backend(pid,5000) FROM pg_stat_activity WHERE pid=" + pid + " AND usename='console_auth_startup' AND application_name='startup_topology_drain';")
                envfile.unlink(missing_ok=True)

        def startup_late_refusal():
            # No canonical tables exist in this owned topology-only database.
            # Their required census fails after role/password work, proving rollback.
            before = snapshot()
            refused = topology(auth_password, startup=startup_password, require_canonical=True)
            require(refused.returncode != 0, 'startup late census did not refuse')
            require('ERROR:  topology.canonical_enforcement_examined_no_tables:' in refused.stderr, 'startup refusal did not reach post-DDL canonical census')
            require(snapshot() == before, 'startup late census refusal mutated role or catalog')
            require(login(startup_role, startup_password).returncode == 0, 'startup late refusal broke retained credential')

        cases.extend([
            ('startup_timeout_overrides_and_unrelated_settings', startup_defaults),
            ('startup_exact_active_backend_drained', startup_session_drain),
            ('startup_forced_readonly_no_secret_or_mutation', lambda: reject_unchanged(lambda: topology(auth_password, startup=startup_password, readonly=True), 'startup forced DDL refusal')),
            ('startup_late_refusal_atomicity', startup_late_refusal),
        ])

        # Native custody captures the schema owner and every ACL grantor. Exercise
        # the shared operator writer, rather than repairing only a SQLx fixture.
        def public_metadata():
            return sql("SELECT jsonb_build_object('database_owner',pg_get_userbyid(d.datdba),'schema_owner',pg_get_userbyid(n.nspowner),'namespace',to_jsonb(n),'default_acl',(SELECT coalesce(jsonb_agg(to_jsonb(a) ORDER BY a.oid),'[]'::jsonb) FROM pg_default_acl a)) FROM pg_namespace n CROSS JOIN pg_database d WHERE n.nspname='public' AND d.datname=current_database();")

        def native_public_owner():
            require(topology(auth_password, startup=startup_password).returncode == 0, 'native public topology invocation failed')
            metadata = json.loads(public_metadata())
            require(metadata['database_owner'] == 'console_app', 'native database owner differs')
            require(metadata['schema_owner'] in ['console_app', 'pg_database_owner'], 'unexpected native public schema owner')
            require(metadata['schema_owner'] == 'pg_database_owner', 'topology.public_schema_native_owner: actual console_app; required pg_database_owner')
            require(sql("SELECT count(*) FROM pg_auth_members WHERE roleid='pg_database_owner'::regrole OR member='pg_database_owner'::regrole;") == '0', 'builtin owner has explicit membership')

        def native_public_login_rights():
            owner = login('console_app', passwords['CONSOLE_APP_POSTGRES_PASSWORD'], "SELECT session_user,current_user,pg_has_role(session_user,'pg_database_owner','MEMBER'),has_schema_privilege(session_user,'public','USAGE'),has_schema_privilege(session_user,'public','CREATE');")
            require(owner.returncode == 0 and owner.stdout.strip() == 'console_app|console_app|t|t|t', 'native owner actual LOGIN rights differ')
            for role, key in [('console_rt', 'CONSOLE_RT_POSTGRES_PASSWORD'), ('console_leave_cmd', 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD'), ('console_ontology_cmd', 'CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD'), ('console_platform_force_cmd', 'CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD'), ('console_auth_rt', 'CONSOLE_AUTH_POSTGRES_PASSWORD'), ('console_auth_startup', 'CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD')]:
                rights = login(role, passwords[key], "SELECT session_user,current_user,pg_has_role(session_user,'pg_database_owner','MEMBER'),has_schema_privilege(session_user,'public','CREATE');")
                require(rights.returncode == 0 and rights.stdout.strip() == role + '|' + role + '|f|f', 'nonowner actual LOGIN acquired schema authority')
                refused = login(role, passwords[key], 'CREATE TABLE public.native_topology_unauthorized (id integer);')
                require(refused.returncode != 0 and 'permission denied for schema public' in refused.stderr, 'nonowner actual CREATE was not denied at schema boundary')
            require(sql("SELECT to_regclass('public.native_topology_unauthorized') IS NULL;") == 't', 'denied CREATE left an object')

        def native_public_repeat():
            before = public_metadata()
            require(topology(auth_password, startup=startup_password).returncode == 0, 'native public repeat failed')
            require(public_metadata() == before, 'native public repeat changed raw namespace/default ACL')

        def native_public_other_database():
            sql('CREATE DATABASE native_public_other_owner;')
            try:
                rights = login('console_app', passwords['CONSOLE_APP_POSTGRES_PASSWORD'], "SELECT session_user,current_user,pg_has_role(session_user,'pg_database_owner','MEMBER'),has_schema_privilege(session_user,'public','CREATE');", database='native_public_other_owner')
                require(rights.returncode == 0 and rights.stdout.strip() == 'console_app|console_app|f|f', 'native owner gained another database owner authority')
                refused = login('console_app', passwords['CONSOLE_APP_POSTGRES_PASSWORD'], 'CREATE TABLE public.native_topology_unauthorized (id integer);', database='native_public_other_owner')
                require(refused.returncode != 0 and 'permission denied for schema public' in refused.stderr, 'other database actual CREATE was not denied')
                require(sql("SELECT pg_get_userbyid(datdba) FROM pg_database WHERE datname='native_public_other_owner';") == 'operator', 'other database owner changed')
            finally:
                sql('DROP DATABASE native_public_other_owner WITH (FORCE);')

        def native_public_transition_refusal():
            # Inject catalog drift only in this owned topology fixture. The late
            # census must roll back the proposed schema-owner transition too.
            sql('ALTER SCHEMA public OWNER TO console_app;')
            try:
                before = snapshot()
                refused = topology(auth_password, startup=startup_password, require_canonical=True)
                require(refused.returncode != 0 and 'ERROR:  topology.canonical_enforcement_examined_no_tables:' in refused.stderr, 'native owner transition did not reach late canonical census refusal')
                require(snapshot() == before, 'native owner transition refusal changed raw roles/namespaces/default ACL')
            finally:
                sql('ALTER SCHEMA public OWNER TO pg_database_owner;')

        cases.extend([
            ('native_public_schema_owner_matches_frozen_custody', native_public_owner),
            ('native_public_actual_owner_and_nonowner_login_rights', native_public_login_rights),
            ('native_public_reconcile_is_metadata_idempotent', native_public_repeat),
            ('native_public_owner_is_database_local', native_public_other_database),
            ('native_public_owner_transition_late_refusal_atomicity', native_public_transition_refusal),
        ])

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
