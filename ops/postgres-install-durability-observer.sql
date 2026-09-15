-- Optional dedicated-cluster capability, installed by the authenticated operator.
-- One atomic statement shared by the operator and disposable SQLx fixtures.
-- Existing partial or drifted state is refused, never repaired. The transport
-- must set statement_timeout before this DO and serialize operator changes.
DO $durability_install$
DECLARE
    observer oid;
    runtime_role oid;
    function_count bigint;
    valid boolean;
BEGIN
    IF session_user <> current_user
       OR NOT (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user)
       OR session_user IN ('console_app','console_rt','console_auth_rt',
          'console_leave_cmd','console_leave_definer','console_ontology_cmd',
          'console_ontology_writer','console_platform_force_cmd',
          'console_account_owner','console_terms_owner','console_durability_observer') THEN
        RAISE EXCEPTION 'durability_observer.operator_identity_mismatch';
    END IF;
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    SELECT oid INTO runtime_role FROM pg_catalog.pg_roles WHERE rolname='console_rt';
    IF runtime_role IS NULL THEN
        RAISE EXCEPTION 'durability_observer.runtime_role_missing';
    END IF;
    SELECT oid INTO observer FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer';
    SELECT count(*) INTO function_count FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
     WHERE n.nspname='public' AND p.proname='console_durability_observation_v1';
    IF observer IS NULL AND function_count=0 THEN
        CREATE ROLE console_durability_observer NOLOGIN NOSUPERUSER NOBYPASSRLS
            INHERIT NOCREATEDB NOCREATEROLE NOREPLICATION CONNECTION LIMIT -1;
        GRANT pg_read_all_stats TO console_durability_observer
            WITH ADMIN FALSE, INHERIT TRUE, SET FALSE;
        -- PostgreSQL18 exposes this native function to PUBLIC by default.
        -- Close that known initial grant atomically; existing drift is refused.
        REVOKE EXECUTE ON FUNCTION pg_catalog.pg_control_system() FROM PUBLIC;
        GRANT EXECUTE ON FUNCTION pg_catalog.pg_control_system()
            TO console_durability_observer;
CREATE FUNCTION public.console_durability_observation_v1(
    expected_slot name,
    expected_replication_role oid
)
RETURNS TABLE (
    primary_system_id text,
    primary_started_at timestamptz,
    primary_in_recovery boolean,
    observer_pid integer,
    observer_started_at timestamptz,
    slot_name text,
    replication_role_oid oid,
    replication_role_name text,
    sender_pid integer,
    sender_started_at timestamptz,
    application_name text,
    sender_state text,
    sender_client_addr text,
    sender_ssl boolean,
    sender_client_dn text,
    sender_client_serial text,
    sender_issuer_dn text,
    flush_lsn pg_lsn,
    replay_lsn pg_lsn
)
LANGUAGE sql
VOLATILE
PARALLEL UNSAFE
SECURITY DEFINER
SET search_path = pg_catalog
AS $observer$
    SELECT c.system_identifier::text,
           pg_catalog.pg_postmaster_start_time(),
           pg_catalog.pg_is_in_recovery(),
           pg_catalog.pg_backend_pid(),
           own.backend_start,
           slots.slot_name::text,
           sender.usesysid,
           sender.usename::text,
           sender.pid,
           sender.backend_start,
           sender.application_name,
           sender.state,
           pg_catalog.host(sender.client_addr),
           tls.ssl,
           tls.client_dn,
           tls.client_serial::text,
           tls.issuer_dn,
           sender.flush_lsn,
           sender.replay_lsn
      FROM pg_catalog.pg_replication_slots AS slots
      JOIN pg_catalog.pg_stat_replication AS sender
        ON sender.pid = slots.active_pid
      JOIN pg_catalog.pg_stat_ssl AS tls ON tls.pid = sender.pid
     CROSS JOIN pg_catalog.pg_control_system() AS c
      JOIN pg_catalog.pg_stat_activity AS own
        ON own.pid = pg_catalog.pg_backend_pid()
     WHERE slots.slot_name = expected_slot
       AND slots.slot_type = 'physical'
       AND slots.active
       AND sender.usesysid = expected_replication_role
$observer$;

-- Exact owner role is created/certified by the privileged topology operator.
ALTER FUNCTION public.console_durability_observation_v1(name, oid)
    OWNER TO console_durability_observer;
REVOKE ALL ON FUNCTION public.console_durability_observation_v1(name, oid)
    FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.console_durability_observation_v1(name, oid)
    TO console_rt;

        SELECT oid INTO observer FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer';
    ELSIF observer IS NULL OR function_count<>1 THEN
        RAISE EXCEPTION 'durability_observer.partial_installation';
    END IF;

    -- Certify the recipient after closing the known initial PUBLIC grant.
    -- Existing elevated access is drift; never strip or silently repair it.
    SELECT r.rolcanlogin AND NOT r.rolsuper AND NOT r.rolbypassrls
       AND NOT r.rolcreatedb AND NOT r.rolcreaterole AND NOT r.rolreplication
       AND NOT pg_catalog.pg_has_role(r.oid,'pg_read_all_stats','MEMBER')
       AND NOT pg_catalog.pg_has_role(r.oid,'pg_monitor','MEMBER')
       AND NOT pg_catalog.pg_has_role(r.oid,observer,'MEMBER')
       AND NOT pg_catalog.has_function_privilege(r.oid,'pg_catalog.pg_control_system()','EXECUTE')
      INTO valid FROM pg_catalog.pg_roles r WHERE r.oid=runtime_role;
    IF valid IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'durability_observer.runtime_profile_mismatch';
    END IF;

    -- Certification is identical after an initial install and on a no-op replay.
    -- A failed postcondition rolls back every new role, membership and ACL.
    SELECT NOT r.rolcanlogin AND NOT r.rolsuper AND NOT r.rolbypassrls
       AND r.rolinherit AND NOT r.rolcreatedb AND NOT r.rolcreaterole AND NOT r.rolreplication
       AND r.rolconnlimit=-1 AND r.rolpassword IS NULL AND r.rolvaliduntil IS NULL
       AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_db_role_setting s WHERE s.setrole=r.oid)
       AND (SELECT count(*)=1 AND bool_and(m.roleid='pg_read_all_stats'::regrole
            AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option)
            FROM pg_catalog.pg_auth_members m WHERE m.member=r.oid)
       AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_auth_members m WHERE m.roleid=r.oid)
      INTO valid FROM pg_catalog.pg_authid r WHERE r.oid=observer;
    IF valid IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'durability_observer.role_profile_mismatch';
    END IF;

    SELECT count(*)=1 AND bool_and(
           p.oid='pg_catalog.pg_control_system()'::regprocedure
           AND a.privilege_type='EXECUTE' AND NOT a.is_grantable)
      INTO valid
      FROM pg_catalog.pg_proc p
      CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a
     WHERE a.grantee=observer AND p.proowner<>observer;
    IF valid IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'durability_observer.native_grant_mismatch';
    END IF;

    SELECT count(*)=1 AND bool_and(
        p.proowner=observer
        AND p.prolang=(SELECT oid FROM pg_catalog.pg_language WHERE lanname='sql')
        AND p.prokind='f' AND p.provolatile='v' AND p.proparallel='u'
        AND p.prosecdef AND NOT p.proleakproof AND NOT p.proisstrict AND p.proretset
        AND p.prorettype='record'::regtype AND p.pronargs=2 AND p.pronargdefaults=0
        AND p.proargtypes='19 26'::oidvector AND p.proargdefaults IS NULL
        AND p.prosqlbody IS NULL AND p.prosupport=0 AND p.protrftypes IS NULL
        AND p.proallargtypes=ARRAY[19,26,25,1184,16,23,1184,25,26,25,23,1184,25,25,25,16,25,25,25,3220,3220]::oid[]
        AND p.proargmodes=ARRAY['i','i','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t','t']::"char"[]
        AND p.proargnames=ARRAY['expected_slot','expected_replication_role','primary_system_id',
            'primary_started_at','primary_in_recovery','observer_pid','observer_started_at','slot_name',
            'replication_role_oid','replication_role_name','sender_pid','sender_started_at',
            'application_name','sender_state','sender_client_addr','sender_ssl','sender_client_dn',
            'sender_client_serial','sender_issuer_dn','flush_lsn','replay_lsn']::text[]
        AND p.proconfig=ARRAY['search_path=pg_catalog']::text[]
        AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=
            '855b83fbb3581ae7b052fd9863e0565acf8608800ac7890fc56300adbad9b01b'
        AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
        AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
             a.grantor=observer AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
             AND a.grantee IN (observer,runtime_role)) FROM pg_catalog.aclexplode(p.proacl) a))
      INTO valid FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
     WHERE n.nspname='public' AND p.proname='console_durability_observation_v1';
    IF valid IS DISTINCT FROM true THEN
        RAISE EXCEPTION 'durability_observer.function_profile_mismatch';
    END IF;
END
$durability_install$;
