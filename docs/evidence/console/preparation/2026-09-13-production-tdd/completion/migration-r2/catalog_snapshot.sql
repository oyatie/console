-- Read-only M0 witness. Contains no credentials, user rows or secret role hashes.
-- Execute inside one READ ONLY REPEATABLE READ transaction. Stable logical names
-- replace local OIDs; ordered key tuples are preserved rather than sorted away.
WITH owned_relations AS (
 SELECT c.*,n.nspname FROM pg_catalog.pg_class c
 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
), columns AS (
 SELECT c.nspname,c.relname,a.attname,a.attnum,
 pg_catalog.format_type(a.atttypid,a.atttypmod) AS type,
 a.attnotnull,a.attidentity,a.attgenerated,(SELECT jsonb_agg(jsonb_build_object('grantor',pg_get_userbyid(x.grantor),'grantee',CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,'privilege',x.privilege_type,'grantable',x.is_grantable) ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable) FROM aclexplode(a.attacl ) x) AS acl,
 pg_catalog.pg_get_expr(d.adbin,d.adrelid) AS default_expression
 FROM owned_relations c JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid
 LEFT JOIN pg_catalog.pg_attrdef d ON d.adrelid=c.oid AND d.adnum=a.attnum
 WHERE a.attnum>0 AND NOT a.attisdropped
), constraints AS (
 SELECT n.nspname,c.relname,k.conname,k.contype,
 pg_catalog.pg_get_constraintdef(k.oid,true) AS definition,
 k.condeferrable,k.condeferred,k.convalidated,k.confupdtype,k.confdeltype,
 (SELECT jsonb_agg(a.attname ORDER BY u.ordinality)
  FROM unnest(k.conkey) WITH ORDINALITY u(attnum,ordinality)
  JOIN pg_catalog.pg_attribute a ON a.attrelid=k.conrelid AND a.attnum=u.attnum) AS source_columns,
 rn.nspname AS referenced_schema,rc.relname AS referenced_relation,
 (SELECT jsonb_agg(a.attname ORDER BY u.ordinality)
  FROM unnest(k.confkey) WITH ORDINALITY u(attnum,ordinality)
  JOIN pg_catalog.pg_attribute a ON a.attrelid=k.confrelid AND a.attnum=u.attnum) AS referenced_columns
 FROM pg_catalog.pg_constraint k JOIN pg_catalog.pg_class c ON c.oid=k.conrelid
 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 LEFT JOIN pg_catalog.pg_class rc ON rc.oid=k.confrelid
 LEFT JOIN pg_catalog.pg_namespace rn ON rn.oid=rc.relnamespace
 WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema'
), functions AS (
 SELECT n.nspname,p.proname,pg_catalog.pg_get_function_identity_arguments(p.oid) AS arguments,
 pg_catalog.pg_get_userbyid(p.proowner) AS owner,p.prosecdef,p.proconfig,(SELECT jsonb_agg(jsonb_build_object('grantor',pg_get_userbyid(x.grantor),'grantee',CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,'privilege',x.privilege_type,'grantable',x.is_grantable) ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable) FROM aclexplode(p.proacl ) x) AS acl,
 p.provolatile,p.proparallel,p.proleakproof,
 pg_catalog.pg_get_functiondef(p.oid) AS definition
 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema' AND p.prokind IN ('f','p')
), policies AS (
 SELECT n.nspname,c.relname,p.polname,p.polcmd,p.polpermissive,
 (SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(role_oid) END)
  FROM unnest(p.polroles) role_oid) AS roles,
 pg_catalog.pg_get_expr(p.polqual,p.polrelid) AS using_expression,
 pg_catalog.pg_get_expr(p.polwithcheck,p.polrelid) AS check_expression
 FROM pg_catalog.pg_policy p JOIN pg_catalog.pg_class c ON c.oid=p.polrelid
 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema'
), triggers AS (
 SELECT c.nspname,c.relname,CASE WHEN t.tgisinternal THEN regexp_replace(t.tgname, '_[0-9]+$', '_OID') ELSE t.tgname END AS tgname,t.tgenabled,t.tgisinternal,
 CASE WHEN t.tgisinternal THEN regexp_replace(pg_catalog.pg_get_triggerdef(t.oid,true), '(RI_ConstraintTrigger_[a-z]+)_[0-9]+', '\1_OID', 'g') ELSE pg_catalog.pg_get_triggerdef(t.oid,true) END AS definition
 FROM pg_catalog.pg_trigger t JOIN owned_relations c ON c.oid=t.tgrelid
) , indexes AS (
 SELECT n.nspname,c.relname,i.relname AS index_name,pg_get_indexdef(x.indexrelid) AS definition,
 x.indisvalid,x.indisready,x.indislive,x.indisreplident
 FROM pg_index x JOIN pg_class c ON c.oid=x.indrelid JOIN pg_class i ON i.oid=x.indexrelid
 JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema'
), schemas AS (
 SELECT nspname,pg_get_userbyid(nspowner) AS owner,(SELECT jsonb_agg(jsonb_build_object('grantor',pg_get_userbyid(x.grantor),'grantee',CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,'privilege',x.privilege_type,'grantable',x.is_grantable) ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable) FROM aclexplode(nspacl ) x) AS acl FROM pg_namespace
 WHERE nspname !~ '^pg_' AND nspname<>'information_schema'
), relations AS (
 SELECT nspname,relname,relkind,pg_catalog.pg_get_userbyid(relowner) AS owner,
 relrowsecurity,relforcerowsecurity,(SELECT jsonb_agg(jsonb_build_object('grantor',pg_get_userbyid(x.grantor),'grantee',CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,'privilege',x.privilege_type,'grantable',x.is_grantable) ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable) FROM aclexplode(relacl ) x) AS acl,reloptions
 FROM owned_relations
), role_memberships AS (
 SELECT pg_catalog.pg_get_userbyid(roleid) AS role,pg_catalog.pg_get_userbyid(member) AS member,
 pg_catalog.pg_get_userbyid(grantor) AS grantor,admin_option,inherit_option,set_option
 FROM pg_catalog.pg_auth_members
), roles AS (
 SELECT rolname,rolsuper,rolinherit,rolcreaterole,rolcreatedb,rolcanlogin,rolreplication,rolbypassrls,rolconfig
 FROM pg_catalog.pg_roles
), effective_column_grants AS (
 SELECT r.rolname,c.nspname,c.relname,a.attname,p.privilege
 FROM pg_catalog.pg_roles r CROSS JOIN owned_relations c
 JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) p(privilege)
 WHERE r.rolcanlogin AND c.relkind IN ('r','p','v','m','f')
 AND pg_catalog.has_column_privilege(r.oid,c.oid,a.attnum,p.privilege)
), default_grants AS (
 SELECT pg_catalog.pg_get_userbyid(d.defaclrole) AS role,n.nspname,d.defaclobjtype,(SELECT jsonb_agg(jsonb_build_object('grantor',pg_get_userbyid(x.grantor),'grantee',CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,'privilege',x.privilege_type,'grantable',x.is_grantable) ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable) FROM aclexplode(d.defaclacl ) x) AS acl
 FROM pg_catalog.pg_default_acl d LEFT JOIN pg_catalog.pg_namespace n ON n.oid=d.defaclnamespace
)
SELECT jsonb_build_object(
 'server_version_num',current_setting('server_version_num'),
 'relations',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,relname) FROM relations r),'[]'::jsonb),
 'schemas',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname) FROM schemas r),'[]'::jsonb),
 'indexes',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,relname,index_name) FROM indexes r),'[]'::jsonb),
 'columns',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,relname,attnum) FROM columns r),'[]'::jsonb),
 'constraints',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,relname,conname) FROM constraints r),'[]'::jsonb),
 'functions',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,proname,arguments) FROM functions r),'[]'::jsonb),
 'policies',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,relname,polname) FROM policies r),'[]'::jsonb),
 'triggers',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY nspname,relname,tgname,definition,tgenabled,tgisinternal) FROM triggers r),'[]'::jsonb),
 'roles',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY rolname) FROM roles r),'[]'::jsonb),
 'memberships',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY role,member,grantor) FROM role_memberships r),'[]'::jsonb),
 'effective_column_grants',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY rolname,nspname,relname,attname,privilege) FROM effective_column_grants r),'[]'::jsonb),
 'default_grants',COALESCE((SELECT jsonb_agg(to_jsonb(r) ORDER BY role,nspname,defaclobjtype) FROM default_grants r),'[]'::jsonb)
);
