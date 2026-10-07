-- UNINSTALLED exact routine ACL source; no role/table/schema privilege change.
ALTER FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid) OWNER TO console_app;
ALTER FUNCTION public.identity_company_information_selected_lock_v1(uuid,uuid) OWNER TO console_app;
ALTER FUNCTION public.identity_company_information_root_material_v1(uuid,uuid) OWNER TO console_account_owner;
ALTER FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) OWNER TO console_account_owner;
DO $acl$
DECLARE routine regprocedure; grantee_name text;
BEGIN
 FOREACH routine IN ARRAY ARRAY[
  'public.identity_company_information_group_lock_v1(uuid,uuid)'::regprocedure,
  'public.identity_company_information_selected_lock_v1(uuid,uuid)'::regprocedure,
  'public.identity_company_information_root_material_v1(uuid,uuid)'::regprocedure,
  'public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)'::regprocedure] LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',routine);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p
   CROSS JOIN LATERAL aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a
   JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid=routine LOOP
   EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',routine,grantee_name);
  END LOOP;
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid),
 public.identity_company_information_selected_lock_v1(uuid,uuid),
 public.identity_company_information_root_material_v1(uuid,uuid) TO console_account_owner;
GRANT EXECUTE ON FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)
 TO console_account_owner,console_rt;
