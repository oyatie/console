DO $$
DECLARE name text; expected text; found text; rows bigint; role_name text;
BEGIN
 IF EXISTS(SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.member JOIN pg_roles g ON g.oid=m.roleid WHERE r.rolname IN ('console_account_owner','console_terms_owner') OR g.rolname IN ('console_account_owner','console_terms_owner')) THEN RAISE EXCEPTION 'custody membership exists'; END IF;
 IF (SELECT count(*) FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner') AND NOT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication)<>2 THEN RAISE EXCEPTION 'bad owner attributes'; END IF;
 FOREACH name IN ARRAY ARRAY['accounts','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts'] LOOP
  expected:=CASE WHEN name IN ('account_terms_head','account_terms_release_receipts') THEN 'console_terms_owner' ELSE 'console_account_owner' END;
  SELECT pg_get_userbyid(c.relowner) INTO found FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname=name;
  IF found IS DISTINCT FROM expected THEN RAISE EXCEPTION 'wrong owner %', name; END IF;
  EXECUTE format('SELECT count(*) FROM public.%I',name) INTO rows;
  IF rows<>0 THEN RAISE EXCEPTION 'unexpected rows %',name; END IF;
  FOREACH role_name IN ARRAY ARRAY['console_app','console_rt','console_leave_cmd','console_ontology_cmd','console_platform_force_cmd','console_leave_definer','console_ontology_writer'] LOOP
   IF pg_has_role(role_name,expected,'MEMBER') OR pg_has_role(role_name,expected,'SET') OR pg_has_role(role_name,expected,'USAGE') OR has_table_privilege(role_name,'public.'||name,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER') OR has_any_column_privilege(role_name,'public.'||name,'SELECT,INSERT,UPDATE,REFERENCES') THEN RAISE EXCEPTION 'custody privilege % %',role_name,name; END IF;
  END LOOP;
 END LOOP;
END $$;
