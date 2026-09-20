-- Original approved round4 DO oracle, unchanged. Transaction bounds owned by SQLx caller.
DO $tests$
DECLARE relation_name text; role_name text; target oid; routine oid; actual_owner text; privileges text;
BEGIN
  FOREACH relation_name IN ARRAY ARRAY['company_enrollment_requests','company_enrollment_receipts','company_enrollment_request_events'] LOOP
    target:=to_regclass('public.'||relation_name);
    IF target IS NULL THEN RAISE EXCEPTION 'company.schema_missing: %',relation_name; END IF;
    SELECT pg_get_userbyid(c.relowner) INTO STRICT actual_owner FROM pg_class c WHERE c.oid=target;
    IF actual_owner<>'console_account_owner' THEN RAISE EXCEPTION 'company.schema_owner: %',relation_name; END IF;
    IF EXISTS(SELECT 1 FROM pg_class c CROSS JOIN LATERAL aclexplode(coalesce(c.relacl,acldefault('r',c.relowner))) a WHERE c.oid=target AND a.grantee=0) THEN
      RAISE EXCEPTION 'company.schema_public_acl: %',relation_name;
    END IF;
    -- Exact direct table and column ACLs, including owner, unknown grantees,
    -- grantor and grant-option; known-role effective checks follow separately.
    IF EXISTS (
      WITH expected AS (
        SELECT 'table'::text AS scope, ''::text AS column_name,
               'console_account_owner'::text AS grantor,
               'console_account_owner'::text AS grantee,
               privilege_type, false AS grantable
        FROM (VALUES ('SELECT'::text),('INSERT'::text)) t(privilege_type)
        UNION ALL
        SELECT 'column', column_name, 'console_account_owner', 'console_account_owner', 'UPDATE', false
        FROM (VALUES
          ('company_enrollment_requests'::text,'input_bytes'::text),
          ('company_enrollment_requests','state'),
          ('company_enrollment_requests','terminal_at'),
          ('company_enrollment_requests','committed_receipt_id'),
          ('company_enrollment_receipts','receipt_id')) c(relation_name_,column_name)
        WHERE c.relation_name_=relation_name
      ), actual AS (
        SELECT 'table'::text AS scope, ''::text AS column_name,
               pg_get_userbyid(a.grantor)::text AS grantor,
               CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee)::text END AS grantee,
               a.privilege_type, a.is_grantable AS grantable
        FROM pg_class c CROSS JOIN LATERAL aclexplode(coalesce(c.relacl,acldefault('r',c.relowner))) a
        WHERE c.oid=target
        UNION ALL
        SELECT 'column',att.attname::text,pg_get_userbyid(a.grantor)::text,
               CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee)::text END,
               a.privilege_type,a.is_grantable
        FROM pg_attribute att CROSS JOIN LATERAL aclexplode(att.attacl) a
        WHERE att.attrelid=target AND att.attnum>0 AND NOT att.attisdropped
      )
      (SELECT * FROM actual EXCEPT SELECT * FROM expected)
      UNION ALL
      (SELECT * FROM expected EXCEPT SELECT * FROM actual)
    ) THEN RAISE EXCEPTION 'company.schema_exact_acl: %',relation_name; END IF;
    IF relation_name='company_enrollment_receipts' AND NOT EXISTS(
      SELECT 1 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid
      JOIN pg_roles r ON r.oid=p.proowner
      WHERE t.tgrelid=target AND NOT t.tgisinternal
        AND t.tgname='company_enrollment_receipts_immutable_v1'
        AND t.tgenabled='A' AND t.tgtype=58
        AND t.tgqual IS NULL AND t.tgnargs=0 AND t.tgattr=''::int2vector
        AND p.oid=to_regprocedure('public.company_enrollment_receipts_immutable_v1()')
        AND r.rolname='console_account_owner' AND NOT p.prosecdef
    ) THEN RAISE EXCEPTION 'company.receipt_keyshare_guard_missing'; END IF;
    FOREACH role_name IN ARRAY ARRAY['console_rt','console_auth_rt','console_auth_startup','console_leave_cmd','console_ontology_cmd','console_platform_force_cmd','console_app','console_terms_owner','console_credential_owner'] LOOP
      IF NOT EXISTS(SELECT 1 FROM pg_roles WHERE rolname=role_name) THEN RAISE EXCEPTION 'company.role_prerequisite_missing: %',role_name; END IF;
      FOREACH privileges IN ARRAY ARRAY['SELECT','INSERT','UPDATE','DELETE','TRUNCATE','REFERENCES','TRIGGER'] LOOP
        IF has_table_privilege(role_name,target,privileges) THEN RAISE EXCEPTION 'company.schema_serving_privilege: %/%/%',relation_name,role_name,privileges; END IF;
      END LOOP;
      FOREACH privileges IN ARRAY ARRAY['SELECT','INSERT','UPDATE','REFERENCES'] LOOP
        IF has_any_column_privilege(role_name,target,privileges) THEN RAISE EXCEPTION 'company.schema_column_privilege: %/%/%',relation_name,role_name,privileges; END IF;
      END LOOP;
    END LOOP;
  END LOOP;
  routine:=to_regprocedure('public.company_enrollment_decode_input_v1(bytea)');
  IF routine IS NULL THEN RAISE EXCEPTION 'company.parser_missing'; END IF;
  IF NOT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_roles r ON r.oid=p.proowner WHERE p.oid=routine AND r.rolname='console_account_owner'
       AND NOT p.prosecdef AND p.provolatile='i' AND NOT p.proisstrict
       AND p.proconfig @> ARRAY['search_path=pg_catalog, pg_temp']) THEN RAISE EXCEPTION 'company.parser_metadata'; END IF;
  IF EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a WHERE p.oid=routine AND a.grantee=0) THEN RAISE EXCEPTION 'company.parser_public_execute'; END IF;
  IF (SELECT count(*) FROM pg_proc p CROSS JOIN LATERAL aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a WHERE p.oid=routine)<>1
     OR EXISTS(SELECT 1 FROM pg_proc p CROSS JOIN LATERAL aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a
       WHERE p.oid=routine AND (a.grantor<>p.proowner OR a.grantee<>p.proowner OR a.privilege_type<>'EXECUTE' OR a.is_grantable))
  THEN RAISE EXCEPTION 'company.parser_exact_acl'; END IF;
  FOREACH role_name IN ARRAY ARRAY['console_rt','console_auth_rt','console_auth_startup','console_leave_cmd','console_ontology_cmd','console_platform_force_cmd','console_app','console_terms_owner','console_credential_owner'] LOOP
    IF has_function_privilege(role_name,routine,'EXECUTE') THEN RAISE EXCEPTION 'company.parser_serving_execute: %',role_name; END IF;
  END LOOP;
  RAISE NOTICE 'company.schema_acl relation_count=3 parser_count=1';
END
$tests$;
