-- Read-only complete custody verdict: six relations and both routines. Caller must use search_path=pg_catalog,pg_temp.
-- Expected fingerprints are fixed from reviewed0226, never from this target.
WITH expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE t.tgrelid=c.oid) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c')
), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        bool_and(CASE
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS ready
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT CASE
 WHEN (SELECT count(*) FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner')
   AND NOT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit
   AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) <> 2
   OR EXISTS (SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.roleid OR r.oid=m.member
     WHERE r.rolname IN ('console_account_owner','console_terms_owner'))
 THEN 'account_custody.owner_topology_mismatch'
 WHEN EXISTS (SELECT 1 FROM relations WHERE oid IS NULL)
 THEN 'account_custody.catalog_missing'
 WHEN EXISTS (SELECT 1 FROM relations WHERE actual_shape IS DISTINCT FROM
   CASE WHEN name='account_terms_release_receipts' AND
     ((SELECT present FROM receipt_guard) OR (SELECT ready FROM acl_profiles))
     THEN 'b72be303c47b094e2291e00bc0a98345ac5cc63a8be1520373ea4cb07c90c410'
     ELSE shape_sha256 END)
   OR ((SELECT present FROM receipt_guard) AND NOT COALESCE((SELECT valid FROM guard_trigger),false))
   OR EXISTS (SELECT 1 FROM pg_class c JOIN expected e ON e.name=c.relname
     JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname<>'public')
 THEN 'account_custody.catalog_shape_mismatch'
 WHEN NOT COALESCE((SELECT pending OR finalized FROM ownership),false)
 THEN 'account_custody.owner_mismatch'
 WHEN (SELECT dormant FROM table_acl_profiles) IS NOT DISTINCT FROM (SELECT present FROM projection)
 THEN 'account_fence_projection.profile_mismatch'
 WHEN (SELECT present AND NOT valid FROM projection)
 THEN 'account_fence_projection.definition_mismatch'
 WHEN (SELECT present AND NOT valid FROM receipt_guard)
 THEN 'account_terms_receipts.definition_mismatch'
 WHEN NOT COALESCE((SELECT dormant OR prepared OR ready FROM acl_profiles),false)
   OR EXISTS (SELECT 1 FROM relations c CROSS JOIN pg_roles r
     WHERE r.rolname NOT LIKE 'pg\_%' ESCAPE '\' AND r.oid<>c.relowner AND NOT r.rolsuper
     AND (has_table_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
       OR has_any_column_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
       OR pg_has_role(r.oid,c.relowner,'MEMBER') OR pg_has_role(r.oid,c.relowner,'SET')))
 THEN 'account_custody.unexpected_privilege'
 WHEN NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='console_auth_rt'
   AND rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit
   AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication)
   OR EXISTS(SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.member OR r.oid=m.roleid
     WHERE r.rolname='console_auth_rt')
 THEN 'account_fence_projection.role_mismatch'
 WHEN (SELECT finalized FROM ownership) AND (SELECT ready FROM acl_profiles)
   AND (SELECT valid FROM projection) AND (SELECT valid FROM receipt_guard)
   AND COALESCE((SELECT valid FROM guard_trigger),false)
 THEN 'account_custody.finalized'
 -- Historical profiles are complete upgrade inputs, never serving profiles.
 WHEN NOT (SELECT present FROM receipt_guard)
   AND ((SELECT dormant FROM acl_profiles) OR
     ((SELECT prepared FROM acl_profiles) AND (SELECT valid FROM projection)))
 THEN CASE WHEN (SELECT pending FROM ownership) THEN 'account_custody.pending'
   ELSE 'account_custody.upgrade_required' END
 ELSE 'account_custody.unexpected_privilege'
END;
