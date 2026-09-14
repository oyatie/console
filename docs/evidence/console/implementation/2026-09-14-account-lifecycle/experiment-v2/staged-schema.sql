-- EXPERIMENT ONLY: ownership transfer omitted; original ACL revocation retained. NOT SQLx migration evidence.
-- Dormant AS1/design30/BW31 catalog only. No identity rows, authentication
-- fence activation, terms publication, legacy ownership or runtime ACL handoff.
-- Closed registry/evidence semantics and owner-only mutation procedures are
-- required before enrollment; no serving role can use these empty relations.
DO $preconditions$
BEGIN
    IF (SELECT count(*) FROM pg_catalog.pg_roles
        WHERE rolname IN ('console_account_owner', 'console_terms_owner')
          AND NOT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls
          AND NOT rolinherit AND NOT rolcreatedb AND NOT rolcreaterole
          AND NOT rolreplication) <> 2 THEN
        RAISE EXCEPTION 'account_catalog.custody_owner_topology_required';
    END IF;
END
$preconditions$;

CREATE TABLE public.accounts (
    id uuid PRIMARY KEY,
    created_at timestamptz NOT NULL
);

CREATE TABLE public.account_security (
    account_id uuid PRIMARY KEY REFERENCES public.accounts(id) ON DELETE RESTRICT,
    security_state text NOT NULL CHECK (security_state IN (
        'PENDING_ENROLLMENT', 'ACTIVE', 'SECURITY_SUSPENDED', 'RECOVERY_REQUIRED'
    )),
    security_generation bigint NOT NULL CHECK (security_generation >= 1),
    revision bigint NOT NULL CHECK (revision >= 1),
    updated_at timestamptz NOT NULL,
    context_generation bigint NOT NULL CHECK (context_generation >= 1)
);

CREATE TABLE public.account_security_events (
    id uuid PRIMARY KEY,
    account_id uuid NOT NULL REFERENCES public.accounts(id) ON DELETE RESTRICT,
    kind text NOT NULL CHECK (kind IN (
        'ENROLLED', 'CREDENTIAL_ADDED', 'CREDENTIAL_REVOKED', 'SESSION_REVOKED',
        'RECOVERY_REQUIRED', 'RECOVERY_ADMITTED', 'TERMS_ACCEPTED',
        'SECURITY_SUSPENDED', 'SECURITY_RESTORED'
    )),
    occurred_at timestamptz NOT NULL,
    actor_account_id uuid REFERENCES public.accounts(id) ON DELETE RESTRICT,
    session_id uuid,
    evidence_ref jsonb CHECK (jsonb_typeof(evidence_ref) = 'object'),
    payload jsonb NOT NULL,
    UNIQUE (account_id, id),
    CHECK (
        jsonb_typeof(payload) = 'object'
        AND payload ?& ARRAY['kind', 'account_id', 'before_generation',
            'after_generation', 'credential_id', 'session_id', 'evidence']
        AND payload - ARRAY['kind', 'account_id', 'before_generation',
            'after_generation', 'credential_id', 'session_id', 'evidence'] = '{}'::jsonb
        AND payload->'kind' = to_jsonb(kind)
        AND payload->'account_id' = to_jsonb(account_id::text)
        AND jsonb_typeof(payload->'before_generation') = 'string'
        AND payload->>'before_generation' ~ '^[1-9][0-9]{0,18}$'
        AND jsonb_typeof(payload->'after_generation') = 'string'
        AND payload->>'after_generation' ~ '^[1-9][0-9]{0,18}$'
        AND payload->'session_id' = COALESCE(to_jsonb(session_id::text), 'null'::jsonb)
        AND (payload->'credential_id' = 'null'::jsonb OR
            (jsonb_typeof(payload->'credential_id') = 'string' AND
             payload->>'credential_id' ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
        AND jsonb_typeof(payload->'evidence') = 'object'
        AND (evidence_ref IS NULL OR payload->'evidence' = evidence_ref)
    )
);

CREATE TABLE public.account_terms_release_receipts (
    id uuid PRIMARY KEY,
    previous_revision bigint CHECK (previous_revision >= 1),
    revision bigint NOT NULL UNIQUE CHECK (revision >= 1),
    manifest_sha256 bytea NOT NULL CHECK (octet_length(manifest_sha256) = 32),
    approved_release_ref jsonb NOT NULL,
    recorded_at timestamptz NOT NULL,
    approval_bytes bytea NOT NULL CHECK (octet_length(approval_bytes) BETWEEN 1 AND 16384),
    UNIQUE (id, revision, manifest_sha256),
    CHECK ((previous_revision IS NULL AND revision = 1) OR
        (previous_revision IS NOT NULL AND revision > 1 AND previous_revision = revision - 1)),
    CHECK (
        jsonb_typeof(approved_release_ref) = 'object'
        AND approved_release_ref ?& ARRAY['kind', 'approval_sha256']
        AND approved_release_ref - ARRAY['kind', 'approval_sha256'] = '{}'::jsonb
        AND approved_release_ref->'kind' = '"OPERATOR_RELEASE_APPROVAL"'::jsonb
        AND approved_release_ref->'approval_sha256' = to_jsonb(encode(sha256(approval_bytes), 'hex'))
    )
);

CREATE TABLE public.account_terms_head (
    id smallint PRIMARY KEY CHECK (id = 1),
    manifest_sha256 bytea NOT NULL CHECK (octet_length(manifest_sha256) = 32),
    revision bigint NOT NULL CHECK (revision >= 1),
    release_receipt_ref uuid NOT NULL,
    updated_at timestamptz NOT NULL,
    FOREIGN KEY (release_receipt_ref, revision, manifest_sha256)
        REFERENCES public.account_terms_release_receipts(id, revision, manifest_sha256)
        ON DELETE RESTRICT
);

CREATE TABLE public.account_terms_acceptances (
    account_id uuid NOT NULL REFERENCES public.accounts(id) ON DELETE RESTRICT,
    terms_kind text NOT NULL CHECK (octet_length(terms_kind) BETWEEN 1 AND 1024 AND btrim(terms_kind) <> ''),
    terms_version text NOT NULL CHECK (octet_length(terms_version) BETWEEN 1 AND 1024 AND btrim(terms_version) <> ''),
    content_sha256 bytea NOT NULL CHECK (octet_length(content_sha256) = 32),
    accepted_at timestamptz NOT NULL,
    security_event_id uuid NOT NULL,
    terms_release_receipt_id uuid,
    terms_release_revision bigint CHECK (terms_release_revision >= 1),
    terms_manifest_sha256 bytea CHECK (octet_length(terms_manifest_sha256) = 32),
    PRIMARY KEY (account_id, terms_kind, terms_version),
    FOREIGN KEY (account_id, security_event_id)
        REFERENCES public.account_security_events(account_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (terms_release_receipt_id, terms_release_revision, terms_manifest_sha256)
        REFERENCES public.account_terms_release_receipts(id, revision, manifest_sha256)
        MATCH FULL ON DELETE RESTRICT,
    CHECK (terms_release_receipt_id IS NULL OR
        (terms_version = encode(terms_manifest_sha256, 'hex') AND terms_kind ~ '^[a-z][a-z0-9_.-]{0,63}$'))
);

-- New relations inherit 0031's business default grants before ownership moves.
-- Remove every explicit/default table ACL, including unexpected grantees. No
-- existing auth table or default-privilege policy is changed here.
DO $custody$
DECLARE
    relation_name text;
    target_owner text;
    grantee_name text;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY['accounts', 'account_security',
        'account_security_events', 'account_terms_acceptances',
        'account_terms_head', 'account_terms_release_receipts']
    LOOP
        target_owner := CASE WHEN relation_name IN ('account_terms_head', 'account_terms_release_receipts')
            THEN 'console_terms_owner' ELSE 'console_account_owner' END;
        EXECUTE format('REVOKE ALL ON TABLE public.%I FROM PUBLIC', relation_name);
        FOR grantee_name IN
            SELECT DISTINCT role.rolname FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace ns ON ns.oid = relation.relnamespace
            CROSS JOIN LATERAL pg_catalog.aclexplode(relation.relacl) acl
            JOIN pg_catalog.pg_roles role ON role.oid = acl.grantee
            WHERE ns.nspname = 'public' AND relation.relname = relation_name
        LOOP
            EXECUTE format('REVOKE ALL ON TABLE public.%I FROM %I', relation_name, grantee_name);
        END LOOP;

    END LOOP;
END
$custody$;
