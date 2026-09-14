-- Read-only six-table v1 metadata verdict. Caller must use search_path=pg_catalog.
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
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
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
 WHEN EXISTS (SELECT 1 FROM relations WHERE actual_shape IS DISTINCT FROM shape_sha256)
   OR EXISTS (SELECT 1 FROM pg_class c JOIN expected e ON e.name=c.relname
     JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname<>'public')
 THEN 'account_custody.catalog_shape_mismatch'
 WHEN NOT COALESCE((SELECT pending OR finalized FROM ownership),false)
 THEN 'account_custody.owner_mismatch'
 WHEN EXISTS (SELECT 1 FROM relations c WHERE c.relacl IS NULL OR cardinality(c.relacl)<>0)
   OR EXISTS (SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) acl)
   OR EXISTS (SELECT 1 FROM relations c CROSS JOIN pg_roles r
     WHERE r.rolname NOT LIKE 'pg\_%' ESCAPE '\' AND r.oid<>c.relowner AND NOT r.rolsuper
     AND (has_table_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
       OR has_any_column_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
       OR pg_has_role(r.oid,c.relowner,'MEMBER') OR pg_has_role(r.oid,c.relowner,'SET')))
 THEN 'account_custody.unexpected_privilege'
 WHEN (SELECT finalized FROM ownership) THEN 'account_custody.finalized'
 ELSE 'account_custody.pending'
END;
