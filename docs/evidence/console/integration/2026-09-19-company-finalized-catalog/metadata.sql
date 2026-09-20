SELECT jsonb_build_object(
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
  WHERE n.nspname='public' AND p.proname LIKE 'platform\_%' ESCAPE '\'),
 'owners', (SELECT jsonb_agg(jsonb_build_object('name',rolname,'login',rolcanlogin,
    'superuser',rolsuper,'bypassrls',rolbypassrls,'inherit',rolinherit))
    FROM pg_roles WHERE rolname IN ('console_app','console_rt','console_auth_startup','console_auth_rt')),
 'tables', (SELECT jsonb_agg(jsonb_build_object('name',c.relname,'owner',pg_get_userbyid(c.relowner),
    'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,'acl',c.relacl::text) ORDER BY c.relname)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public'
    AND c.relname IN ('organizations','groups','group_memberships','group_role_grants','policy_roles','user_role_assignments'))
);