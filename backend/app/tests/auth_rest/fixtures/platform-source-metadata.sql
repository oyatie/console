SELECT jsonb_build_object(
          'relations',(SELECT jsonb_agg(jsonb_build_object('name',c.relname,'oid',c.oid,'owner',c.relowner,'acl',c.relacl,
            'rls',c.relrowsecurity,'force',c.relforcerowsecurity,
            'columns',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.attnum) FROM pg_catalog.pg_attribute a WHERE a.attrelid=c.oid),
            'defaults',(SELECT COALESCE(jsonb_agg(to_jsonb(d) ORDER BY d.oid),'[]'::jsonb) FROM pg_catalog.pg_attrdef d WHERE d.adrelid=c.oid),
            'constraints',(SELECT COALESCE(jsonb_agg(to_jsonb(k) ORDER BY k.oid),'[]'::jsonb) FROM pg_catalog.pg_constraint k WHERE k.conrelid=c.oid),
            'triggers',(SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.oid),'[]'::jsonb) FROM pg_catalog.pg_trigger t WHERE t.tgrelid=c.oid),
            'indexes',(SELECT COALESCE(jsonb_agg(to_jsonb(i) ORDER BY i.indexrelid),'[]'::jsonb) FROM pg_catalog.pg_index i WHERE i.indrelid=c.oid),
            'policies',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_policy p WHERE p.polrelid=c.oid)) ORDER BY c.relname)
            FROM pg_catalog.pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relkind IN ('r','p')),
          'functions',(SELECT COALESCE(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb) FROM pg_catalog.pg_proc p
            WHERE p.pronamespace='public'::regnamespace OR p.oid IN (SELECT tgfoid FROM pg_catalog.pg_trigger WHERE tgrelid='public.audit_events'::regclass AND NOT tgisinternal)),
          'roles',(SELECT jsonb_agg(to_jsonb(r) ORDER BY r.oid) FROM pg_catalog.pg_roles r),
          'memberships',(SELECT COALESCE(jsonb_agg(to_jsonb(m) ORDER BY m.oid),'[]'::jsonb) FROM pg_catalog.pg_auth_members m),
          'function_acls',(SELECT COALESCE(jsonb_object_agg(p.oid::text,
            (SELECT COALESCE(jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),
              CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
              a.privilege_type,a.is_grantable) ORDER BY a.grantor,a.grantee,a.privilege_type,a.is_grantable),'[]'::jsonb)
             FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a)),'{}'::jsonb)
            FROM pg_proc p WHERE p.pronamespace='public'::regnamespace
              OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid='public.audit_events'::regclass AND NOT tgisinternal)),
          'public_other_relations',(SELECT COALESCE(jsonb_agg(to_jsonb(c) ORDER BY c.oid),'[]'::jsonb)
            FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relkind NOT IN ('r','p')),
          'public_rules',(SELECT COALESCE(jsonb_agg(to_jsonb(r) ORDER BY r.oid),'[]'::jsonb)
            FROM pg_rewrite r JOIN pg_class c ON c.oid=r.ev_class WHERE c.relnamespace='public'::regnamespace),
          'public_schema',(SELECT to_jsonb(n) FROM pg_namespace n WHERE n.nspname='public'),
          'default_acls',(SELECT COALESCE(jsonb_agg(to_jsonb(a) ORDER BY a.oid),'[]'::jsonb) FROM pg_default_acl a),
          'role_settings',(SELECT COALESCE(jsonb_agg(to_jsonb(s) ORDER BY s.setdatabase,s.setrole),'[]'::jsonb) FROM pg_db_role_setting s),
          'sequences',(SELECT COALESCE(jsonb_agg(to_jsonb(s) ORDER BY s.sequencename),'[]'::jsonb) FROM pg_sequences s WHERE s.schemaname='public'),
          'ledger',(SELECT jsonb_agg(to_jsonb(m) ORDER BY version) FROM public._sqlx_migrations m),'control_system',(SELECT to_jsonb(p) FROM pg_proc p WHERE p.oid='pg_catalog.pg_control_system()'::regprocedure));
