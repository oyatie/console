#!/usr/bin/env python3
"""Freeze the shared metadata query into the standalone operator statement.

No runtime SQL inputs or database-derived expectations. --check verifies bytes
without writing. Numbered migration checksums match SQLx 0.9's SHA-384 format.
"""
from pathlib import Path
import hashlib
import sys

ROOT = Path(__file__).resolve().parents[1]
TABLES = ('accounts', 'account_security', 'account_security_events',
          'account_terms_acceptances', 'account_terms_head', 'account_terms_release_receipts')


FENCE_BODY = """BEGIN
    IF subject_account_id IS NULL THEN
        RAISE EXCEPTION USING MESSAGE='account_fence_projection.null_identity', ERRCODE='22004';
    END IF;
    RETURN EXISTS(SELECT 1 FROM public.account_security WHERE account_id=subject_account_id);
END;"""

# Installed only by the existing operator transaction. This is a presence
# projection, never credential or lifecycle write authority. Valid replay is
# read-only; any incompatible existing routine is drift, not repair input.
GUARD_BODY = """BEGIN
    RAISE EXCEPTION USING MESSAGE='account_terms_receipts.immutable', ERRCODE='P0001';
END;"""

CURRENT_BODY = 'SELECT h.manifest_sha256,h.revision FROM public.account_terms_head AS h WHERE h.id=1'

ROOT_GUARD_BODY = """BEGIN
    RAISE EXCEPTION USING MESSAGE='account_roots.immutable', ERRCODE='P0001';
END;"""

ROOT_BRIDGE_BODY = """BEGIN
    INSERT INTO public.accounts(id,created_at) VALUES(NEW.id,NEW.created_at);
    RETURN NEW;
END;"""

USER_ID_GUARD_BODY = """BEGIN
    IF NEW.id IS DISTINCT FROM OLD.id THEN
        RAISE EXCEPTION USING MESSAGE='account_legacy_user_id.immutable', ERRCODE='P0001';
    END IF;
    RETURN NEW;
END;"""

DEACTIVATION_GUARD_BODY = """DECLARE
    fenced boolean;
BEGIN
    IF pg_catalog.current_setting('transaction_isolation') <> 'read committed' THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.unsupported_isolation', ERRCODE='P0001';
    END IF;
    IF company_id IS NULL OR subject_id IS NULL THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.null_identity', ERRCODE='22004';
    END IF;
    IF company_id IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.company_context_mismatch', ERRCODE='42501';
    END IF;
    -- Keep Company RLS active and hold both locks in the caller's transaction.
    PERFORM u.id FROM public.users u
        WHERE u.id=subject_id AND u.org_id=company_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.subject_not_found', ERRCODE='P0002';
    END IF;
    PERFORM a.id FROM public.accounts a WHERE a.id=subject_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.missing_root', ERRCODE='P0001';
    END IF;
    -- Separate SPI statement: READ COMMITTED takes a fresh snapshot after the
    -- root wait. The strict Auth projection refuses hidden authority as well.
    SELECT public.account_legacy_fenced_v1(subject_id) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.invalid_fence', ERRCODE='P0001';
    END IF;
    RETURN NOT fenced;
END;"""

DEACTIVATION_INSTALL = """
    -- Only a completely absent guard and users ACL reach this narrow upgrade.
    GRANT SELECT(id,org_id), UPDATE(id) ON public.users TO console_account_owner;
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_company_deactivation_guard_v1(company_id uuid,subject_id uuid)
        RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
        SET search_path=pg_catalog,pg_temp AS %L', expected_deactivation_guard_body);
    ALTER FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) FROM PUBLIC;
    GRANT EXECUTE ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) TO console_rt;
"""

INSTALL = """
    IF NOT guard_present THEN
        EXECUTE pg_catalog.format('CREATE FUNCTION public.account_terms_receipts_immutable_v1()
            RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER
            SET search_path=pg_catalog,pg_temp AS %L', expected_guard_body);
        ALTER FUNCTION public.account_terms_receipts_immutable_v1() OWNER TO console_terms_owner;
        REVOKE ALL ON FUNCTION public.account_terms_receipts_immutable_v1() FROM PUBLIC, console_terms_owner;
        CREATE TRIGGER account_terms_receipts_immutable_v1
            BEFORE UPDATE OR DELETE OR TRUNCATE ON public.account_terms_release_receipts
            FOR EACH STATEMENT EXECUTE FUNCTION public.account_terms_receipts_immutable_v1();
        ALTER TABLE public.account_terms_release_receipts ENABLE ALWAYS TRIGGER account_terms_receipts_immutable_v1;
        -- FK key-share checks need these ordinary privileges. Install the
        -- unconditional statement guard before granting any UPDATE authority.
        GRANT SELECT(id,revision,manifest_sha256), UPDATE(id)
            ON public.account_terms_release_receipts TO console_terms_owner;
    END IF;
    IF NOT fence_present THEN
        GRANT SELECT ON public.accounts, public.account_security TO console_account_owner;
        GRANT UPDATE(id) ON public.accounts TO console_account_owner;
        EXECUTE pg_catalog.format('CREATE FUNCTION public.account_legacy_fenced_v1(subject_account_id uuid)
            RETURNS boolean LANGUAGE plpgsql STABLE SECURITY DEFINER
            SET search_path=pg_catalog,pg_temp SET row_security=off AS %L', expected_fence_body);
        ALTER FUNCTION public.account_legacy_fenced_v1(uuid) OWNER TO console_account_owner;
        REVOKE ALL ON FUNCTION public.account_legacy_fenced_v1(uuid) FROM PUBLIC;
        GRANT EXECUTE ON FUNCTION public.account_legacy_fenced_v1(uuid) TO console_auth_rt;
    END IF;
    IF NOT current_present THEN
        GRANT SELECT(id,manifest_sha256,revision) ON public.account_terms_head TO console_terms_owner;
        EXECUTE pg_catalog.format('CREATE FUNCTION public.account_terms_current_v1()
            RETURNS TABLE(manifest_sha256 bytea,revision bigint) LANGUAGE sql STABLE SECURITY DEFINER
            SET search_path=pg_catalog,pg_temp AS %L', expected_current_body);
        ALTER FUNCTION public.account_terms_current_v1() OWNER TO console_terms_owner;
        REVOKE ALL ON FUNCTION public.account_terms_current_v1() FROM PUBLIC;
        GRANT EXECUTE ON FUNCTION public.account_terms_current_v1() TO console_auth_rt;
    END IF;
"""

ROOT_INSTALL = """
    -- The old profile was certified under users-first exclusive locks. Reject
    -- all initial UUID overlaps before reaching this plain, one-time copy.
    INSERT INTO public.accounts(id,created_at)
        SELECT id,created_at FROM public.users;
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_roots_immutable_v1()
        RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER
        SET search_path=pg_catalog,pg_temp AS %L', expected_root_guard_body);
    ALTER FUNCTION public.account_roots_immutable_v1() OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_roots_immutable_v1() FROM PUBLIC, console_account_owner;
    CREATE TRIGGER account_roots_immutable_v1
        BEFORE UPDATE OR DELETE OR TRUNCATE ON public.accounts
        FOR EACH STATEMENT EXECUTE FUNCTION public.account_roots_immutable_v1();
    ALTER TABLE public.accounts ENABLE ALWAYS TRIGGER account_roots_immutable_v1;
    -- UPDATE(id) remains only for native FK/key-share semantics. The statement
    -- guard refuses every actual UPDATE, including no-op and zero-row writes.
    GRANT INSERT(id,created_at) ON public.accounts TO console_account_owner;
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_legacy_user_root_v1()
        RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY DEFINER
        SET search_path=pg_catalog,pg_temp AS %L', expected_root_bridge_body);
    ALTER FUNCTION public.account_legacy_user_root_v1() OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_legacy_user_root_v1() FROM PUBLIC, console_account_owner;
    CREATE TRIGGER "00_account_legacy_user_root_v1" AFTER INSERT ON public.users
        FOR EACH ROW EXECUTE FUNCTION public.account_legacy_user_root_v1();
    ALTER TABLE public.users ENABLE ALWAYS TRIGGER "00_account_legacy_user_root_v1";
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_legacy_user_id_immutable_v1()
        RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER
        SET search_path=pg_catalog,pg_temp AS %L', expected_user_id_guard_body);
    ALTER FUNCTION public.account_legacy_user_id_immutable_v1() OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_legacy_user_id_immutable_v1() FROM PUBLIC, console_account_owner;
    -- AFTER UPDATE without OF observes the final NEW.id, including rewrites
    -- made by other BEFORE triggers. Numeric names precede immediate RI checks.
    CREATE TRIGGER "00_account_legacy_user_id_immutable_v1" AFTER UPDATE ON public.users
        FOR EACH ROW EXECUTE FUNCTION public.account_legacy_user_id_immutable_v1();
    ALTER TABLE public.users ENABLE ALWAYS TRIGGER "00_account_legacy_user_id_immutable_v1";
    ALTER TABLE public.users ADD CONSTRAINT users_account_root_v1
        FOREIGN KEY(id) REFERENCES public.accounts(id)
        ON DELETE RESTRICT ON UPDATE RESTRICT DEFERRABLE INITIALLY DEFERRED;
"""


def generated_files():
    query = (ROOT / 'backend/app/src/account_custody_state.sql').read_text().strip().removesuffix(';')
    for name, body in (('account_legacy_fenced_v1', FENCE_BODY),
                       ('account_terms_receipts_immutable_v1', GUARD_BODY),
                       ('account_terms_current_v1', CURRENT_BODY),
                       ('account_roots_immutable_v1', ROOT_GUARD_BODY),
                       ('account_legacy_user_root_v1', ROOT_BRIDGE_BODY),
                       ('account_legacy_user_id_immutable_v1', USER_ID_GUARD_BODY),
                       ('account_company_deactivation_guard_v1', DEACTIVATION_GUARD_BODY)):
        expected = f"('{name}','{hashlib.sha256(body.encode()).hexdigest()}')"
        if query.count(expected) != 1:
            raise SystemExit('reviewed custody routine digest differs: ' + name)
    names = ','.join("'" + name + "'" for name in TABLES)
    locks = ', '.join('ONLY public.' + name for name in TABLES)
    inspect = 'state := (\n' + query + '\n);'
    sql = f"""-- Generated by ops/generate-account-custody.py; review the canonical query.
-- One atomic statement for SQLx and psql. The operator transport must set its
-- statement_timeout before issuing this DO; changing it inside a running
-- statement would not bound that statement. Relation waits are bounded here.
DO $account_custody$
DECLARE
    state text;
    relation_name text;
    target_owner text;
    populated boolean;
    fence_present boolean;
    guard_present boolean;
    current_present boolean;
    root_present boolean;
    expected_fence_body text := $fence_body${FENCE_BODY}$fence_body$;
    expected_guard_body text := $guard_body${GUARD_BODY}$guard_body$;
    expected_current_body text := $current_body${CURRENT_BODY}$current_body$;
    expected_root_guard_body text := $root_guard_body${ROOT_GUARD_BODY}$root_guard_body$;
    expected_root_bridge_body text := $root_bridge_body${ROOT_BRIDGE_BODY}$root_bridge_body$;
    expected_user_id_guard_body text := $user_id_guard_body${USER_ID_GUARD_BODY}$user_id_guard_body$;
    expected_deactivation_guard_body text := $deactivation_guard_body${DEACTIVATION_GUARD_BODY}$deactivation_guard_body$;
BEGIN
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    PERFORM pg_catalog.set_config('lock_timeout','5s',true);
    IF session_user<>current_user
       OR NOT (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user)
       OR session_user IN ('console_app','console_rt','console_auth_rt',
          'console_leave_cmd','console_leave_definer','console_ontology_cmd',
          'console_ontology_writer','console_platform_force_cmd',
          'console_account_owner','console_terms_owner','console_credential_owner') THEN
        RAISE EXCEPTION USING MESSAGE='account_custody.operator_identity_mismatch', ERRCODE='P0001';
    END IF;
    -- Before locking, inspect only existence and kind. Catalog helpers in the
    -- full verdict can wait across another finalizer's ownership/ACL commit.
    IF (SELECT count(*) FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN ({names})) <> {len(TABLES)} THEN
        RAISE EXCEPTION USING MESSAGE='account_custody.catalog_missing', ERRCODE='P0001';
    END IF;
    IF EXISTS (SELECT 1 FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN ({names}) AND c.relkind<>'r') THEN
        RAISE EXCEPTION USING MESSAGE='account_custody.catalog_shape_mismatch', ERRCODE='P0001';
    END IF;
    IF NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class
        WHERE oid=pg_catalog.to_regclass('public.users') AND relkind='r') THEN
        RAISE EXCEPTION USING MESSAGE='account_root_transition.profile_mismatch', ERRCODE='P0001';
    END IF;
    LOCK TABLE ONLY public.organizations IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE ONLY public.users IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE {locks} IN ACCESS EXCLUSIVE MODE;
    -- Freeze legacy writers first, then certify the complete historical or
    -- current profile under all seven locks. Never infer identity from overlap.
    {inspect}
    IF state IS DISTINCT FROM 'account_custody.finalized'
       AND state IS DISTINCT FROM 'account_custody.pending'
       AND state IS DISTINCT FROM 'account_custody.upgrade_required' THEN
        RAISE EXCEPTION USING MESSAGE=COALESCE(state,'account_custody.catalog_missing'), ERRCODE='P0001';
    END IF;
    -- Auth LOGIN topology is an operator precondition, including replay.
    -- Serving availability belongs to the retained Auth transport's health
    -- check; temporarily stopping that LOGIN does not corrupt custody metadata.
    IF NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_auth_rt'
          AND rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit
          AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication
    ) OR EXISTS (
        SELECT 1 FROM pg_catalog.pg_auth_members m
        JOIN pg_catalog.pg_roles r ON r.oid=m.member OR r.oid=m.roleid
        WHERE r.rolname='console_auth_rt'
    ) THEN
        RAISE EXCEPTION 'account_fence_projection.role_mismatch';
    END IF;
    -- The complete verdict above certified every present root/profile field.
    -- Presence now selects the already-certified upgrade path, never authority.
    SELECT pg_catalog.to_regprocedure('public.account_roots_immutable_v1()') IS NOT NULL INTO root_present;
    IF root_present THEN
        -- Serving admission is metadata-only. The privileged operator alone
        -- detects corrupt missing roots and refuses rather than repairing them.
        IF EXISTS(SELECT 1 FROM public.users u LEFT JOIN public.accounts a ON a.id=u.id WHERE a.id IS NULL) THEN
            RAISE EXCEPTION USING MESSAGE='account_root_transition.missing_root', ERRCODE='P0001';
        END IF;
    END IF;
    IF state='account_custody.finalized' THEN
        RETURN;
    END IF;
    IF NOT root_present THEN
    -- The same complete read-only contract certifies historical input before
    -- any mutation and current output afterward. No second routine validator.
    SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') INTO fence_present;
    SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') INTO guard_present;
    SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='public' AND p.proname='account_terms_current_v1') INTO current_present;
    IF state='account_custody.pending' THEN
    FOREACH relation_name IN ARRAY ARRAY[{names}] LOOP
        EXECUTE format('SELECT EXISTS(SELECT 1 FROM public.%I)',relation_name) INTO populated;
        IF populated IS DISTINCT FROM false THEN
            RAISE EXCEPTION USING MESSAGE='account_custody.nonempty_staging', ERRCODE='P0001';
        END IF;
    END LOOP;
    END IF;
    IF EXISTS(SELECT 1 FROM public.users u JOIN public.accounts a ON a.id=u.id) THEN
        RAISE EXCEPTION USING MESSAGE='account_root_transition.overlap_requires_admission', ERRCODE='P0001';
    END IF;
    IF state='account_custody.pending' THEN
    FOREACH relation_name IN ARRAY ARRAY[{names}] LOOP
        target_owner := CASE WHEN relation_name IN ('account_terms_head','account_terms_release_receipts')
            THEN 'console_terms_owner' ELSE 'console_account_owner' END;
        EXECUTE format('ALTER TABLE public.%I OWNER TO %I',relation_name,target_owner);
    END LOOP;
    {inspect}
    IF state IS DISTINCT FROM 'account_custody.upgrade_required' THEN
        RAISE EXCEPTION USING MESSAGE=COALESCE(state,'account_custody.catalog_missing'), ERRCODE='P0001';
    END IF;
    END IF;
{INSTALL}
{ROOT_INSTALL}
    END IF;
{DEACTIVATION_INSTALL}
    -- Certify the complete installed state, not merely the routine definition.
    {inspect}
    IF state IS DISTINCT FROM 'account_custody.finalized' THEN
        RAISE EXCEPTION USING MESSAGE=COALESCE(state,'account_custody.catalog_missing'), ERRCODE='P0001';
    END IF;
END
$account_custody$;
"""
    migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
    ledger = ''.join(str(int(path.name.split('_', 1)[0])) + '\t'
                     + hashlib.sha384(path.read_bytes()).hexdigest() + '\n' for path in migrations)
    return {'ops/postgres-finalize-account-custody.sql': sql,
            'ops/account-custody-migrations.sha384': ledger,
            **credential_generated_files()}


# Auth7 constants freeze the admitted226 catalog and explicit final mutations.
CREDENTIAL_TABLES = ('auth_bootstrap_credentials', 'auth_device_login_handoffs', 'auth_refresh_token_families', 'auth_refresh_tokens', 'auth_webauthn_ceremonies', 'auth_webauthn_ceremony_bindings', 'auth_webauthn_credentials')
CREDENTIAL_STATE_QUERY = r"""-- Generated from admitted source226 plus the fixed Auth7 transition in ops/generate-account-custody.py.
-- Read-only metadata verdict; no credential/user/audit row reads and no expected-data learning.
WITH wanted(name) AS (VALUES
 ('auth_webauthn_credentials'), ('auth_webauthn_ceremonies'),
 ('auth_refresh_token_families'), ('auth_refresh_tokens'),
 ('auth_bootstrap_credentials'), ('auth_webauthn_ceremony_bindings'),
 ('auth_device_login_handoffs')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE n.nspname='public' AND p.proname IN ('auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org')
), credential_role AS (
 SELECT oid,NOT (rolsuper OR rolcanlogin OR rolinherit OR rolbypassrls OR rolcreatedb OR rolcreaterole OR rolreplication)
   AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=r.oid OR m.member=r.oid)
   AND NOT EXISTS(SELECT 1 FROM pg_default_acl d WHERE d.defaclrole=r.oid)
   AND NOT has_schema_privilege(r.oid,'public','CREATE') AS valid
 FROM pg_roles r WHERE rolname='console_credential_owner'
), role_boundaries AS (
 SELECT NOT EXISTS(SELECT 1 FROM credential_role WHERE NOT valid)
   AND NOT EXISTS(SELECT 1 FROM pg_roles r WHERE r.rolname='console_auth_rt'
    AND (r.rolsuper OR r.rolinherit OR r.rolbypassrls OR r.rolcreatedb OR r.rolcreaterole OR r.rolreplication))
   AND EXISTS(SELECT 1 FROM pg_roles WHERE rolname='console_auth_rt')
   AND NOT EXISTS(SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.roleid OR r.oid=m.member WHERE r.rolname='console_auth_rt')
   AND NOT EXISTS(SELECT 1 FROM credential_role owner_role JOIN pg_class c ON c.relowner=owner_role.oid
      WHERE c.relkind IN ('r','p','v','m','S','f') AND c.oid NOT IN (SELECT oid FROM relations))
   AND NOT EXISTS(SELECT 1 FROM credential_role r JOIN pg_namespace n ON n.nspowner=r.oid)
   AND NOT EXISTS(SELECT 1 FROM credential_role r CROSS JOIN pg_class c
     CROSS JOIN LATERAL aclexplode(c.relacl) x WHERE x.grantee=r.oid AND c.oid NOT IN (SELECT oid FROM relations))
   AND NOT EXISTS(SELECT 1 FROM credential_role r CROSS JOIN pg_attribute a
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE x.grantee=r.oid AND a.attrelid NOT IN (SELECT oid FROM relations))
   AND NOT EXISTS(SELECT 1 FROM credential_role owner_role JOIN pg_proc p ON p.proowner=owner_role.oid
      WHERE p.oid NOT IN (SELECT oid FROM routine_records))
   AND NOT EXISTS(SELECT 1 FROM credential_role r CROSS JOIN (VALUES('users'),('audit_events'),('accounts'),('account_security'),
     ('account_security_events'),('account_terms_acceptances'),('account_terms_head'),('account_terms_release_receipts')) denied(name)
     WHERE has_table_privilege(r.oid,to_regclass('public.'||denied.name),'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
      OR has_any_column_privilege(r.oid,to_regclass('public.'||denied.name),'SELECT,INSERT,UPDATE,REFERENCES')) AS valid
), foreign_keys AS (
 SELECT k.*,
   k.connamespace='public'::regnamespace AND k.contypid=0 AND k.conparentid=0
   AND k.convalidated AND k.conenforced AND NOT k.conperiod AND k.connoinherit
   AND k.conislocal AND k.coninhcount=0 AND NOT k.condeferrable AND NOT k.condeferred
   AND k.confmatchtype='s' AND k.confdelsetcols IS NULL AND k.conbin IS NULL AND k.conexclop IS NULL
   AND k.conpfeqop=array_fill('pg_catalog.=(uuid,uuid)'::regoperator::oid,ARRAY[cardinality(k.conkey)])
   AND k.conppeqop=k.conpfeqop AND k.conffeqop=k.conpfeqop
   AND i.indrelid=k.confrelid AND i.indisunique AND i.indisvalid AND i.indisready
   AND i.indislive AND i.indimmediate AND NOT i.indisexclusion
   AND i.indexprs IS NULL AND i.indpred IS NULL AND i.indnkeyatts=cardinality(k.confkey)
   AND i.indnatts=i.indnkeyatts AND i.indkey::text=array_to_string(k.confkey,' ')
   AND NOT EXISTS(SELECT 1 FROM unnest(k.conkey) key_column LEFT JOIN pg_attribute a
      ON a.attrelid=k.conrelid AND a.attnum=key_column
      WHERE a.atttypid IS DISTINCT FROM 'pg_catalog.uuid'::regtype::oid OR a.attisdropped) AS valid
 FROM pg_constraint k LEFT JOIN pg_index i ON i.indexrelid=k.conindid
 WHERE k.contype='f' AND k.conrelid IN (SELECT oid FROM relations)
), foreign_ri AS (
 SELECT f.oid,
   (SELECT count(*)=4 FROM pg_trigger t WHERE t.tgconstraint=f.oid)
   AND (SELECT count(*)=4 AND count(DISTINCT e.function_name)=4 AND bool_and(
      t.tgrelid=CASE WHEN e.child THEN f.conrelid ELSE f.confrelid END
      AND t.tgconstrrelid=CASE WHEN e.child THEN f.confrelid ELSE f.conrelid END
      AND t.tgconstrindid=f.conindid AND t.tgconstraint=f.oid
      AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
      AND NOT t.tgdeferrable AND NOT t.tginitdeferred AND t.tgparentid=0
      AND t.tgname::text ~ CASE WHEN e.child THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
      AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
      AND t.tgqual IS NULL AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
    FROM (VALUES(true,'RI_FKey_check_ins',5),(true,'RI_FKey_check_upd',17),
      (false,'RI_FKey_'||CASE f.confdeltype WHEN 'r' THEN 'restrict' WHEN 'a' THEN 'noaction' WHEN 'c' THEN 'cascade' WHEN 'n' THEN 'setnull' END||'_del',9),
      (false,'RI_FKey_'||CASE f.confupdtype WHEN 'r' THEN 'restrict' WHEN 'a' THEN 'noaction' END||'_upd',17)) e(child,function_name,trigger_type)
    JOIN pg_trigger t ON t.tgconstraint=f.oid AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM foreign_keys f
), metadata_boundary AS (
 SELECT NOT EXISTS(SELECT 1 FROM foreign_keys WHERE valid IS DISTINCT FROM true)
   AND NOT EXISTS(SELECT 1 FROM foreign_ri WHERE valid IS DISTINCT FROM true)
   AND NOT EXISTS(SELECT 1 FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)
      AND (NOT k.conenforced OR k.conperiod OR k.contypid<>0 OR k.conparentid<>0 OR k.coninhcount<>0 OR NOT k.conislocal))
   AND NOT EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgrelid IN (SELECT oid FROM relations)
     AND (t.tgparentid<>0 OR t.tgnargs<>0 OR octet_length(t.tgargs)<>0 OR t.tgattr<>''::int2vector
       OR t.tgoldtable IS NOT NULL OR t.tgnewtable IS NOT NULL OR (t.tgisinternal AND t.tgqual IS NOT NULL))) AS valid
), snapshots AS (
 SELECT (SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records) AS tables,
        (SELECT jsonb_agg(record ORDER BY record->>'name') FROM routine_records) AS routines
)
SELECT CASE
 WHEN (SELECT valid FROM role_boundaries) IS DISTINCT FROM true OR (SELECT valid FROM metadata_boundary) IS DISTINCT FROM true
   THEN CASE WHEN EXISTS(SELECT 1 FROM relations WHERE relowner=(SELECT oid FROM credential_role)) THEN 'account_credentials.profile_mismatch' ELSE 'account_credentials.legacy_profile_mismatch' END
 WHEN s.tables='[{"name":"auth_bootstrap_credentials","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"registration_ceremony_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"registration_started_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[11,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[12,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_bootstrap_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_pkey ON public.auth_bootstrap_credentials USING btree (id)"],["auth_bootstrap_credentials_registration_ceremony_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_registration_ceremony_id_key ON public.auth_bootstrap_credentials USING btree (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_token_hash_key ON public.auth_bootstrap_credentials USING btree (token_hash)"],["idx_auth_bootstrap_credentials_active_hash",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_active_hash ON public.auth_bootstrap_credentials USING btree (token_hash, expires_at) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_one_open_per_user",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX idx_auth_bootstrap_credentials_one_open_per_user ON public.auth_bootstrap_credentials USING btree (user_id) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_user ON public.auth_bootstrap_credentials USING btree (user_id, issued_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_bootstrap_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_bootstrap_credentials_org_immutable BEFORE UPDATE ON public.auth_bootstrap_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null]],"constraints":[["auth_bootstrap_credentials_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_bootstrap_credentials_check1","c",true,false,false,false,true,0,true,[6,7],null," "," "," ",null,null,"CHECK (((registration_ceremony_id IS NULL) OR (registration_started_at IS NOT NULL)))"],["auth_bootstrap_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[11],null," "," "," ",null,null,"NOT NULL created_at"],["auth_bootstrap_credentials_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_bootstrap_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_bootstrap_credentials_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_bootstrap_credentials_org_fk","f",true,false,false,true,true,0,true,[12],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_bootstrap_credentials_org_id_not_null","n",true,false,false,false,true,0,true,[12],null," "," "," ",null,null,"NOT NULL org_id"],["auth_bootstrap_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_bootstrap_credentials_registration_ceremony_id_fkey","f",true,false,false,true,true,0,true,[6],[1],"a","n","s","public","auth_webauthn_ceremonies","FOREIGN KEY (registration_ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON DELETE SET NULL"],["auth_bootstrap_credentials_registration_ceremony_id_key","u",true,false,false,true,true,0,true,[6],null," "," "," ",null,null,"UNIQUE (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_bootstrap_credentials_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_bootstrap_credentials_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_bootstrap_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"],["auth_bootstrap_credentials_user_same_org_fk","f",true,false,false,true,true,0,true,[2,12],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"registration_ceremony_id","number":6,"acl_is_null":true},{"acl":null,"name":"registration_started_at","number":7,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":10,"acl_is_null":true},{"acl":null,"name":"created_at","number":11,"acl_is_null":true},{"acl":null,"name":"org_id","number":12,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]},{"name":"auth_device_login_handoffs","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"poll_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[3,"approve_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"target_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"target_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[8,"approved_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"approved_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[10,"approved_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[11,"approved_passkey_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[12,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[13,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_device_login_handoffs_approve_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_approve_token_hash_key ON public.auth_device_login_handoffs USING btree (approve_token_hash)"],["auth_device_login_handoffs_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_pkey ON public.auth_device_login_handoffs USING btree (id)"],["auth_device_login_handoffs_poll_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_poll_token_hash_key ON public.auth_device_login_handoffs USING btree (poll_token_hash)"],["idx_auth_device_login_handoffs_approve_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_approve_active ON public.auth_device_login_handoffs USING btree (approve_token_hash, expires_at) WHERE ((approved_at IS NULL) AND (consumed_at IS NULL))"],["idx_auth_device_login_handoffs_poll_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_poll_active ON public.auth_device_login_handoffs USING btree (poll_token_hash, expires_at) WHERE (consumed_at IS NULL)"],["idx_auth_device_login_handoffs_target",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_target ON public.auth_device_login_handoffs USING btree (target_user_id, issued_at DESC) WHERE (target_user_id IS NOT NULL)"],["idx_auth_device_login_handoffs_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_user ON public.auth_device_login_handoffs USING btree (approved_user_id, approved_at DESC) WHERE (approved_user_id IS NOT NULL)"]],"policies":0,"relation":["r","p",false,false,false,"d",null],"triggers":null,"constraints":[["auth_device_login_handoffs_approve_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (approve_token_hash)"],["auth_device_login_handoffs_approve_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL approve_token_hash"],["auth_device_login_handoffs_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_device_login_handoffs_check1","c",true,false,false,false,true,0,true,[6,7],null," "," "," ",null,null,"CHECK (((target_user_id IS NULL) = (target_org_id IS NULL)))"],["auth_device_login_handoffs_check2","c",true,false,false,false,true,0,true,[8,9,10],null," "," "," ",null,null,"CHECK ((((approved_at IS NULL) AND (approved_user_id IS NULL) AND (approved_org_id IS NULL)) OR ((approved_at IS NOT NULL) AND (approved_user_id IS NOT NULL) AND (approved_org_id IS NOT NULL))))"],["auth_device_login_handoffs_created_at_not_null","n",true,false,false,false,true,0,true,[13],null," "," "," ",null,null,"NOT NULL created_at"],["auth_device_login_handoffs_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_device_login_handoffs_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_device_login_handoffs_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_device_login_handoffs_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_device_login_handoffs_poll_token_hash_key","u",true,false,false,true,true,0,true,[2],null," "," "," ",null,null,"UNIQUE (poll_token_hash)"],["auth_device_login_handoffs_poll_token_hash_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL poll_token_hash"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"poll_token_hash","number":2,"acl_is_null":true},{"acl":null,"name":"approve_token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"target_user_id","number":6,"acl_is_null":true},{"acl":null,"name":"target_org_id","number":7,"acl_is_null":true},{"acl":null,"name":"approved_at","number":8,"acl_is_null":true},{"acl":null,"name":"approved_user_id","number":9,"acl_is_null":true},{"acl":null,"name":"approved_org_id","number":10,"acl_is_null":true},{"acl":null,"name":"approved_passkey_id","number":11,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":12,"acl_is_null":true},{"acl":null,"name":"created_at","number":13,"acl_is_null":true}],"policies":[]},{"name":"auth_refresh_token_families","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[4,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[5,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[6,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_refresh_token_families_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_token_families_pkey ON public.auth_refresh_token_families USING btree (id)"],["idx_auth_refresh_token_families_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_token_families_user ON public.auth_refresh_token_families USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_token_families_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_token_families_org_immutable BEFORE UPDATE ON public.auth_refresh_token_families FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_cascade_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_token_families_created_at_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL created_at"],["auth_refresh_token_families_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_token_families_org_fk","f",true,false,false,true,true,0,true,[6],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_token_families_org_id_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL org_id"],["auth_refresh_token_families_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_token_families_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_refresh_token_families_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"],["auth_refresh_token_families_user_same_org_fk","f",true,false,false,true,true,0,true,[2,6],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"created_at","number":3,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":4,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":5,"acl_is_null":true},{"acl":null,"name":"org_id","number":6,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]},{"name":"auth_refresh_tokens","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"family_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[5,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"replaced_by","pg_catalog","uuid",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"reuse_detected_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[11,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_refresh_tokens_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_tokens_pkey ON public.auth_refresh_tokens USING btree (id)"],["auth_refresh_tokens_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_refresh_tokens_token_hash_key ON public.auth_refresh_tokens USING btree (token_hash)"],["idx_auth_refresh_tokens_family",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_family ON public.auth_refresh_tokens USING btree (family_id, issued_at)"],["idx_auth_refresh_tokens_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_user_active ON public.auth_refresh_tokens USING btree (user_id, expires_at) WHERE ((used_at IS NULL) AND (revoked_at IS NULL))"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_tokens_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_tokens_org_immutable BEFORE UPDATE ON public.auth_refresh_tokens FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_token_families",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_tokens",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_token_families",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_noaction_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_tokens_check","c",true,false,false,false,true,0,true,[6,5],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_refresh_tokens_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_refresh_tokens_family_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","auth_refresh_token_families","FOREIGN KEY (family_id) REFERENCES public.auth_refresh_token_families(id) ON DELETE CASCADE"],["auth_refresh_tokens_family_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL family_id"],["auth_refresh_tokens_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_tokens_issued_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_refresh_tokens_org_fk","f",true,false,false,true,true,0,true,[11],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_tokens_org_id_not_null","n",true,false,false,false,true,0,true,[11],null," "," "," ",null,null,"NOT NULL org_id"],["auth_refresh_tokens_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_tokens_replaced_by_fkey","f",true,false,false,true,true,0,true,[8],[1],"a","a","s","public","auth_refresh_tokens","FOREIGN KEY (replaced_by) REFERENCES public.auth_refresh_tokens(id)"],["auth_refresh_tokens_token_hash_key","u",true,false,false,true,true,0,true,[4],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_refresh_tokens_token_hash_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_refresh_tokens_user_id_fkey","f",true,false,false,true,true,0,true,[3],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_refresh_tokens_user_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL user_id"],["auth_refresh_tokens_user_same_org_fk","f",true,false,false,true,true,0,true,[3,11],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"family_id","number":2,"acl_is_null":true},{"acl":null,"name":"user_id","number":3,"acl_is_null":true},{"acl":null,"name":"token_hash","number":4,"acl_is_null":true},{"acl":null,"name":"issued_at","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"used_at","number":7,"acl_is_null":true},{"acl":null,"name":"replaced_by","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"reuse_detected_at","number":10,"acl_is_null":true},{"acl":null,"name":"org_id","number":11,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]},{"name":"auth_webauthn_ceremonies","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[3,"ceremony_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"challenge_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"state_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremonies_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremonies_pkey ON public.auth_webauthn_ceremonies USING btree (id)"],["idx_auth_webauthn_ceremonies_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremonies_user_active ON public.auth_webauthn_ceremonies USING btree (user_id, ceremony_kind, expires_at) WHERE (consumed_at IS NULL)"]],"policies":0,"relation":["r","p",false,false,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_bootstrap_credentials",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_cascade_del","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_setnull_del","public","auth_bootstrap_credentials",null]],"constraints":[["auth_webauthn_ceremonies_ceremony_kind_check","c",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"CHECK ((ceremony_kind = ANY (ARRAY[''registration''::text, ''authentication''::text])))"],["auth_webauthn_ceremonies_ceremony_kind_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL ceremony_kind"],["auth_webauthn_ceremonies_challenge_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL challenge_json"],["auth_webauthn_ceremonies_check","c",true,false,false,false,true,0,true,[6,8],null," "," "," ",null,null,"CHECK ((expires_at > created_at))"],["auth_webauthn_ceremonies_created_at_not_null","n",true,false,false,false,true,0,true,[8],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremonies_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_webauthn_ceremonies_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_ceremonies_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_ceremonies_state_json_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL state_json"],["auth_webauthn_ceremonies_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"ceremony_kind","number":3,"acl_is_null":true},{"acl":null,"name":"challenge_json","number":4,"acl_is_null":true},{"acl":null,"name":"state_json","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":7,"acl_is_null":true},{"acl":null,"name":"created_at","number":8,"acl_is_null":true}],"policies":[]},{"name":"auth_webauthn_ceremony_bindings","owner":"console_app","shape":{"rules":null,"columns":[[1,"ceremony_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[2,"action_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[3,"object_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"reason_key","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[5,"replay_attempt","pg_catalog","int4",-1,false,false,"","",null,null,null],[6,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremony_bindings_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremony_bindings_pkey ON public.auth_webauthn_ceremony_bindings USING btree (ceremony_id)"],["idx_auth_webauthn_ceremony_bindings_action",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremony_bindings_action ON public.auth_webauthn_ceremony_bindings USING btree (action_kind, object_id, replay_attempt)"]],"policies":0,"relation":["r","p",false,false,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null]],"constraints":[["auth_webauthn_ceremony_bindings_action_kind_check","c",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"CHECK ((action_kind = ANY (ARRAY[''APPROVAL_DECISION''::text, ''POLL_VOTE''::text])))"],["auth_webauthn_ceremony_bindings_action_kind_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL action_kind"],["auth_webauthn_ceremony_bindings_ceremony_id_fkey","f",true,false,false,true,true,0,true,[1],[1],"a","c","s","public","auth_webauthn_ceremonies","FOREIGN KEY (ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON DELETE CASCADE"],["auth_webauthn_ceremony_bindings_ceremony_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL ceremony_id"],["auth_webauthn_ceremony_bindings_check","c",true,false,false,false,true,0,true,[2,4],null," "," "," ",null,null,"CHECK ((((action_kind = ''APPROVAL_DECISION''::text) AND (reason_key = ''operations_passkey_approval_decision''::text)) OR ((action_kind = ''POLL_VOTE''::text) AND (reason_key = ''operations_passkey_poll_vote''::text))))"],["auth_webauthn_ceremony_bindings_created_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremony_bindings_object_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL object_id"],["auth_webauthn_ceremony_bindings_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (ceremony_id)"],["auth_webauthn_ceremony_bindings_reason_key_check","c",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"CHECK ((reason_key = ANY (ARRAY[''operations_passkey_approval_decision''::text, ''operations_passkey_poll_vote''::text])))"],["auth_webauthn_ceremony_bindings_reason_key_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL reason_key"],["auth_webauthn_ceremony_bindings_replay_attempt_check","c",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"CHECK (((replay_attempt IS NULL) OR (replay_attempt >= 1)))"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"ceremony_id","number":1,"acl_is_null":true},{"acl":null,"name":"action_kind","number":2,"acl_is_null":true},{"acl":null,"name":"object_id","number":3,"acl_is_null":true},{"acl":null,"name":"reason_key","number":4,"acl_is_null":true},{"acl":null,"name":"replay_attempt","number":5,"acl_is_null":true},{"acl":null,"name":"created_at","number":6,"acl_is_null":true}],"policies":[]},{"name":"auth_webauthn_credentials","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"credential_id","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"passkey_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[6,"last_used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[7,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_webauthn_credentials_credential_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_credential_id_key ON public.auth_webauthn_credentials USING btree (credential_id)"],["auth_webauthn_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_pkey ON public.auth_webauthn_credentials USING btree (id)"],["idx_auth_webauthn_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_credentials_user ON public.auth_webauthn_credentials USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_webauthn_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_webauthn_credentials_org_immutable BEFORE UPDATE ON public.auth_webauthn_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null]],"constraints":[["auth_webauthn_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_credentials_credential_id_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (credential_id)"],["auth_webauthn_credentials_credential_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL credential_id"],["auth_webauthn_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_credentials_org_fk","f",true,false,false,true,true,0,true,[7],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_webauthn_credentials_org_id_not_null","n",true,false,false,false,true,0,true,[7],null," "," "," ",null,null,"NOT NULL org_id"],["auth_webauthn_credentials_passkey_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL passkey_json"],["auth_webauthn_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_credentials_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_webauthn_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"],["auth_webauthn_credentials_user_same_org_fk","f",true,false,false,true,true,0,true,[2,7],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"credential_id","number":3,"acl_is_null":true},{"acl":null,"name":"passkey_json","number":4,"acl_is_null":true},{"acl":null,"name":"created_at","number":5,"acl_is_null":true},{"acl":null,"name":"last_used_at","number":6,"acl_is_null":true},{"acl":null,"name":"org_id","number":7,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]}]'::jsonb AND s.routines='[{"name":"enforce_org_id_immutable","identity_arguments":"","result":"trigger","owner":"console_app","language":"plpgsql","kind":"f","security_definer":false,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":null,"argnames":null,"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"a46ee31ccf7fae026153b8bea7419777f055a4028e9ecfe0632e262a6f41f3ba","acl":[["console_app","PUBLIC","EXECUTE",false],["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_direct_org_children","identity_arguments":"p_id uuid","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"8c1cc63c439dd5aafe28090526099c33966e882e0b28e647a9f8ecec34458ba6","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"5d6c71c6b4d393d90e51a187e3e016bd02d4fdc47ff684cfee396d752efe5224","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_list_group_accounts","identity_arguments":"p_group_id uuid","result":"TABLE(user_id uuid, display_name text, phone text, tenant_roles text[], is_active boolean, has_passkey boolean, account_status text, org_id uuid, org_slug text, org_name text, group_roles text[], created_at timestamp with time zone)","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_group_id","user_id","display_name","phone","tenant_roles","is_active","has_passkey","account_status","org_id","org_slug","org_name","group_roles","created_at"],"argmodes":["i","t","t","t","t","t","t","t","t","t","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"6a7ff12b1e25edd0ec06ab431f897601a01731078620c363dcdbe5cb5387af71","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"b8b9c005155e036ad206fdbd2426ab39591726cf5dee2e89e1e02186b716d77a","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_bootstrap_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"27f96d16d7e825bed30402da9d9e4d1bdd0695ebec669b438d3805250d956233","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_credential_org","identity_arguments":"p_credential_id text","result":"uuid","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_credential_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"360c2f961244beee18e21607fd0e38635a137b568ae15889b11a2c91993d98bf","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_token_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_app","language":"sql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"125b8ba9b42757a439cb9c74e175bb11d2af62e3df99d34d71b88f05ab8a155c","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]}]'::jsonb
   AND NOT EXISTS(SELECT 1 FROM routine_records WHERE NOT extra_valid)
   THEN 'account_credentials.pending'
 WHEN s.tables='[{"name":"auth_bootstrap_credentials","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"registration_ceremony_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"registration_started_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[11,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[12,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_bootstrap_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_pkey ON public.auth_bootstrap_credentials USING btree (id)"],["auth_bootstrap_credentials_registration_ceremony_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_registration_ceremony_id_key ON public.auth_bootstrap_credentials USING btree (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_token_hash_key ON public.auth_bootstrap_credentials USING btree (token_hash)"],["idx_auth_bootstrap_credentials_active_hash",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_active_hash ON public.auth_bootstrap_credentials USING btree (token_hash, expires_at) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_one_open_per_user",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX idx_auth_bootstrap_credentials_one_open_per_user ON public.auth_bootstrap_credentials USING btree (user_id) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_user ON public.auth_bootstrap_credentials USING btree (user_id, issued_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_bootstrap_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_bootstrap_credentials_org_immutable BEFORE UPDATE ON public.auth_bootstrap_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null]],"constraints":[["auth_bootstrap_credentials_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_bootstrap_credentials_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_bootstrap_credentials_check1","c",true,false,false,false,true,0,true,[6,7],null," "," "," ",null,null,"CHECK (((registration_ceremony_id IS NULL) OR (registration_started_at IS NOT NULL)))"],["auth_bootstrap_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[11],null," "," "," ",null,null,"NOT NULL created_at"],["auth_bootstrap_credentials_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_bootstrap_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_bootstrap_credentials_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_bootstrap_credentials_org_fk","f",true,false,false,true,true,0,true,[12],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_bootstrap_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_bootstrap_credentials_registration_ceremony_id_fkey","f",true,false,false,true,true,0,true,[6],[1],"r","r","s","public","auth_webauthn_ceremonies","FOREIGN KEY (registration_ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_bootstrap_credentials_registration_ceremony_id_key","u",true,false,false,true,true,0,true,[6],null," "," "," ",null,null,"UNIQUE (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_bootstrap_credentials_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_bootstrap_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","INSERT",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"registration_ceremony_id","number":6,"acl_is_null":true},{"acl":null,"name":"registration_started_at","number":7,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":10,"acl_is_null":true},{"acl":null,"name":"created_at","number":11,"acl_is_null":true},{"acl":null,"name":"org_id","number":12,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_device_login_handoffs","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"poll_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[3,"approve_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"target_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"target_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[8,"approved_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"approved_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[10,"approved_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[11,"approved_passkey_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[12,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[13,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_device_login_handoffs_approve_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_approve_token_hash_key ON public.auth_device_login_handoffs USING btree (approve_token_hash)"],["auth_device_login_handoffs_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_pkey ON public.auth_device_login_handoffs USING btree (id)"],["auth_device_login_handoffs_poll_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_poll_token_hash_key ON public.auth_device_login_handoffs USING btree (poll_token_hash)"],["idx_auth_device_login_handoffs_approve_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_approve_active ON public.auth_device_login_handoffs USING btree (approve_token_hash, expires_at) WHERE ((approved_at IS NULL) AND (consumed_at IS NULL))"],["idx_auth_device_login_handoffs_poll_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_poll_active ON public.auth_device_login_handoffs USING btree (poll_token_hash, expires_at) WHERE (consumed_at IS NULL)"],["idx_auth_device_login_handoffs_target",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_target ON public.auth_device_login_handoffs USING btree (target_user_id, issued_at DESC) WHERE (target_user_id IS NOT NULL)"],["idx_auth_device_login_handoffs_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_user ON public.auth_device_login_handoffs USING btree (approved_user_id, approved_at DESC) WHERE (approved_user_id IS NOT NULL)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null]],"constraints":[["auth_device_login_handoffs_approve_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (approve_token_hash)"],["auth_device_login_handoffs_approve_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL approve_token_hash"],["auth_device_login_handoffs_approved_account_v1","f",true,false,false,true,true,0,true,[9],[1],"r","r","s","public","accounts","FOREIGN KEY (approved_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_device_login_handoffs_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_device_login_handoffs_check1","c",true,false,false,false,true,0,true,[7,6],null," "," "," ",null,null,"CHECK (((target_org_id IS NULL) OR (target_user_id IS NOT NULL)))"],["auth_device_login_handoffs_check2","c",true,false,false,false,true,0,true,[8,9,10],null," "," "," ",null,null,"CHECK ((((approved_at IS NULL) AND (approved_user_id IS NULL) AND (approved_org_id IS NULL)) OR ((approved_at IS NOT NULL) AND (approved_user_id IS NOT NULL))))"],["auth_device_login_handoffs_created_at_not_null","n",true,false,false,false,true,0,true,[13],null," "," "," ",null,null,"NOT NULL created_at"],["auth_device_login_handoffs_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_device_login_handoffs_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_device_login_handoffs_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_device_login_handoffs_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_device_login_handoffs_poll_token_hash_key","u",true,false,false,true,true,0,true,[2],null," "," "," ",null,null,"UNIQUE (poll_token_hash)"],["auth_device_login_handoffs_poll_token_hash_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL poll_token_hash"],["auth_device_login_handoffs_target_account_v1","f",true,false,false,true,true,0,true,[6],[1],"r","r","s","public","accounts","FOREIGN KEY (target_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","SELECT",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"poll_token_hash","number":2,"acl_is_null":true},{"acl":null,"name":"approve_token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"target_user_id","number":6,"acl_is_null":true},{"acl":null,"name":"target_org_id","number":7,"acl_is_null":true},{"acl":null,"name":"approved_at","number":8,"acl_is_null":true},{"acl":null,"name":"approved_user_id","number":9,"acl_is_null":true},{"acl":null,"name":"approved_org_id","number":10,"acl_is_null":true},{"acl":null,"name":"approved_passkey_id","number":11,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":12,"acl_is_null":true},{"acl":null,"name":"created_at","number":13,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_refresh_token_families","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[4,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[5,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[6,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_refresh_token_families_id_user_v1",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_refresh_token_families_id_user_v1 ON public.auth_refresh_token_families USING btree (id, user_id)"],["auth_refresh_token_families_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_token_families_pkey ON public.auth_refresh_token_families USING btree (id)"],["idx_auth_refresh_token_families_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_token_families_user ON public.auth_refresh_token_families USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_token_families_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_token_families_org_immutable BEFORE UPDATE ON public.auth_refresh_token_families FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_restrict_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_restrict_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_token_families_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_refresh_token_families_created_at_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL created_at"],["auth_refresh_token_families_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_token_families_id_user_v1","u",true,false,false,true,true,0,true,[1,2],null," "," "," ",null,null,"UNIQUE (id, user_id)"],["auth_refresh_token_families_org_fk","f",true,false,false,true,true,0,true,[6],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_token_families_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_token_families_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"created_at","number":3,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":4,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":5,"acl_is_null":true},{"acl":null,"name":"org_id","number":6,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_refresh_tokens","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"family_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[5,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"replaced_by","pg_catalog","uuid",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"reuse_detected_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[11,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_refresh_tokens_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_tokens_pkey ON public.auth_refresh_tokens USING btree (id)"],["auth_refresh_tokens_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_refresh_tokens_token_hash_key ON public.auth_refresh_tokens USING btree (token_hash)"],["idx_auth_refresh_tokens_family",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_family ON public.auth_refresh_tokens USING btree (family_id, issued_at)"],["idx_auth_refresh_tokens_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_user_active ON public.auth_refresh_tokens USING btree (user_id, expires_at) WHERE ((used_at IS NULL) AND (revoked_at IS NULL))"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_tokens_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_tokens_org_immutable BEFORE UPDATE ON public.auth_refresh_tokens FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_token_families",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_tokens",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_token_families",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_noaction_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_tokens_account_v1","f",true,false,false,true,true,0,true,[3],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_refresh_tokens_check","c",true,false,false,false,true,0,true,[6,5],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_refresh_tokens_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_refresh_tokens_family_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL family_id"],["auth_refresh_tokens_family_subject_v1","f",true,false,false,true,true,0,true,[2,3],[1,2],"r","r","s","public","auth_refresh_token_families","FOREIGN KEY (family_id, user_id) REFERENCES public.auth_refresh_token_families(id, user_id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_refresh_tokens_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_tokens_issued_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_refresh_tokens_org_fk","f",true,false,false,true,true,0,true,[11],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_tokens_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_tokens_replaced_by_fkey","f",true,false,false,true,true,0,true,[8],[1],"a","a","s","public","auth_refresh_tokens","FOREIGN KEY (replaced_by) REFERENCES public.auth_refresh_tokens(id)"],["auth_refresh_tokens_token_hash_key","u",true,false,false,true,true,0,true,[4],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_refresh_tokens_token_hash_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_refresh_tokens_user_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"family_id","number":2,"acl_is_null":true},{"acl":null,"name":"user_id","number":3,"acl_is_null":true},{"acl":null,"name":"token_hash","number":4,"acl_is_null":true},{"acl":null,"name":"issued_at","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"used_at","number":7,"acl_is_null":true},{"acl":null,"name":"replaced_by","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"reuse_detected_at","number":10,"acl_is_null":true},{"acl":null,"name":"org_id","number":11,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_webauthn_ceremonies","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[3,"ceremony_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"challenge_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"state_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremonies_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremonies_pkey ON public.auth_webauthn_ceremonies USING btree (id)"],["idx_auth_webauthn_ceremonies_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremonies_user_active ON public.auth_webauthn_ceremonies USING btree (user_id, ceremony_kind, expires_at) WHERE (consumed_at IS NULL)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_restrict_upd","public","auth_bootstrap_credentials",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_cascade_del","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_restrict_del","public","auth_bootstrap_credentials",null]],"constraints":[["auth_webauthn_ceremonies_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_webauthn_ceremonies_ceremony_kind_check","c",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"CHECK ((ceremony_kind = ANY (ARRAY[''registration''::text, ''authentication''::text])))"],["auth_webauthn_ceremonies_ceremony_kind_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL ceremony_kind"],["auth_webauthn_ceremonies_challenge_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL challenge_json"],["auth_webauthn_ceremonies_check","c",true,false,false,false,true,0,true,[6,8],null," "," "," ",null,null,"CHECK ((expires_at > created_at))"],["auth_webauthn_ceremonies_created_at_not_null","n",true,false,false,false,true,0,true,[8],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremonies_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_webauthn_ceremonies_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_ceremonies_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_ceremonies_state_json_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL state_json"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"ceremony_kind","number":3,"acl_is_null":true},{"acl":null,"name":"challenge_json","number":4,"acl_is_null":true},{"acl":null,"name":"state_json","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":7,"acl_is_null":true},{"acl":null,"name":"created_at","number":8,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_webauthn_ceremony_bindings","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"ceremony_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[2,"action_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[3,"object_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"reason_key","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[5,"replay_attempt","pg_catalog","int4",-1,false,false,"","",null,null,null],[6,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremony_bindings_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremony_bindings_pkey ON public.auth_webauthn_ceremony_bindings USING btree (ceremony_id)"],["idx_auth_webauthn_ceremony_bindings_action",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremony_bindings_action ON public.auth_webauthn_ceremony_bindings USING btree (action_kind, object_id, replay_attempt)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null]],"constraints":[["auth_webauthn_ceremony_bindings_action_kind_check","c",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"CHECK ((action_kind = ANY (ARRAY[''APPROVAL_DECISION''::text, ''POLL_VOTE''::text])))"],["auth_webauthn_ceremony_bindings_action_kind_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL action_kind"],["auth_webauthn_ceremony_bindings_ceremony_id_fkey","f",true,false,false,true,true,0,true,[1],[1],"a","c","s","public","auth_webauthn_ceremonies","FOREIGN KEY (ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON DELETE CASCADE"],["auth_webauthn_ceremony_bindings_ceremony_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL ceremony_id"],["auth_webauthn_ceremony_bindings_check","c",true,false,false,false,true,0,true,[2,4],null," "," "," ",null,null,"CHECK ((((action_kind = ''APPROVAL_DECISION''::text) AND (reason_key = ''operations_passkey_approval_decision''::text)) OR ((action_kind = ''POLL_VOTE''::text) AND (reason_key = ''operations_passkey_poll_vote''::text))))"],["auth_webauthn_ceremony_bindings_created_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremony_bindings_object_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL object_id"],["auth_webauthn_ceremony_bindings_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (ceremony_id)"],["auth_webauthn_ceremony_bindings_reason_key_check","c",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"CHECK ((reason_key = ANY (ARRAY[''operations_passkey_approval_decision''::text, ''operations_passkey_poll_vote''::text])))"],["auth_webauthn_ceremony_bindings_reason_key_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL reason_key"],["auth_webauthn_ceremony_bindings_replay_attempt_check","c",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"CHECK (((replay_attempt IS NULL) OR (replay_attempt >= 1)))"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false]],"column_security":[{"acl":null,"name":"ceremony_id","number":1,"acl_is_null":true},{"acl":null,"name":"action_kind","number":2,"acl_is_null":true},{"acl":null,"name":"object_id","number":3,"acl_is_null":true},{"acl":null,"name":"reason_key","number":4,"acl_is_null":true},{"acl":null,"name":"replay_attempt","number":5,"acl_is_null":true},{"acl":null,"name":"created_at","number":6,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_webauthn_credentials","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"credential_id","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"passkey_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[6,"last_used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[7,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_webauthn_credentials_credential_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_credential_id_key ON public.auth_webauthn_credentials USING btree (credential_id)"],["auth_webauthn_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_pkey ON public.auth_webauthn_credentials USING btree (id)"],["idx_auth_webauthn_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_credentials_user ON public.auth_webauthn_credentials USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_webauthn_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_webauthn_credentials_org_immutable BEFORE UPDATE ON public.auth_webauthn_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null]],"constraints":[["auth_webauthn_credentials_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_webauthn_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_credentials_credential_id_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (credential_id)"],["auth_webauthn_credentials_credential_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL credential_id"],["auth_webauthn_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_credentials_org_fk","f",true,false,false,true,true,0,true,[7],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_webauthn_credentials_passkey_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL passkey_json"],["auth_webauthn_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","DELETE",false],["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"credential_id","number":3,"acl_is_null":true},{"acl":null,"name":"passkey_json","number":4,"acl_is_null":true},{"acl":null,"name":"created_at","number":5,"acl_is_null":true},{"acl":null,"name":"last_used_at","number":6,"acl_is_null":true},{"acl":null,"name":"org_id","number":7,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]}]'::jsonb AND s.routines='[{"name":"auth_legacy_audit_append_v1","identity_arguments":"p_id uuid, p_actor uuid, p_action text, p_target_type text, p_target_id text, p_branch_id uuid, p_before_snap jsonb, p_after_snap jsonb, p_trace_id character, p_span_id character, p_occurred_at timestamp with time zone, p_org_id uuid, p_ip text, p_user_agent text, p_auth_method text, p_device text, p_classification_badges text[], p_anomaly boolean, p_reason text","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_id","p_actor","p_action","p_target_type","p_target_id","p_branch_id","p_before_snap","p_after_snap","p_trace_id","p_span_id","p_occurred_at","p_org_id","p_ip","p_user_agent","p_auth_method","p_device","p_classification_badges","p_anomaly","p_reason"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"f4f6716bcafffcdcd1cb094a3d2b0fd538984851de426d73eb5b0e4c2a8e3d83","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false]]},{"name":"auth_legacy_bootstrap_issue_v1","identity_arguments":"p_company uuid, p_subject uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"text","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","p_id","p_token_hash","p_issued_at","p_expires_at"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"400055092e8e5ed2151682a44cb20f55c667b6bfa8d1f5f5e6d5852d25668555","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_bootstrap_seed_v1","identity_arguments":"p_subject uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_subject","p_id","p_token_hash","p_issued_at","p_expires_at"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"dae7ca9e08dcceb6148b560ecc21fc9c3ad66bc8ec7b700137b8e68c8a9ca523","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_cold_start_admin_v1","identity_arguments":"","result":"uuid","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":null,"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"93fd18fda5db1d5f1674a249a1891cf67e3c2b62a3afbe93087a772c86e81667","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_company_lock_v1","identity_arguments":"p_company uuid","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"4bc29fb7d1029e49eadb217b81bb1fc44235603178079e1b8e5010ae0159a8fd","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"auth_legacy_deactivate_credentials_v1","identity_arguments":"p_company uuid, p_subject uuid, p_occurred_at timestamp with time zone","result":"TABLE(revoked_credentials bigint, revoked_families bigint)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","p_occurred_at","revoked_credentials","revoked_families"],"argmodes":["i","i","i","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"6daa76dec864b9cf83ebd98a7124a49eb9a541ed2c4aba0a9844e70cb91d22bc","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_group_passkey_flag_v1","identity_arguments":"p_subject uuid","result":"boolean","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_subject"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"3a8e4eecf8f48f4bcde9cbfd26c838c586a102bf870da95659070dfa9a228988","acl":[["console_credential_owner","console_app","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_purge_company_v1","identity_arguments":"p_company uuid","result":"void","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"3219296a90b6a8e48c08ed72d16a0cd66739d0b187bc4edccacb682612d61ae3","acl":[["console_credential_owner","console_app","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_purge_subjects_v1","identity_arguments":"p_company uuid","result":"uuid[]","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"afc2a82b13a3ac6198d2a8aafef3ea9ea1acc8372a32b72ea866af1401bd375f","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_reset_credentials_v1","identity_arguments":"p_company uuid, p_subject uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"TABLE(deleted_key_id uuid, deleted_credential_id text)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","p_id","p_token_hash","p_issued_at","p_expires_at","deleted_key_id","deleted_credential_id"],"argmodes":["i","i","i","i","i","i","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"f29a2bc7d7cad98b496f0659129648d8306c09b90e0dec7a62acd10aa461cc79","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_bootstrap_replace_v1","identity_arguments":"p_company uuid, p_actor uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"void","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","p_id","p_token_hash","p_issued_at","p_expires_at"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"e022b62f445985d8801137e7eebb7c04746f3247da85458dc9339531c4123de5","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkey_count_v1","identity_arguments":"p_company uuid, p_actor uuid","result":"bigint","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"1f9d74da4666ae6120a4d0710db4a65694138978e10aca0acf3cb8c79c486776","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkey_delete_v1","identity_arguments":"p_company uuid, p_actor uuid, p_key_id uuid","result":"TABLE(outcome text, credential_id text)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","p_key_id","outcome","credential_id"],"argmodes":["i","i","i","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"4570434174614ee74a75f88ef9193184cfcb018956f7b8db8216fe30fc65ed10","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkey_state_v1","identity_arguments":"p_company uuid, p_actor uuid, p_key_id uuid","result":"TABLE(last_used_at timestamp with time zone)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","p_key_id","last_used_at"],"argmodes":["i","i","i","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"4238746c6b34f3028f9fae8da99f647218c0d28f3382f76905d8f626835a5476","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkeys_v1","identity_arguments":"p_company uuid, p_actor uuid","result":"TABLE(id uuid, created_at timestamp with time zone, last_used_at timestamp with time zone)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","id","created_at","last_used_at"],"argmodes":["i","i","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"76d95d0ea27f02c9ea77627345eb3ff11f2ee40ad32efbc4018ee50a1fe1eb93","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_session_context_v1","identity_arguments":"p_company uuid, p_subject uuid","result":"TABLE(display_name text, username text, roles text[], branches uuid[], group_roles text[], authz_subject_version bigint, authz_policy_version bigint, session_generation bigint)","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","display_name","username","roles","branches","group_roles","authz_subject_version","authz_policy_version","session_generation"],"argmodes":["i","i","t","t","t","t","t","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"92f7dfc3df3ad17f6d3f9fbbe8065c4722fac414efb9b1db738909c8652ffbeb","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false]]},{"name":"auth_legacy_user_active_v1","identity_arguments":"p_company uuid, p_subject uuid","result":"boolean","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"6493b7a81d16126abdc41eed831fb61057864e09e373cb0d44eed56655eb7e46","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_user_has_passkey_v1","identity_arguments":"p_company uuid, p_subject uuid","result":"boolean","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"90a86775826a7e5012204c43fde83d08cc20020a0f1181369afe2855e8ea64ab","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"enforce_org_id_immutable","identity_arguments":"","result":"trigger","owner":"console_app","language":"plpgsql","kind":"f","security_definer":false,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":null,"argnames":null,"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"a46ee31ccf7fae026153b8bea7419777f055a4028e9ecfe0632e262a6f41f3ba","acl":[["console_app","PUBLIC","EXECUTE",false],["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_direct_org_children","identity_arguments":"p_id uuid","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"77fde115be5e5de1f1e75f84e86b59cf82496d5dd063cd3b9fe9f6f7ceff6b95","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"ef0d500cef7ee5d283e6d471587bb96bbe1e0d6aa391524cda9acdd4277d4797","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_list_group_accounts","identity_arguments":"p_group_id uuid","result":"TABLE(user_id uuid, display_name text, phone text, tenant_roles text[], is_active boolean, has_passkey boolean, account_status text, org_id uuid, org_slug text, org_name text, group_roles text[], created_at timestamp with time zone)","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_group_id","user_id","display_name","phone","tenant_roles","is_active","has_passkey","account_status","org_id","org_slug","org_name","group_roles","created_at"],"argmodes":["i","t","t","t","t","t","t","t","t","t","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"27f6f2f8fc4f8326bb5274fc1faf03d682cfe5be72000ca6e5653625087ba88b","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"57c4a8b2ad1fe05a9a620d0d45dfaeaaaac24f006c9f6a97937ead5fb48db21a","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_bootstrap_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"c6b61cbb4aae4f8fb1daee70995be892020e5b906c46f0f327a30404ff87e7e7","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"platform_resolve_credential_org","identity_arguments":"p_credential_id text","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_credential_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"58b08df5e3f2a7b171ec80098832bd394d16b3d9f4a0a286639e7f1417808312","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"platform_resolve_token_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"300cbab098531971f5832bba377e0c9d1e14bb94dd4035a0c2e86c3d7d1b5bfb","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]}]'::jsonb
   AND (SELECT count(*)=1 AND bool_and(valid) FROM credential_role)
   AND NOT EXISTS(SELECT 1 FROM routine_records WHERE NOT extra_valid)
   THEN 'account_credentials.finalized'
 ELSE CASE WHEN EXISTS(SELECT 1 FROM relations WHERE relowner=(SELECT oid FROM credential_role)) THEN 'account_credentials.profile_mismatch' ELSE 'account_credentials.legacy_profile_mismatch' END
 END FROM snapshots s;
"""
CREDENTIAL_INSTALL = r"""ALTER TABLE public.auth_bootstrap_credentials DROP CONSTRAINT auth_bootstrap_credentials_user_id_fkey;
ALTER TABLE public.auth_bootstrap_credentials DROP CONSTRAINT auth_bootstrap_credentials_user_same_org_fk;
ALTER TABLE public.auth_bootstrap_credentials ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_bootstrap_credentials;
ALTER TABLE public.auth_bootstrap_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_bootstrap_credentials FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_bootstrap_credentials TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_bootstrap_credentials OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_bootstrap_credentials FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,INSERT,UPDATE,DELETE ON public.auth_bootstrap_credentials TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_bootstrap_credentials TO console_auth_rt;
ALTER TABLE public.auth_device_login_handoffs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_device_login_handoffs FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_device_login_handoffs TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_device_login_handoffs OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_device_login_handoffs FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT ON public.auth_device_login_handoffs TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_device_login_handoffs TO console_auth_rt;
ALTER TABLE public.auth_refresh_token_families DROP CONSTRAINT auth_refresh_token_families_user_id_fkey;
ALTER TABLE public.auth_refresh_token_families DROP CONSTRAINT auth_refresh_token_families_user_same_org_fk;
ALTER TABLE public.auth_refresh_token_families ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_refresh_token_families;
ALTER TABLE public.auth_refresh_token_families ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_refresh_token_families FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_refresh_token_families TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_refresh_token_families OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_refresh_token_families FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,UPDATE,DELETE ON public.auth_refresh_token_families TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_refresh_token_families TO console_auth_rt;
ALTER TABLE public.auth_refresh_tokens DROP CONSTRAINT auth_refresh_tokens_user_id_fkey;
ALTER TABLE public.auth_refresh_tokens DROP CONSTRAINT auth_refresh_tokens_user_same_org_fk;
ALTER TABLE public.auth_refresh_tokens ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_refresh_tokens;
ALTER TABLE public.auth_refresh_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_refresh_tokens FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_refresh_tokens TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_refresh_tokens OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_refresh_tokens FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,UPDATE,DELETE ON public.auth_refresh_tokens TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_refresh_tokens TO console_auth_rt;
ALTER TABLE public.auth_webauthn_ceremonies DROP CONSTRAINT auth_webauthn_ceremonies_user_id_fkey;
ALTER TABLE public.auth_webauthn_ceremonies ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_webauthn_ceremonies FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_webauthn_ceremonies TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_webauthn_ceremonies OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_webauthn_ceremonies FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,UPDATE,DELETE ON public.auth_webauthn_ceremonies TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_webauthn_ceremonies TO console_auth_rt;
ALTER TABLE public.auth_webauthn_ceremony_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_webauthn_ceremony_bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_webauthn_ceremony_bindings TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_webauthn_ceremony_bindings OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_webauthn_ceremony_bindings FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,DELETE ON public.auth_webauthn_ceremony_bindings TO console_credential_owner;
GRANT SELECT,INSERT ON public.auth_webauthn_ceremony_bindings TO console_auth_rt;
ALTER TABLE public.auth_webauthn_credentials DROP CONSTRAINT auth_webauthn_credentials_user_id_fkey;
ALTER TABLE public.auth_webauthn_credentials DROP CONSTRAINT auth_webauthn_credentials_user_same_org_fk;
ALTER TABLE public.auth_webauthn_credentials ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_webauthn_credentials;
ALTER TABLE public.auth_webauthn_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_webauthn_credentials FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_webauthn_credentials TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_webauthn_credentials OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_webauthn_credentials FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,DELETE ON public.auth_webauthn_credentials TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE,DELETE ON public.auth_webauthn_credentials TO console_auth_rt;
ALTER TABLE public.auth_bootstrap_credentials ADD CONSTRAINT auth_bootstrap_credentials_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_refresh_token_families ADD CONSTRAINT auth_refresh_token_families_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_refresh_tokens ADD CONSTRAINT auth_refresh_tokens_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_webauthn_ceremonies ADD CONSTRAINT auth_webauthn_ceremonies_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_webauthn_credentials ADD CONSTRAINT auth_webauthn_credentials_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_target_account_v1 FOREIGN KEY (target_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_approved_account_v1 FOREIGN KEY (approved_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_refresh_token_families ADD CONSTRAINT auth_refresh_token_families_id_user_v1 UNIQUE(id,user_id);
ALTER TABLE public.auth_refresh_tokens DROP CONSTRAINT auth_refresh_tokens_family_id_fkey;
ALTER TABLE public.auth_refresh_tokens ADD CONSTRAINT auth_refresh_tokens_family_subject_v1 FOREIGN KEY (family_id, user_id) REFERENCES public.auth_refresh_token_families(id, user_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_bootstrap_credentials DROP CONSTRAINT auth_bootstrap_credentials_registration_ceremony_id_fkey;
ALTER TABLE public.auth_bootstrap_credentials ADD CONSTRAINT auth_bootstrap_credentials_registration_ceremony_id_fkey FOREIGN KEY (registration_ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_device_login_handoffs DROP CONSTRAINT auth_device_login_handoffs_check1;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_check1 CHECK (((target_org_id IS NULL) OR (target_user_id IS NOT NULL)));
ALTER TABLE public.auth_device_login_handoffs DROP CONSTRAINT auth_device_login_handoffs_check2;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_check2 CHECK ((((approved_at IS NULL) AND (approved_user_id IS NULL) AND (approved_org_id IS NULL)) OR ((approved_at IS NOT NULL) AND (approved_user_id IS NOT NULL))));
CREATE OR REPLACE FUNCTION public.auth_legacy_company_lock_v1(p_company uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF pg_catalog.current_setting('transaction_isolation') <> 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.unsupported_isolation';
    END IF;
    PERFORM o.id FROM public.organizations o WHERE o.id=p_company FOR KEY SHARE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.company_not_found';
    END IF;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_company_lock_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_company_lock_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_company_lock_v1(uuid) TO console_app,console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_user_active_v1(p_company uuid,p_subject uuid) RETURNS boolean
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE active boolean;
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF p_subject IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    SELECT u.is_active INTO active FROM public.users u WHERE u.id=p_subject AND u.org_id=p_company;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.subject_not_found';
    END IF;
    IF active IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    RETURN active;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_user_active_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_user_active_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_user_active_v1(uuid,uuid) TO console_app,console_auth_rt,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkeys_v1(p_company uuid,p_actor uuid) RETURNS TABLE(id uuid,created_at timestamptz,last_used_at timestamptz)
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_actor);
    SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN QUERY SELECT k.id,k.created_at,k.last_used_at
        FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company ORDER BY k.created_at;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_state_v1(p_company uuid,p_actor uuid,p_key_id uuid) RETURNS TABLE(last_used_at timestamptz)
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_actor);
    SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN QUERY SELECT k.last_used_at FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_count_v1(p_company uuid,p_actor uuid) RETURNS bigint
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_actor);
    SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN (SELECT count(*) FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_count_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_count_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_count_v1(uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_user_has_passkey_v1(p_company uuid,p_subject uuid) RETURNS boolean
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_subject);
    SELECT public.account_legacy_fenced_v1(p_subject) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RETURN false;
    END IF;
    RETURN EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_subject AND k.org_id=p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_user_has_passkey_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_user_has_passkey_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_user_has_passkey_v1(uuid,uuid) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_group_passkey_flag_v1(p_subject uuid) RETURNS boolean
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    IF p_subject IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    SELECT public.account_legacy_fenced_v1(p_subject) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN RETURN false; END IF;
    RETURN EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_group_passkey_flag_v1(uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_group_passkey_flag_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_group_passkey_flag_v1(uuid) TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_delete_v1(p_company uuid,p_actor uuid,p_key_id uuid) RETURNS TABLE(outcome text,credential_id text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean; victim text; total bigint;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_actor) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    SELECT k.credential_id INTO victim FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company FOR UPDATE;
    IF NOT FOUND THEN
        RETURN QUERY SELECT 'not_found'::text,NULL::text;
        RETURN;
    END IF;
    SELECT count(*) INTO total FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company;
    IF total <= 1 THEN
        RETURN QUERY SELECT 'last_key'::text,NULL::text;
        RETURN;
    END IF;
    DELETE FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company;
    RETURN QUERY SELECT 'deleted'::text,victim;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_bootstrap_issue_v1(p_company uuid,p_subject uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS text
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='expired'
        WHERE b.user_id=p_subject AND b.org_id=p_company AND b.consumed_at IS NULL
          AND b.revoked_at IS NULL AND b.expires_at<=p_issued_at;
    IF EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject AND k.org_id=p_company) THEN
        RETURN 'has_passkey';
    END IF;
    IF EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b WHERE b.user_id=p_subject
        AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL) THEN
        RETURN 'open_exists';
    END IF;
    INSERT INTO public.auth_bootstrap_credentials(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_subject,p_token_hash,p_issued_at,p_expires_at,p_company);
    RETURN 'issued';
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_reset_credentials_v1(p_company uuid,p_subject uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS TABLE(deleted_key_id uuid,deleted_credential_id text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    RETURN QUERY DELETE FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_subject AND k.org_id=p_company RETURNING k.id,k.credential_id;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='expired'
        WHERE b.user_id=p_subject AND b.org_id=p_company AND b.consumed_at IS NULL
          AND b.revoked_at IS NULL AND b.expires_at<=p_issued_at;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='reset'
        WHERE b.user_id=p_subject AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL;
    INSERT INTO public.auth_bootstrap_credentials(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_subject,p_token_hash,p_issued_at,p_expires_at,p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_bootstrap_replace_v1(p_company uuid,p_actor uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_actor) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='expired'
        WHERE b.user_id=p_actor AND b.org_id=p_company AND b.consumed_at IS NULL
          AND b.revoked_at IS NULL AND b.expires_at<=p_issued_at;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='reset'
        WHERE b.user_id=p_actor AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL;
    INSERT INTO public.auth_bootstrap_credentials(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_actor,p_token_hash,p_issued_at,p_expires_at,p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_deactivate_credentials_v1(p_company uuid,p_subject uuid,p_occurred_at timestamptz) RETURNS TABLE(revoked_credentials bigint,revoked_families bigint)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean; key_count bigint; family_count bigint;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_occurred_at IS NULL OR NOT isfinite(p_occurred_at) THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    DELETE FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject AND k.org_id=p_company;
    GET DIAGNOSTICS key_count = ROW_COUNT;
    UPDATE public.auth_refresh_token_families f SET revoked_at=p_occurred_at,revoked_reason='user_deactivated'
        WHERE f.user_id=p_subject AND f.org_id=p_company AND f.revoked_at IS NULL;
    GET DIAGNOSTICS family_count = ROW_COUNT;
    UPDATE public.auth_refresh_tokens t SET revoked_at=COALESCE(t.revoked_at,p_occurred_at)
        WHERE t.user_id=p_subject AND t.org_id=p_company;
    RETURN QUERY SELECT key_count,family_count;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_cold_start_admin_v1() RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    IF NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid
       IS DISTINCT FROM '00000000-0000-0000-0000-00000000face'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    RETURN (SELECT u.id FROM public.users u
        WHERE u.org_id='00000000-0000-0000-0000-00000000face'::uuid
          AND u.display_name='Cold Start Admin' AND u.roles @> ARRAY['SUPER_ADMIN']::text[]
        ORDER BY u.id LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_cold_start_admin_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_cold_start_admin_v1() FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_cold_start_admin_v1() TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_bootstrap_seed_v1(p_subject uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS uuid
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE
    allowed boolean;
    p_company constant uuid := '00000000-0000-0000-0000-00000000face'::uuid;
    selected uuid;
    opened uuid;
BEGIN
    SELECT public.auth_legacy_cold_start_admin_v1() INTO selected;
    IF selected IS NULL OR selected IS DISTINCT FROM p_subject THEN RETURN NULL; END IF;
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    SELECT public.auth_legacy_cold_start_admin_v1() INTO selected;
    IF selected IS NULL OR selected IS DISTINCT FROM p_subject THEN RETURN NULL; END IF;
    IF EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject AND k.org_id=p_company)
       OR EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b WHERE b.user_id=p_subject
         AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL AND b.expires_at>p_issued_at) THEN
        RETURN NULL;
    END IF;
    INSERT INTO public.auth_bootstrap_credentials AS b(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_subject,p_token_hash,p_issued_at,p_expires_at,p_company)
        ON CONFLICT(token_hash) DO UPDATE SET issued_at=EXCLUDED.issued_at,expires_at=EXCLUDED.expires_at,
            revoked_at=NULL,revoked_reason=NULL,consumed_at=NULL
        WHERE b.user_id=EXCLUDED.user_id AND b.consumed_at IS NULL
          AND (b.revoked_at IS NOT NULL OR b.expires_at<=EXCLUDED.issued_at)
        RETURNING b.id INTO opened;
    RETURN opened;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_session_context_v1(p_company uuid,p_subject uuid) RETURNS TABLE(display_name text,username text,roles text[],branches uuid[],group_roles text[],authz_subject_version bigint,authz_policy_version bigint,session_generation bigint)
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE subject public.users%ROWTYPE; fenced boolean;
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF p_subject IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    SELECT u.* INTO subject FROM public.users u WHERE u.id=p_subject AND u.org_id=p_company;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.subject_not_found';
    END IF;
    IF subject.is_active IS DISTINCT FROM true THEN
        RAISE EXCEPTION USING ERRCODE='28000', MESSAGE='auth_legacy.subject_inactive';
    END IF;
    IF subject.roles IS NULL OR cardinality(subject.roles)=0 THEN
        RAISE EXCEPTION USING ERRCODE='28000', MESSAGE='auth_legacy.subject_has_no_roles';
    END IF;
    SELECT public.account_legacy_fenced_v1(p_subject) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN QUERY SELECT subject.display_name,COALESCE(subject.phone,p_subject::text),subject.roles,
        ARRAY(SELECT b.branch_id FROM public.user_branches b
            WHERE b.user_id=p_subject AND b.org_id=p_company ORDER BY b.branch_id),
        ARRAY(SELECT DISTINCT g.group_role FROM public.group_role_grants g
            WHERE g.user_id=p_subject ORDER BY g.group_role),
        GREATEST(COALESCE(v.version,0),0),
        GREATEST(COALESCE((SELECT pv.version FROM public.policy_versions pv WHERE pv.org_id=p_company),0),0),
        GREATEST(COALESCE(v.session_generation,0),0)
    FROM (SELECT 1) singleton LEFT JOIN public.subject_authz_versions v
        ON v.user_id=p_subject AND v.org_id=p_company;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) TO console_app,console_auth_rt;
CREATE OR REPLACE FUNCTION public.platform_resolve_bootstrap_org(p_token_hash bytea) RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    RETURN (SELECT x.org_id FROM public.auth_bootstrap_credentials x WHERE x.token_hash=p_token_hash LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.platform_resolve_bootstrap_org(bytea) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.platform_resolve_bootstrap_org(bytea) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.platform_resolve_bootstrap_org(bytea) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.platform_resolve_token_org(p_token_hash bytea) RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    RETURN (SELECT x.org_id FROM public.auth_refresh_tokens x WHERE x.token_hash=p_token_hash LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.platform_resolve_token_org(bytea) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.platform_resolve_token_org(bytea) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.platform_resolve_token_org(bytea) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.platform_resolve_credential_org(p_credential_id text) RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    RETURN (SELECT x.org_id FROM public.auth_webauthn_credentials x WHERE x.credential_id=p_credential_id LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.platform_resolve_credential_org(text) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.platform_resolve_credential_org(text) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.platform_resolve_credential_org(text) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_purge_subjects_v1(p_company uuid) RETURNS uuid[]
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE subjects uuid[];
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF pg_catalog.current_setting('transaction_isolation') <> 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.unsupported_isolation';
    END IF;
    PERFORM o.id FROM public.organizations o WHERE o.id=p_company FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.company_not_found';
    END IF;
    SELECT COALESCE(array_agg(locked.id ORDER BY locked.id),ARRAY[]::uuid[]) INTO subjects
        FROM (SELECT u.id FROM public.users u WHERE u.org_id=p_company ORDER BY u.id FOR UPDATE) locked;
    RETURN subjects;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_purge_subjects_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_purge_subjects_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_purge_subjects_v1(uuid) TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_purge_company_v1(p_company uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE
    subjects uuid[];
    subject uuid;
    allowed boolean;
    previous_company text := pg_catalog.current_setting('app.current_org',true);
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    PERFORM pg_catalog.set_config('app.current_org',p_company::text,true);
    SELECT public.auth_legacy_purge_subjects_v1(p_company) INTO subjects;
    FOREACH subject IN ARRAY subjects LOOP
        SELECT public.account_company_deactivation_guard_v1(p_company,subject) INTO allowed;
        IF allowed IS DISTINCT FROM true THEN
            RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy_purge.retained_custody';
        END IF;
    END LOOP;
    -- Account custody cannot be inferred from a retained Company provenance.
    -- Classify every Company-attributed subject before any credential deletion.
    IF EXISTS(SELECT 1 FROM (
        SELECT k.user_id FROM public.auth_webauthn_credentials k WHERE k.org_id=p_company
        UNION ALL SELECT f.user_id FROM public.auth_refresh_token_families f WHERE f.org_id=p_company
        UNION ALL SELECT t.user_id FROM public.auth_refresh_tokens t WHERE t.org_id=p_company
        UNION ALL SELECT b.user_id FROM public.auth_bootstrap_credentials b WHERE b.org_id=p_company
        UNION ALL SELECT h.target_user_id FROM public.auth_device_login_handoffs h WHERE h.target_org_id=p_company
        UNION ALL SELECT h.approved_user_id FROM public.auth_device_login_handoffs h WHERE h.approved_org_id=p_company
    ) attributed WHERE attributed.user_id IS NULL OR NOT (attributed.user_id=ANY(subjects))) THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy_purge.retained_custody';
    END IF;
    DELETE FROM public.auth_bootstrap_credentials b WHERE b.org_id=p_company;
    DELETE FROM public.auth_refresh_tokens t WHERE t.org_id=p_company;
    DELETE FROM public.auth_refresh_token_families f WHERE f.org_id=p_company;
    DELETE FROM public.auth_webauthn_credentials k WHERE k.org_id=p_company;
    DELETE FROM public.auth_webauthn_ceremonies c WHERE c.user_id=ANY(subjects);
    -- The original purge retained global handoff history; preserve it.
    PERFORM pg_catalog.set_config('app.current_org',COALESCE(previous_company,''),true);
EXCEPTION WHEN OTHERS THEN
    PERFORM pg_catalog.set_config('app.current_org',COALESCE(previous_company,''),true);
    RAISE;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_purge_company_v1(uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_purge_company_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_purge_company_v1(uuid) TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_audit_append_v1(p_id uuid,p_actor uuid,p_action text,p_target_type text,p_target_id text,p_branch_id uuid,p_before_snap jsonb,p_after_snap jsonb,p_trace_id char(32),p_span_id char(16),p_occurred_at timestamptz,p_org_id uuid,p_ip text,p_user_agent text,p_auth_method text,p_device text,p_classification_badges text[],p_anomaly boolean,p_reason text) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE
    keys text[];
    expected_keys text[];
    uuid_fields text[] := ARRAY[]::text[];
    bool_fields text[] := ARRAY[]::text[];
    time_fields text[] := ARRAY[]::text[];
    field text;
    target text;
    anonymous boolean := false;
    fenced boolean;
    value jsonb;
    parts bigint[];
    year_value bigint;
    ordinal_max bigint;
BEGIN
    IF p_id IS NULL OR p_action IS NULL OR p_target_type IS NULL OR p_target_id IS NULL
       OR p_before_snap IS NOT NULL OR jsonb_typeof(p_after_snap) IS DISTINCT FROM 'object'
       OR p_occurred_at IS NULL OR NOT isfinite(p_occurred_at)
       OR p_trace_id IS NULL OR p_trace_id::text !~ '^[0-9a-f]{32}$'
       OR p_span_id IS NULL OR p_span_id::text !~ '^[0-9a-f]{16}$'
       OR p_branch_id IS NOT NULL OR p_ip IS NOT NULL OR p_user_agent IS NOT NULL
       OR p_auth_method IS NOT NULL OR p_device IS NOT NULL OR p_classification_badges IS NOT NULL
       OR p_anomaly IS NOT NULL OR p_reason IS NOT NULL THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
    END IF;
    CASE p_action
    WHEN 'auth.passkey.register' THEN
        target := 'auth_webauthn_credential'; expected_keys := ARRAY['credential_id','user_id']; uuid_fields := ARRAY['user_id'];
        IF jsonb_typeof(p_after_snap->'credential_id') IS DISTINCT FROM 'string' OR p_after_snap->>'credential_id'='' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.refresh.issue' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','token_id','user_id','expires_at'];
        uuid_fields := ARRAY['family_id','token_id','user_id']; time_fields := ARRAY['expires_at'];
    WHEN 'auth.refresh' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','used_token_id','replacement_token_id','expires_at'];
        uuid_fields := ARRAY['family_id','used_token_id','replacement_token_id']; time_fields := ARRAY['expires_at'];
    WHEN 'auth.refresh.absolute_ttl_revoked' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','revoked_reason','family_created_at'];
        uuid_fields := ARRAY['family_id']; time_fields := ARRAY['family_created_at'];
        IF p_after_snap->'revoked_reason' IS DISTINCT FROM '"absolute_ttl_exceeded"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.refresh.reuse_detected' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','revoked_reason','reused_token_id'];
        uuid_fields := ARRAY['family_id','reused_token_id'];
        IF p_after_snap->'revoked_reason' IS DISTINCT FROM '"reuse_detected"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.logout' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','revoked_reason']; uuid_fields := ARRAY['family_id'];
        IF p_after_snap->'revoked_reason' IS DISTINCT FROM '"logout"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.otp.redeem' THEN
        target := 'auth_bootstrap_credential'; expected_keys := ARRAY['user_id','requires_passkey_setup'];
        uuid_fields := ARRAY['user_id']; bool_fields := ARRAY['requires_passkey_setup'];
    WHEN 'auth.otp.consume' THEN
        target := 'auth_bootstrap_credential'; expected_keys := ARRAY['user_id']; uuid_fields := ARRAY['user_id'];
    WHEN 'auth.passkey.enroll_handoff_issued' THEN
        target := 'auth_bootstrap_credential'; expected_keys := ARRAY['user_id','expires_at','purpose'];
        uuid_fields := ARRAY['user_id']; time_fields := ARRAY['expires_at'];
        IF p_after_snap->'purpose' IS DISTINCT FROM '"passkey_enrollment_handoff"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.login' THEN
        target := 'users'; expected_keys := ARRAY['passkey_id','refresh_family_id']; uuid_fields := expected_keys;
    WHEN 'auth.otp.signin' THEN
        target := 'users'; expected_keys := ARRAY['refresh_family_id','requires_passkey_setup'];
        uuid_fields := ARRAY['refresh_family_id']; bool_fields := ARRAY['requires_passkey_setup'];
    WHEN 'auth.device_login.approve' THEN
        target := 'users'; expected_keys := ARRAY['handoff_id','passkey_id']; uuid_fields := expected_keys;
    WHEN 'auth.device_login.approve_session' THEN
        target := 'users'; expected_keys := ARRAY['handoff_id','passkey_id']; uuid_fields := ARRAY['handoff_id'];
        IF p_after_snap->'passkey_id' IS DISTINCT FROM 'null'::jsonb THEN uuid_fields := array_append(uuid_fields,'passkey_id'); END IF;
    WHEN 'auth.device_login.consume' THEN
        target := 'users'; expected_keys := ARRAY['handoff_id','passkey_id','refresh_family_id']; uuid_fields := ARRAY['handoff_id','refresh_family_id'];
        IF p_after_snap->'passkey_id' IS DISTINCT FROM 'null'::jsonb THEN uuid_fields := array_append(uuid_fields,'passkey_id'); END IF;
    WHEN 'auth.otp.redeem_failed' THEN
        anonymous := true; target := 'auth_bootstrap_credential'; expected_keys := ARRAY['outcome'];
        IF p_after_snap->'outcome' IS DISTINCT FROM '"rejected"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.device_login.start' THEN
        anonymous := true; target := 'auth_bootstrap_credential'; expected_keys := ARRAY['handoff_id','expires_at'];
        uuid_fields := ARRAY['handoff_id']; time_fields := ARRAY['expires_at'];
    ELSE
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
    END CASE;
    SELECT array_agg(k ORDER BY k) INTO keys FROM jsonb_object_keys(p_after_snap) k;
    IF keys IS DISTINCT FROM ARRAY(SELECT k FROM unnest(expected_keys) k ORDER BY k)
       OR p_target_type IS DISTINCT FROM target THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
    END IF;
    FOREACH field IN ARRAY uuid_fields LOOP
        IF jsonb_typeof(p_after_snap->field) IS DISTINCT FROM 'string'
           OR (p_after_snap->>field) !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END LOOP;
    FOREACH field IN ARRAY bool_fields LOOP
        IF jsonb_typeof(p_after_snap->field) IS DISTINCT FROM 'boolean' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END LOOP;
    FOREACH field IN ARRAY time_fields LOOP
        value := p_after_snap->field;
        IF jsonb_typeof(value) IS DISTINCT FROM 'array' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
        IF jsonb_array_length(value)<>9 OR EXISTS(SELECT 1 FROM jsonb_array_elements(value) part
            WHERE jsonb_typeof(part)<>'number' OR part::text !~ '^-?[0-9]{1,10}$') THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
        SELECT array_agg(part::text::bigint ORDER BY ordinal) INTO parts
            FROM jsonb_array_elements(value) WITH ORDINALITY elements(part,ordinal);
        year_value := parts[1];
        ordinal_max := CASE WHEN year_value%4=0 AND (year_value%100<>0 OR year_value%400=0) THEN 366 ELSE 365 END;
        IF year_value NOT BETWEEN -9999 AND 9999 OR parts[2] NOT BETWEEN 1 AND ordinal_max
           OR parts[3] NOT BETWEEN 0 AND 23 OR parts[4] NOT BETWEEN 0 AND 59 OR parts[5] NOT BETWEEN 0 AND 59
           OR parts[6] NOT BETWEEN 0 AND 999999999 OR parts[7] NOT BETWEEN -23 AND 23
           OR parts[8] NOT BETWEEN -59 AND 59 OR parts[9] NOT BETWEEN -59 AND 59
           OR parts[7]*parts[8]<0 OR parts[7]*parts[9]<0 OR parts[8]*parts[9]<0 THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END LOOP;
    IF anonymous THEN
        IF p_actor IS NOT NULL OR p_org_id IS NOT NULL OR p_target_id<>'redeem' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    ELSE
        IF p_actor IS NULL OR p_org_id IS NULL
           OR p_target_id !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
           OR (target='users' AND p_target_id IS DISTINCT FROM p_actor::text)
           OR (p_after_snap ? 'user_id' AND p_after_snap->>'user_id' IS DISTINCT FROM p_actor::text)
           OR (p_after_snap ? 'family_id' AND p_after_snap->>'family_id' IS DISTINCT FROM p_target_id) THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
        PERFORM public.auth_legacy_user_active_v1(p_org_id,p_actor);
        SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
        IF fenced IS DISTINCT FROM false THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END IF;
    INSERT INTO public.audit_events(id,actor,action,target_type,target_id,branch_id,before_snap,after_snap,
        trace_id,span_id,occurred_at,org_id,ip,user_agent,auth_method,device,classification_badges,anomaly,reason)
    VALUES(p_id,p_actor,p_action,p_target_type,p_target_id,p_branch_id,p_before_snap,p_after_snap,
        p_trace_id,p_span_id,p_occurred_at,p_org_id,p_ip,p_user_agent,p_auth_method,p_device,p_classification_badges,p_anomaly,p_reason);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text) TO console_app,console_auth_rt;
CREATE OR REPLACE FUNCTION public.platform_remove_organization(p_id uuid)
 RETURNS text
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
DECLARE
    sentinel_org CONSTANT UUID := '00000000-0000-0000-0000-00000000face'::uuid;
    org_exists   BOOLEAN;
    has_data     BOOLEAN;
BEGIN
    IF p_id = sentinel_org THEN
        RETURN 'not_found';
    END IF;

    SET LOCAL row_security = off;

    SELECT EXISTS (SELECT 1 FROM organizations WHERE id = p_id) INTO org_exists;
    IF NOT org_exists THEN
        SET LOCAL row_security = on;
        RETURN 'not_found';
    END IF;

    SELECT
        EXISTS (SELECT 1 FROM registry_equipment           WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM work_orders                   WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM registry_sites               WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM registry_customers           WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM inspection_rounds            WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM regular_inspection_schedules WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM sales_listings               WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM customer_inquiries           WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_rental_quotes      WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_purchase_requests  WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_purchase_attachments WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_regular_purchase_prices WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_expense_ledger      WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM equipment_cost_ledger        WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM messenger_threads            WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM location_consents            WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM site_attendance_events       WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM governance_findings          WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM registered_devices rd
                JOIN users u ON u.id = rd.user_id WHERE u.org_id = p_id)
    INTO has_data;

    IF has_data THEN
        SET LOCAL row_security = on;
        RETURN 'blocked_has_data';
    END IF;

    PERFORM public.auth_legacy_purge_company_v1(p_id);

    DELETE FROM user_branches WHERE org_id = p_id;

    PERFORM set_config('app.audit_rehome', 'on', true);
    UPDATE audit_events
    SET org_id    = sentinel_org,
        actor     = NULL,
        branch_id = NULL
    WHERE org_id = p_id;
    PERFORM set_config('app.audit_rehome', 'off', true);

    DELETE FROM users    WHERE org_id = p_id;
    DELETE FROM branches WHERE org_id = p_id;
    DELETE FROM regions  WHERE org_id = p_id;

    DELETE FROM organizations WHERE id = p_id;

    SET LOCAL row_security = on;
    RETURN 'removed';
END;
$function$;
CREATE OR REPLACE FUNCTION public.platform_force_remove_organization(p_id uuid)
 RETURNS text
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
DECLARE
    sentinel_org CONSTANT UUID := '00000000-0000-0000-0000-00000000face'::uuid;
    org_status   TEXT;
BEGIN
    IF p_id = sentinel_org THEN
        RETURN 'not_found';
    END IF;

    SET LOCAL row_security = off;
    PERFORM set_config('app.maintenance_force_remove', 'on', true);

    SELECT status INTO org_status FROM organizations WHERE id = p_id;
    IF NOT FOUND THEN
        SET LOCAL row_security = on;
        RETURN 'not_found';
    END IF;

    IF org_status <> 'ARCHIVED' THEN
        SET LOCAL row_security = on;
        RETURN 'blocked_active';
    END IF;

    PERFORM set_config('app.platform_force_remove_org', 'on', true);
    -- Close direct restrictive tenant edges before deleting employee/user roots.
    -- The catalog query excludes only the explicitly ordered roots below.
    PERFORM platform_force_remove_direct_org_children(p_id);
    PERFORM public.auth_legacy_purge_company_v1(p_id);
    DELETE FROM attendance_direct_import_events  WHERE org_id = p_id;
    DELETE FROM data_import_rows                 WHERE org_id = p_id;
    DELETE FROM data_import_runs                 WHERE org_id = p_id;
    DELETE FROM payroll_draft_lines             WHERE org_id = p_id;
    DELETE FROM annual_leave_obligations        WHERE org_id = p_id;
    DELETE FROM payroll_draft_runs              WHERE org_id = p_id;
    DELETE FROM employee_lifecycle_events       WHERE org_id = p_id;
    UPDATE users SET employee_id = NULL         WHERE org_id = p_id;
    DELETE FROM employees                       WHERE org_id = p_id;
    PERFORM set_config('app.platform_force_remove_org', 'off', true);


    DELETE FROM comms_send_rate                 WHERE org_id = p_id;

    DELETE FROM customer_inquiries              WHERE org_id = p_id;
    DELETE FROM daily_work_plan_items           WHERE org_id = p_id;
    DELETE FROM daily_work_plans                WHERE org_id = p_id;

    DELETE FROM email_attachments               WHERE org_id = p_id;
    DELETE FROM email_messages                  WHERE org_id = p_id;
    DELETE FROM email_threads                   WHERE org_id = p_id;
    DELETE FROM email_folders                   WHERE org_id = p_id;
    DELETE FROM email_accounts                  WHERE org_id = p_id;
    DELETE FROM mailbox_deliveries             WHERE org_id = p_id;
    DELETE FROM mailbox_messages               WHERE org_id = p_id;
    DELETE FROM mailbox_aliases                WHERE org_id = p_id;
    DELETE FROM mailboxes                      WHERE org_id = p_id;
    DELETE FROM mailbox_domains                WHERE org_id = p_id;

    DELETE FROM equipment_maintenance_history_costs WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history_evidence WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history WHERE org_id = p_id;
    DELETE FROM equipment_cost_ledger           WHERE org_id = p_id;
    DELETE FROM equipment_substitutions         WHERE org_id = p_id;
    DELETE FROM excel_export_logs               WHERE org_id = p_id;

    DELETE FROM user_feature_preferences        WHERE org_id = p_id;

    DELETE FROM financial_regular_purchase_prices WHERE org_id = p_id;
    DELETE FROM financial_expense_ledger          WHERE org_id = p_id;
    DELETE FROM financial_purchase_attachments    WHERE org_id = p_id;
    DELETE FROM financial_purchase_request_lines  WHERE org_id = p_id;

    DELETE FROM financial_purchase_history      WHERE org_id = p_id;
    DELETE FROM financial_purchase_requests     WHERE org_id = p_id;
    DELETE FROM financial_rental_quote_lines    WHERE org_id = p_id;
    DELETE FROM financial_rental_quotes         WHERE org_id = p_id;

    DELETE FROM governance_findings             WHERE org_id = p_id;

    PERFORM set_config('app.audit_rehome', 'on', true);
    UPDATE audit_events
    SET org_id    = sentinel_org,
        actor     = NULL,
        branch_id = NULL
    WHERE org_id = p_id;
    PERFORM set_config('app.audit_rehome', 'off', true);

    DELETE FROM inspection_rounds               WHERE org_id = p_id;
    DELETE FROM kpi_exclusions                  WHERE org_id = p_id;

    DELETE FROM location_collection_logs        WHERE org_id = p_id;
    DELETE FROM location_consent_ledger         WHERE org_id = p_id;
    DELETE FROM location_consents               WHERE org_id = p_id;
    DELETE FROM location_pings                  WHERE org_id = p_id;

    DELETE FROM messenger_message_attachments   WHERE org_id = p_id;
    DELETE FROM messenger_read_receipts         WHERE org_id = p_id;
    DELETE FROM messenger_messages              WHERE org_id = p_id;
    DELETE FROM messenger_thread_members        WHERE org_id = p_id;
    DELETE FROM messenger_threads               WHERE org_id = p_id;

    DELETE FROM evidence_media                  WHERE org_id = p_id;

    DELETE FROM offline_sync_requests           WHERE org_id = p_id;

    DELETE FROM outsource_works                 WHERE org_id = p_id;
    DELETE FROM outsource_vendors               WHERE org_id = p_id;

    DELETE FROM p1_dispatch_alerts              WHERE org_id = p_id;
    DELETE FROM p1_dispatch_responses           WHERE org_id = p_id;
    DELETE FROM p1_dispatch_targets             WHERE org_id = p_id;
    DELETE FROM p1_dispatches                   WHERE org_id = p_id;

    DELETE FROM registered_devices              WHERE org_id = p_id;

    DELETE FROM regular_inspection_schedules    WHERE org_id = p_id;

    DELETE FROM sales_listing_media             WHERE org_id = p_id;
    DELETE FROM sales_listings                  WHERE org_id = p_id;

    DELETE FROM site_attendance_events          WHERE org_id = p_id;
    DELETE FROM site_geofence_presence          WHERE org_id = p_id;

    DELETE FROM support_ticket_comments         WHERE org_id = p_id;
    DELETE FROM support_tickets                 WHERE org_id = p_id;

    DELETE FROM target_change_requests          WHERE org_id = p_id;

    DELETE FROM user_branches                   WHERE org_id = p_id;

    DELETE FROM work_diary_drafts               WHERE org_id = p_id;
    DELETE FROM work_order_approval_steps       WHERE org_id = p_id;
    DELETE FROM work_order_assignments          WHERE org_id = p_id;
    DELETE FROM work_order_request_counters     WHERE org_id = p_id;
    DELETE FROM work_order_status_history       WHERE org_id = p_id;
    DELETE FROM work_orders                     WHERE org_id = p_id;

    DELETE FROM registry_equipment              WHERE org_id = p_id;
    DELETE FROM registry_sites                  WHERE org_id = p_id;
    DELETE FROM registry_customers              WHERE org_id = p_id;

    DELETE FROM users    WHERE org_id = p_id;
    DELETE FROM branches WHERE org_id = p_id;
    DELETE FROM regions  WHERE org_id = p_id;

    DELETE FROM organizations WHERE id = p_id;

    SET LOCAL row_security = on;
    RETURN 'removed';
END;
$function$;
CREATE OR REPLACE FUNCTION public.platform_force_remove_direct_org_children(p_id uuid)
 RETURNS void
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
DECLARE
    target RECORD;
BEGIN
    PERFORM public.auth_legacy_purge_company_v1(p_id);
    -- The maintenance-history subtree goes FIRST, before the catalog sweep below can reach any
    -- of the tables it points at.
    --
    -- These three are structurally invisible to that sweep: `equipment_maintenance_history` is
    -- `org_id ... ON DELETE CASCADE` (0193:19) and the sweep admits only 'a' and 'r', while
    -- `_costs` and `_evidence` reach their parents through COMPOSITE foreign keys and the sweep
    -- requires `cardinality(fk.conkey) = 1`. So the sweep deletes `work_orders`,
    -- `registry_equipment`, `evidence_media` and `equipment_cost_ledger` while rows in this
    -- subtree still reference them, and every one of those raises 23001 in turn.
    --
    -- Deleting the subtree here, child-first, removes the whole class in one place. 0196's
    -- hand-ordered block still deletes these three later; by then they are already empty, which
    -- is why this is additive rather than a reordering of that block.
    DELETE FROM equipment_maintenance_history_costs    WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history_evidence WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history          WHERE org_id = p_id;

    FOR target IN
        SELECT child_ns.nspname AS schema_name, child.relname AS relation_name
        FROM pg_catalog.pg_constraint AS fk
        JOIN pg_catalog.pg_class AS child ON child.oid = fk.conrelid
        JOIN pg_catalog.pg_namespace AS child_ns ON child_ns.oid = child.relnamespace
        JOIN pg_catalog.pg_class AS parent ON parent.oid = fk.confrelid
        JOIN pg_catalog.pg_namespace AS parent_ns ON parent_ns.oid = parent.relnamespace
        JOIN pg_catalog.pg_attribute AS child_attr
          ON child_attr.attrelid = child.oid
         AND child_attr.attnum = fk.conkey[1]
         AND NOT child_attr.attisdropped
        WHERE fk.contype = 'f'
          AND fk.confdeltype IN ('a', 'r')
          AND parent_ns.nspname = 'public'
          AND parent.relname = 'organizations'
          AND child_ns.nspname = 'public'
          AND child.relkind IN ('r', 'p')
          -- These roots have specialized ordering: the audit ledger must be
          -- re-homed, and employee/user/branch/region references are released
          -- only after their direct children have been closed by this pass.
          --
          -- equipment_cost_ledger joins them for the same reason and was missing:
          -- its children reach it by COMPOSITE FK and are therefore invisible to
          -- this catalog sweep, so deleting it here raises 23001 before the
          -- hand-ordered child-first block below can run.
          -- These roots have specialized ordering: the audit ledger must be
          -- re-homed, and employee/user/branch/region references are released
          -- only after their direct children have been closed by this pass.
          AND child.relname NOT IN (
              'audit_events', 'employees', 'users', 'branches', 'regions',
              'auth_bootstrap_credentials', 'auth_device_login_handoffs', 'auth_refresh_token_families', 'auth_refresh_tokens', 'auth_webauthn_ceremonies', 'auth_webauthn_ceremony_bindings', 'auth_webauthn_credentials'
          )
          AND cardinality(fk.conkey) = 1
          AND child_attr.attname = 'org_id'
        -- New tenant-facing tables normally reference older roots.  Descending
        -- OID gives those children priority before their direct parents.
        ORDER BY child.oid DESC
    LOOP
        EXECUTE format('DELETE FROM %I.%I WHERE org_id = $1',
                       target.schema_name, target.relation_name)
            USING p_id;
    END LOOP;
END;
$function$;
CREATE OR REPLACE FUNCTION public.platform_list_group_accounts(p_group_id uuid)
 RETURNS TABLE(user_id uuid, display_name text, phone text, tenant_roles text[], is_active boolean, has_passkey boolean, account_status text, org_id uuid, org_slug text, org_name text, group_roles text[], created_at timestamp with time zone)
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
BEGIN
    SET LOCAL row_security = off;

    RETURN QUERY
        SELECT
            u.id AS user_id,
            u.display_name,
            u.phone,
            u.roles AS tenant_roles,
            u.is_active,
            public.auth_legacy_group_passkey_flag_v1(u.id) AS has_passkey,
            CASE
                WHEN NOT u.is_active THEN 'DEACTIVATED'
                WHEN public.auth_legacy_group_passkey_flag_v1(u.id) THEN 'ACTIVE'
                ELSE 'PENDING_SETUP'
            END AS account_status,
            o.id AS org_id,
            o.slug AS org_slug,
            o.name AS org_name,
            array_agg(gr.group_role ORDER BY gr.group_role) AS group_roles,
            u.created_at
        FROM group_role_grants gr
        JOIN group_memberships gm
          ON gm.group_id = gr.group_id
        JOIN users u
          ON u.id = gr.user_id
         AND u.org_id = gm.org_id
        JOIN organizations o
          ON o.id = u.org_id
         AND o.id <> '00000000-0000-0000-0000-00000000face'::uuid
        WHERE gr.group_id = p_group_id
        GROUP BY
            u.id,
            u.display_name,
            u.phone,
            u.roles,
            u.is_active,
            o.id,
            o.slug,
            o.name,
            u.created_at
        ORDER BY o.name ASC, u.display_name ASC, u.id ASC;

    SET LOCAL row_security = on;
EXCEPTION WHEN OTHERS THEN
    SET LOCAL row_security = on;
    RAISE;
END;
$function$;
GRANT EXECUTE ON FUNCTION public.account_legacy_fenced_v1(uuid) TO console_credential_owner,console_app;
GRANT EXECUTE ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) TO console_credential_owner,console_auth_rt;
"""


def credential_generated_files():
    query = CREDENTIAL_STATE_QUERY.strip().removesuffix(';')
    root_query = (ROOT / 'backend/app/src/account_custody_state.sql').read_text().strip().removesuffix(';')
    root_inspect = 'root_state := (\n' + root_query + '\n);'
    inspect = 'state := (\n' + query + '\n);'
    locks = ', '.join('ONLY public.' + name for name in CREDENTIAL_TABLES)
    sql = f"""-- Generated by ops/generate-account-custody.py; one atomic operator statement.
-- Compose after root custody in the same outer transaction. Replay validates
-- exact metadata and preserves all row and catalog bytes.
DO $account_credentials$
DECLARE
    state text;
    root_state text;
BEGIN
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    PERFORM pg_catalog.set_config('lock_timeout','5s',true);
    IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
       OR session_user IN ('console_app','console_rt','console_auth_rt','console_credential_owner','console_account_owner','console_terms_owner') THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account_credentials.operator_identity_mismatch';
    END IF;
    LOCK TABLE ONLY public.organizations IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE ONLY public.users IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE ONLY public.accounts, ONLY public.account_security, ONLY public.account_security_events,
        ONLY public.account_terms_acceptances, ONLY public.account_terms_head, ONLY public.account_terms_release_receipts IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE {locks} IN ACCESS EXCLUSIVE MODE;
    {root_inspect}
    IF root_state IS DISTINCT FROM 'account_custody.finalized' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE=COALESCE(root_state,'account_credentials.root_prerequisite');
    END IF;
    {inspect}
    IF state='account_credentials.finalized' THEN RETURN; END IF;
    IF state IS DISTINCT FROM 'account_credentials.pending' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE=COALESCE(state,'account_credentials.legacy_profile_mismatch');
    END IF;
    -- Root finalization must already have supplied each legacy subject, and no
    -- foreign credential principal may be guessed or synthesized here.
    IF pg_catalog.to_regprocedure('public.account_company_deactivation_guard_v1(uuid,uuid)') IS NULL
       OR EXISTS(SELECT 1 FROM public.users u LEFT JOIN public.accounts a ON a.id=u.id WHERE a.id IS NULL) THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account_credentials.root_prerequisite';
    END IF;
    IF NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_credential_owner') THEN
        CREATE ROLE console_credential_owner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
    END IF;
    GRANT USAGE ON SCHEMA public TO console_credential_owner;
{CREDENTIAL_INSTALL}
    {root_inspect}
    IF root_state IS DISTINCT FROM 'account_custody.finalized' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE=COALESCE(root_state,'account_credentials.root_prerequisite');
    END IF;
    {inspect}
    IF state IS DISTINCT FROM 'account_credentials.finalized' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE=COALESCE(state,'account_credentials.profile_mismatch');
    END IF;
END
$account_credentials$;
"""
    return {'backend/app/src/account_credential_custody_state.sql': CREDENTIAL_STATE_QUERY,
            'ops/postgres-finalize-account-credentials.sql': sql}


def main():
    if sys.argv[1:] not in ([], ['--check']):
        raise SystemExit('usage: generate-account-custody.py [--check]')
    for name, expected in generated_files().items():
        path = ROOT / name
        if sys.argv[1:]:
            if not path.is_file() or path.read_bytes() != expected.encode():
                raise SystemExit('generated Account custody artifact differs: ' + name)
        else:
            path.write_bytes(expected.encode())


if __name__ == '__main__':
    main()
