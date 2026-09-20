SELECT 
            p.proargtypes='2950'::oidvector AND p.pronargs=1 AND p.pronargdefaults=0
            AND p.proargnames=ARRAY['p_account']::text[] AND p.proallargtypes IS NULL AND p.proargmodes IS NULL
            AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proleakproof
            AND p.provolatile='v' AND p.proparallel='u' AND NOT p.proretset
            AND p.prorettype='bool'::regtype AND p.proowner='console_account_owner'::regrole
            AND p.prolang=(SELECT oid FROM pg_language WHERE lanname='plpgsql')
            AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=on']::text[]
            AND (SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
                CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
                a.privilege_type,a.is_grantable)
                ORDER BY CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
                FROM aclexplode(p.proacl) a)
              = '[["console_account_owner","console_account_owner","EXECUTE",false],["console_account_owner","console_auth_rt","EXECUTE",false]]'::jsonb
          FROM pg_proc p WHERE p.pronamespace='public'::regnamespace AND p.proname='account_company_setup_eligibility_v1';
