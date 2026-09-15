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
                       ('account_legacy_user_id_immutable_v1', USER_ID_GUARD_BODY)):
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
    expected_fence_body text := $fence_body${FENCE_BODY}$fence_body$;
    expected_guard_body text := $guard_body${GUARD_BODY}$guard_body$;
    expected_current_body text := $current_body${CURRENT_BODY}$current_body$;
    expected_root_guard_body text := $root_guard_body${ROOT_GUARD_BODY}$root_guard_body$;
    expected_root_bridge_body text := $root_bridge_body${ROOT_BRIDGE_BODY}$root_bridge_body$;
    expected_user_id_guard_body text := $user_id_guard_body${USER_ID_GUARD_BODY}$user_id_guard_body$;
BEGIN
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    PERFORM pg_catalog.set_config('lock_timeout','5s',true);
    IF session_user<>current_user
       OR NOT (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user)
       OR session_user IN ('console_app','console_rt','console_auth_rt',
          'console_leave_cmd','console_leave_definer','console_ontology_cmd',
          'console_ontology_writer','console_platform_force_cmd',
          'console_account_owner','console_terms_owner') THEN
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
    IF state='account_custody.finalized' THEN
        -- Serving admission is metadata-only. The privileged operator alone
        -- detects corrupt missing roots and refuses rather than repairing them.
        IF EXISTS(SELECT 1 FROM public.users u LEFT JOIN public.accounts a ON a.id=u.id WHERE a.id IS NULL) THEN
            RAISE EXCEPTION USING MESSAGE='account_root_transition.missing_root', ERRCODE='P0001';
        END IF;
        RETURN;
    END IF;
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
            'ops/account-custody-migrations.sha384': ledger}


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
