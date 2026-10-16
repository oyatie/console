-- Derived from the original bound-registration owner verifier. Its v1 body
-- and serving ACL remain unchanged. No serving EXECUTE grant on this helper.
CREATE FUNCTION public.native_company_policy_registration_custody_v1(p_account uuid)
RETURNS TABLE(manifest_sha256 bytea, terms_kind text, content_sha256 bytea, accepted_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    control record;
    enrolled record;
    first_terms record;
    first_acceptance record;
    item record;
    birth_at timestamptz;
    key_id text;
    referenced_event uuid;
    terms_evidence jsonb;
    enrolled_evidence jsonb;
    batch_size bigint;
    seen_kinds text[] := ARRAY[]::text[];
BEGIN
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
    SELECT a.created_at INTO STRICT birth_at FROM public.accounts a WHERE a.id=p_account;
    -- This private integrity-only successor does not establish eligibility.
    -- Callers separately require current ACTIVE state and discovery authority.
    BEGIN
    SELECT e.id,e.account_id,e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        INTO STRICT enrolled FROM public.account_security_events e
        WHERE e.account_id=p_account AND e.kind='ENROLLED';
    key_id := enrolled.payload->>'credential_id';
    IF enrolled.actor_account_id IS DISTINCT FROM p_account OR enrolled.session_id IS NULL
        OR enrolled.session_id='00000000-0000-0000-0000-000000000000'::uuid
        OR key_id IS NULL OR key_id !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        OR key_id='00000000-0000-0000-0000-000000000000'
        OR NOT COALESCE((enrolled.evidence_ref->>'event_id') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',false)
        OR enrolled.occurred_at<birth_at OR enrolled.occurred_at>=birth_at+interval '5 minutes'
        OR enrolled.occurred_at>clock_timestamp() THEN
        RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
    END IF;
    referenced_event := (enrolled.evidence_ref->>'event_id')::uuid;
    IF referenced_event='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
    END IF;
    enrolled_evidence := jsonb_build_object('kind','ACCOUNT_SECURITY_EVENT',
        'account_id',p_account::text,'event_id',referenced_event::text);
    IF enrolled.evidence_ref IS DISTINCT FROM enrolled_evidence
        OR enrolled.payload IS DISTINCT FROM jsonb_build_object('kind','ENROLLED','account_id',p_account::text,
            'before_generation','1','after_generation','1','credential_id',key_id,
            'session_id',enrolled.session_id::text,'evidence',enrolled_evidence) THEN
        RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
    END IF;
    SELECT e.id,e.account_id,e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        INTO STRICT first_terms FROM public.account_security_events e
        WHERE e.account_id=p_account AND e.id=referenced_event AND e.kind='TERMS_ACCEPTED';
    SELECT a.account_id,a.terms_kind,a.terms_version,a.content_sha256,a.accepted_at,a.security_event_id,
        a.terms_release_receipt_id,a.terms_release_revision,a.terms_manifest_sha256
        INTO STRICT first_acceptance FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.security_event_id=first_terms.id;
    IF first_acceptance.terms_release_receipt_id IS NULL
        OR first_acceptance.terms_release_receipt_id='00000000-0000-0000-0000-000000000000'::uuid
        OR first_acceptance.terms_release_revision IS NULL OR first_acceptance.terms_release_revision<1
        OR first_acceptance.terms_manifest_sha256 IS NULL OR octet_length(first_acceptance.terms_manifest_sha256)<>32
        OR first_acceptance.accepted_at IS DISTINCT FROM enrolled.occurred_at
        OR NOT EXISTS(SELECT 1 FROM public.account_terms_release_receipts r
            WHERE r.id=first_acceptance.terms_release_receipt_id
              AND r.revision=first_acceptance.terms_release_revision
              AND r.manifest_sha256=first_acceptance.terms_manifest_sha256)
        OR EXISTS(SELECT 1 FROM public.account_terms_acceptances a
            WHERE a.account_id=p_account AND a.accepted_at<enrolled.occurred_at) THEN
        RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
    END IF;
    terms_evidence := jsonb_build_object('kind','ACCOUNT_TERMS_RELEASE',
        'receipt_id',first_acceptance.terms_release_receipt_id::text,
        'revision',first_acceptance.terms_release_revision::text,
        'manifest_sha256',encode(first_acceptance.terms_manifest_sha256,'hex'));
    SELECT count(*) INTO batch_size FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at;
    IF batch_size NOT BETWEEN 1 AND 8
        OR (SELECT count(DISTINCT a.security_event_id) FROM public.account_terms_acceptances a
            WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at)<>batch_size
        OR (SELECT count(*) FROM public.account_security_events e
            WHERE e.account_id=p_account AND e.kind='TERMS_ACCEPTED'
              AND e.occurred_at=enrolled.occurred_at)<>batch_size THEN
        RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
    END IF;
    FOR item IN SELECT a.terms_kind,a.terms_version,a.content_sha256,a.accepted_at,
        a.terms_release_receipt_id,a.terms_release_revision,a.terms_manifest_sha256,
        e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        FROM public.account_terms_acceptances a
        LEFT JOIN public.account_security_events e ON e.account_id=a.account_id AND e.id=a.security_event_id
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at
    LOOP
        IF item.terms_kind IS NULL OR item.terms_kind !~ '^[a-z][a-z0-9_.-]{0,63}$'
            OR item.terms_kind=ANY(seen_kinds)
            OR item.terms_version IS DISTINCT FROM encode(first_acceptance.terms_manifest_sha256,'hex')
            OR item.content_sha256 IS NULL OR octet_length(item.content_sha256)<>32
            OR item.terms_release_receipt_id IS DISTINCT FROM first_acceptance.terms_release_receipt_id
            OR item.terms_release_revision IS DISTINCT FROM first_acceptance.terms_release_revision
            OR item.terms_manifest_sha256 IS DISTINCT FROM first_acceptance.terms_manifest_sha256
            OR item.kind IS DISTINCT FROM 'TERMS_ACCEPTED'
            OR item.occurred_at IS DISTINCT FROM enrolled.occurred_at
            OR item.actor_account_id IS DISTINCT FROM p_account
            OR item.session_id IS DISTINCT FROM enrolled.session_id
            OR item.evidence_ref IS DISTINCT FROM terms_evidence
            OR item.payload IS DISTINCT FROM jsonb_build_object('kind','TERMS_ACCEPTED','account_id',p_account::text,
                'before_generation','1','after_generation','1','credential_id',key_id,
                'session_id',enrolled.session_id::text,'evidence',terms_evidence) THEN
            RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
        END IF;
        seen_kinds := array_append(seen_kinds,item.terms_kind);
    END LOOP;
    RETURN QUERY SELECT a.terms_manifest_sha256,a.terms_kind,a.content_sha256,a.accepted_at
        FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at
        ORDER BY a.terms_kind;
    EXCEPTION WHEN no_data_found OR too_many_rows THEN
        RAISE EXCEPTION 'native_company_policy.registration_custody_invalid';
    END;
END
$body$;
ALTER FUNCTION public.native_company_policy_registration_custody_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_registration_custody_v1(uuid) FROM PUBLIC;
