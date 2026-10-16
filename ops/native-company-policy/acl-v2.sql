-- Exact new routine roster; activation requires independently verified custody.
DO $acl$
DECLARE routine record; grantee_name text;
BEGIN
 FOR routine IN SELECT * FROM (VALUES
  ('public.native_company_policy_codec_v2(bytea)','console_account_owner',NULL::text),
  ('public.native_company_policy_decode_v2(smallint,bytea)','console_account_owner',NULL::text),
  ('public.native_company_people_manifest_v1()','console_account_owner','console_ontology_writer'),
  ('public.native_company_policy_assert_people_catalog_v1(uuid)','console_account_owner','console_ontology_writer'),
  ('ontology_api.install_native_company_people_catalog_v1(uuid,uuid)','console_ontology_writer','console_account_owner'),
  ('public.native_company_policy_form_v2(uuid,uuid,uuid,uuid,smallint,smallint,text)','console_account_owner','console_rt'),
  ('public.native_company_policy_clause_v2(uuid,timestamptz,text)','console_account_owner',NULL::text)
 ) AS required(signature,owner_name,executor_name)
 LOOP
  EXECUTE format('ALTER FUNCTION %s OWNER TO %I',routine.signature,routine.owner_name);
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',routine.signature);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p
   CROSS JOIN LATERAL aclexplode(p.proacl) a JOIN pg_roles r ON r.oid=a.grantee
   WHERE p.oid=routine.signature::regprocedure AND r.rolname<>routine.owner_name
  LOOP
   EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',routine.signature,grantee_name);
  END LOOP;
  IF routine.executor_name IS NOT NULL THEN
   EXECUTE format('GRANT EXECUTE ON FUNCTION %s TO %I',routine.signature,routine.executor_name);
  END IF;
 END LOOP;
END
$acl$;
