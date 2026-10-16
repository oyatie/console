#!/usr/bin/env python3
"""Observe only metadata in the single failed, owned SQLx probe database."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
from urllib.parse import parse_qsl, unquote, urlsplit

CARGO = '/Users/jasonlee/.cargo/bin/cargo'
args = sys.argv[1:]
if '--no-run' in args or 'designated_account_discovers_mounted_company_setup_without_business_identity' not in ' '.join(args):
    os.execv(CARGO, [CARGO, *args])
result = subprocess.run([CARGO, *args])
out = Path(__file__).resolve().parent
uri = urlsplit(os.environ['DATABASE_URL'])
assert uri.hostname == '127.0.0.1' and uri.username == 'console_buck_admin'
assert parse_qsl(uri.query) == [('options[console.sqlx_test_bootstrap]', 'buck-sqlx-superuser-v1')]
env = {k: v for k, v in os.environ.items() if not k.startswith('PG')}
env.update(PGHOST=uri.hostname, PGPORT=str(uri.port), PGUSER=uri.username,
           PGPASSWORD=unquote(uri.password), PGDATABASE=unquote(uri.path[1:]),
           PGOPTIONS='-c console.sqlx_test_bootstrap=buck-sqlx-superuser-v1')

def query(sql):
    proc = subprocess.run(['/opt/homebrew/bin/psql', '-X', '-w', '-At', '-v', 'ON_ERROR_STOP=1'],
                          input=sql, env=env, capture_output=True, text=True, timeout=20)
    if proc.returncode:
        raise RuntimeError('metadata query failed; server text deliberately withheld')
    return proc.stdout

names = query("SELECT datname FROM pg_database WHERE datname LIKE '\\_sqlx\\_test\\_%' ESCAPE '\\';").splitlines()
assert len(names) == 1 and names[0].startswith('_sqlx_test_')
env['PGDATABASE'] = names[0]
sql = """SELECT jsonb_build_object(
 'server_version_num', current_setting('server_version_num'),
 'database', current_database(),
 'marked_admin', session_user='console_buck_admin' AND current_user=session_user
  AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1',
 'routines', (SELECT jsonb_agg(jsonb_build_object(
  'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
  'owner',pg_get_userbyid(p.proowner),'security_definer',p.prosecdef,
  'config',p.proconfig,'acl',p.proacl::text,
  'runtime_execute',has_function_privilege('console_rt',p.oid,'EXECUTE'),
  'startup_execute',has_function_privilege('console_auth_startup',p.oid,'EXECUTE'),
  'auth_execute',has_function_privilege('console_auth_rt',p.oid,'EXECUTE'),
  'definition',pg_get_functiondef(p.oid)) ORDER BY p.proname,pg_get_function_identity_arguments(p.oid))
  FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
  WHERE n.nspname='public' AND p.proname LIKE 'platform\\_%' ESCAPE '\\'),
 'owners', (SELECT jsonb_agg(jsonb_build_object('name',rolname,'login',rolcanlogin,
    'superuser',rolsuper,'bypassrls',rolbypassrls,'inherit',rolinherit))
    FROM pg_roles WHERE rolname IN ('console_app','console_rt','console_auth_startup','console_auth_rt')),
 'tables', (SELECT jsonb_agg(jsonb_build_object('name',c.relname,'owner',pg_get_userbyid(c.relowner),
    'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,'acl',c.relacl::text) ORDER BY c.relname)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public'
    AND c.relname IN ('organizations','groups','group_memberships','group_role_grants','policy_roles','user_role_assignments'))
);"""
(out/'metadata.sql').write_text(sql)
data = json.loads(query(sql))
assert data['marked_admin'] and data['server_version_num']=='180004'
for routine in data['routines']:
    routine['definition_sha256'] = hashlib.sha256(routine['definition'].encode()).hexdigest()
(out/'metadata.json').write_text(json.dumps(data, indent=2)+'\n')
print('COMPANY_CATALOG_METADATA_CAPTURED routines='+str(len(data['routines'])), flush=True)
sys.exit(result.returncode)
