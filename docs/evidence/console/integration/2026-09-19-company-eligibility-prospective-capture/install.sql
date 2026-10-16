-- One prospective read owner. Apply only after exact predecessor validation
-- inside the existing custody finalizer transaction; never standalone rollout.
CREATE FUNCTION public.account_company_setup_eligibility_v1(p_account uuid)
RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    control record;
    head public.deployment_operator_head%ROWTYPE;
    receipt public.deployment_operator_receipts%ROWTYPE;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
    BEGIN
        SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
    EXCEPTION WHEN no_data_found THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END;
    IF control.security_state IS DISTINCT FROM 'ACTIVE' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
    IF control.security_generation IS NULL OR control.security_generation < 1
        OR control.revision IS NULL OR control.revision < 1
        OR control.context_generation IS NULL OR control.context_generation < 1 THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;

    -- Account SHARE also serializes an absent head against same-Account genesis.
    -- Keep the selected head SHARE until the caller's transaction ends.
    SELECT h.* INTO head FROM public.deployment_operator_head h
        WHERE h.singleton=1 AND h.account_id=p_account FOR SHARE OF h;
    IF NOT FOUND THEN RETURN false; END IF;

    -- Separate SPI statement gets fresh visibility after a possible head wait.
    SELECT r.* INTO receipt FROM public.deployment_operator_receipts r
        WHERE r.receipt_id=head.receipt_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;
    IF (receipt.receipt_id,receipt.revision,receipt.account_id,
            receipt.system_identifier,receipt.database_name,receipt.database_oid)
        IS DISTINCT FROM
        (head.receipt_id,head.revision,p_account,
            head.system_identifier,head.database_name,head.database_oid)
        OR head.system_identifier IS DISTINCT FROM
            (SELECT c.system_identifier::text FROM pg_catalog.pg_control_system() c)
        OR head.database_name IS DISTINCT FROM pg_catalog.current_database()::text
        OR head.database_oid IS DISTINCT FROM
            (SELECT d.oid::bigint FROM pg_catalog.pg_database d
                WHERE d.datname=pg_catalog.current_database())
        OR receipt.receipt_id='00000000-0000-0000-0000-000000000000'::uuid
        OR receipt.command_id='00000000-0000-0000-0000-000000000000'::uuid
        OR receipt.revision < 1 OR receipt.expected_revision < 0
        OR receipt.expected_revision IS DISTINCT FROM receipt.revision-1 THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;
    IF receipt.kind='DESIGNATE'
        AND receipt.revision=1 AND receipt.expected_revision=0
        AND receipt.expected_security_generation IS NOT NULL
        AND receipt.expected_security_generation > 0 AND receipt.reason IS NULL THEN
        -- Admission generation is immutable provenance, not a live-session fence.
        RETURN true;
    END IF;
    IF receipt.kind='REVOKE' AND receipt.revision > 1
        AND receipt.expected_security_generation IS NULL
        AND receipt.reason IS NOT NULL
        AND pg_catalog.octet_length(receipt.reason) BETWEEN 1 AND 512
        AND pg_catalog.length(pg_catalog.btrim(receipt.reason,E' \t\n\r\f\013')) > 0 THEN
        RETURN false;
    END IF;
    RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
END
$body$;
ALTER FUNCTION public.account_company_setup_eligibility_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_company_setup_eligibility_v1(uuid)
    FROM PUBLIC,console_app,console_rt,console_auth_rt,console_auth_startup,
        console_account_owner,console_terms_owner,console_credential_owner,
        console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
GRANT EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid)
    TO console_account_owner,console_auth_rt;
