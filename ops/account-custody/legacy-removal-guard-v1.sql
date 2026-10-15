-- Uninstalled Account-only correction source, revision 1.
-- Apply only after exact predecessor admission under the owning finalizer locks.
-- Historical producers, migrations and profile identities remain unchanged.

CREATE FUNCTION public.account_legacy_actor_mutation_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $actor_guard$
BEGIN
    RAISE EXCEPTION USING ERRCODE='P0001',
        MESSAGE='account_legacy_actor.mutation_denied';
END;
$actor_guard$;
ALTER FUNCTION public.account_legacy_actor_mutation_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_legacy_actor_mutation_guard_v1()
    FROM PUBLIC,console_account_owner,console_app,console_rt,console_auth_rt,
         console_auth_startup,console_terms_owner,console_credential_owner,
         console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;

-- Statement coverage includes no-op and zero-row DML. RI row locks are not DML.
CREATE TRIGGER account_legacy_actor_mutation_guard_v1
    BEFORE UPDATE OR DELETE OR TRUNCATE ON public.company_actors
    FOR EACH STATEMENT EXECUTE FUNCTION public.account_legacy_actor_mutation_guard_v1();
ALTER TABLE public.company_actors ENABLE ALWAYS TRIGGER account_legacy_actor_mutation_guard_v1;

-- Owner reads retain the selected Company scope, including under FORCE RLS.
CREATE POLICY account_legacy_actor_owner_read_v1 ON public.company_actors
    FOR SELECT TO console_account_owner
    USING (org_id=NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid);

-- Install the immutable statement guard before opening either RI key-lock port.
GRANT SELECT(org_id),UPDATE(org_id) ON public.company_actors TO console_account_owner;
GRANT UPDATE(org_id) ON public.auth_webauthn_credentials TO console_credential_owner;
