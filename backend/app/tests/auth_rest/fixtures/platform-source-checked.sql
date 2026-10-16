SELECT
 p.proargtypes='2950 2950'::oidvector AND p.pronargs=2 AND p.pronargdefaults=0
 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.prosupport=0 AND p.protrftypes IS NULL
 AND p.proargnames=ARRAY['p_subject','p_family','roles','family_id','family_user_id','family_org_id',
     'family_protocol','family_created_at','family_revoked_at','family_account_security_generation','family_auth_time','family_assurance']::text[]
 AND p.proallargtypes=ARRAY[2950,2950,1009,2950,2950,2950,25,1184,1184,20,1184,25]::oid[]
 AND p.proargmodes=ARRAY['i','i','t','t','t','t','t','t','t','t','t','t']::"char"[]
 AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof
 AND p.provolatile='v' AND p.proparallel='u' AND p.proretset AND p.prorettype='record'::regtype
 AND p.proowner='console_credential_owner'::regrole
 AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
 AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=on']::text[]
 AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
      CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable)
      ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
      FROM aclexplode(p.proacl) a)
     = '[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]'::jsonb
 AND has_function_privilege('console_rt',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_app',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_auth_rt',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_auth_startup',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_account_owner',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_terms_owner',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_leave_cmd',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_ontology_cmd',p.oid,'EXECUTE')
 AND NOT has_function_privilege('console_platform_force_cmd',p.oid,'EXECUTE')
 FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname='auth_legacy_platform_source_material_v1';
