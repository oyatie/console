-- Prospective retained source facts; installed only by the locked custody finalizer.
CREATE FUNCTION public.auth_legacy_platform_source_material_v1(p_subject uuid,p_family uuid)
RETURNS TABLE(roles text[],family_id uuid,family_user_id uuid,family_org_id uuid,
    family_protocol text,family_created_at timestamptz,family_revoked_at timestamptz,
    family_account_security_generation bigint,family_auth_time timestamptz,family_assurance text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    home constant uuid := '00000000-0000-0000-0000-00000000face';
    current_roles text[];
    guarded boolean;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy.unsupported_isolation';
    END IF;
    IF home IS DISTINCT FROM NULLIF(current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF p_subject IS NULL OR p_subject='00000000-0000-0000-0000-000000000000'::uuid
        OR p_family='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='auth_legacy_platform.invalid_identity';
    END IF;
    PERFORM public.auth_legacy_company_lock_v1(home);
    SELECT public.account_company_deactivation_guard_v1(home,p_subject) INTO guarded;
    IF guarded IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_platform.invalid_guard';
    END IF;
    IF NOT guarded THEN
        RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='auth_legacy_platform.source_fenced';
    END IF;
    SELECT public.auth_legacy_user_active_v1(home,p_subject) INTO guarded;
    IF guarded IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_platform.invalid_guard';
    END IF;
    IF NOT guarded THEN
        RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='auth_legacy_platform.source_inactive';
    END IF;
    SELECT c.roles INTO STRICT current_roles FROM public.auth_legacy_session_context_v1(home,p_subject) c;
    IF current_roles IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_platform.invalid_roles';
    END IF;
    IF cardinality(current_roles)=0 THEN
        RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='auth_legacy.subject_has_no_roles';
    END IF;
    IF p_family IS NULL THEN
        RETURN QUERY SELECT current_roles,NULL::uuid,NULL::uuid,NULL::uuid,NULL::text,
            NULL::timestamptz,NULL::timestamptz,NULL::bigint,NULL::timestamptz,NULL::text;
    ELSE
        RETURN QUERY SELECT current_roles,f.id,f.user_id,f.org_id,f.protocol,f.created_at,
            f.revoked_at,f.account_security_generation,f.auth_time,f.assurance
            FROM public.auth_refresh_token_families f WHERE f.id=p_family FOR SHARE OF f;
        IF NOT FOUND THEN
            RAISE EXCEPTION USING ERRCODE='P0002',MESSAGE='auth_legacy_platform.family_not_found';
        END IF;
    END IF;
END
$body$;
ALTER FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid)
    FROM PUBLIC,console_app,console_rt,console_auth_rt,console_auth_startup,
        console_account_owner,console_terms_owner,console_credential_owner,
        console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid)
    TO console_credential_owner,console_rt;
GRANT EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) TO console_credential_owner;
