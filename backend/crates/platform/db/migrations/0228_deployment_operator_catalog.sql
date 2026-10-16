-- Empty operational catalogs only. The privileged, profile-checked custody
-- transition attaches the Account FK, ownership, guards and function grants.
-- Historical numbered migrations 1..227 remain byte-for-byte unchanged.
CREATE TABLE public.deployment_operator_receipts (
    receipt_id uuid PRIMARY KEY,
    command_id uuid NOT NULL UNIQUE,
    system_identifier text NOT NULL,
    database_name text NOT NULL,
    database_oid bigint NOT NULL,
    account_id uuid NOT NULL,
    kind text NOT NULL,
    expected_revision bigint NOT NULL,
    revision bigint NOT NULL,
    expected_security_generation bigint,
    reason text,
    recorded_at timestamptz NOT NULL,
    UNIQUE (receipt_id, revision, account_id, system_identifier, database_name, database_oid),
    CONSTRAINT deployment_operator_receipts_kind_check CHECK (kind IN ('DESIGNATE', 'REVOKE')),
    CONSTRAINT deployment_operator_receipts_expected_revision_check CHECK (expected_revision >= 0),
    CONSTRAINT deployment_operator_receipts_revision_check CHECK (revision > 0),
    CONSTRAINT deployment_operator_receipts_generation_kind_check CHECK (
        CASE kind
            WHEN 'DESIGNATE' THEN expected_security_generation IS NOT NULL AND expected_security_generation > 0
            WHEN 'REVOKE' THEN expected_security_generation IS NULL
            ELSE true
        END
    ),
    CONSTRAINT deployment_operator_receipts_reason_kind_check CHECK (
        CASE kind
            WHEN 'DESIGNATE' THEN reason IS NULL
            WHEN 'REVOKE' THEN reason IS NOT NULL AND pg_catalog.octet_length(reason) BETWEEN 1 AND 512
                AND pg_catalog.length(pg_catalog.btrim(reason, E' \t\n\r\f\013')) > 0
            ELSE true
        END
    )
);

CREATE TABLE public.deployment_operator_head (
    singleton smallint PRIMARY KEY,
    system_identifier text NOT NULL,
    database_name text NOT NULL,
    database_oid bigint NOT NULL,
    account_id uuid NOT NULL,
    revision bigint NOT NULL,
    receipt_id uuid NOT NULL,
    CONSTRAINT deployment_operator_head_singleton_check CHECK (singleton = 1),
    CONSTRAINT deployment_operator_head_revision_check CHECK (revision > 0),
    CONSTRAINT deployment_operator_head_receipt_v1
        FOREIGN KEY (receipt_id, revision, account_id, system_identifier, database_name, database_oid)
        REFERENCES public.deployment_operator_receipts
            (receipt_id, revision, account_id, system_identifier, database_name, database_oid)
        ON UPDATE RESTRICT ON DELETE RESTRICT
);

-- Reuse the existing numbered migration custody pattern: remove every default
-- grant, including the migration owner's ordinary self-grants. No DML window.
DO $custody$
DECLARE relation_name text; grantee_name text;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY['deployment_operator_receipts', 'deployment_operator_head'] LOOP
        EXECUTE pg_catalog.format('REVOKE ALL ON TABLE public.%I FROM PUBLIC', relation_name);
        FOR grantee_name IN
            SELECT DISTINCT role.rolname FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace ns ON ns.oid = relation.relnamespace
            CROSS JOIN LATERAL pg_catalog.aclexplode(relation.relacl) acl
            JOIN pg_catalog.pg_roles role ON role.oid = acl.grantee
            WHERE ns.nspname = 'public' AND relation.relname = relation_name
        LOOP
            EXECUTE pg_catalog.format('REVOKE ALL ON TABLE public.%I FROM %I', relation_name, grantee_name);
        END LOOP;
    END LOOP;
END
$custody$;

COMMENT ON COLUMN public.deployment_operator_receipts.receipt_id IS 'pd:personal — Deployment command attribution';
COMMENT ON COLUMN public.deployment_operator_receipts.command_id IS 'pd:personal — Deployment command attribution';
COMMENT ON COLUMN public.deployment_operator_receipts.system_identifier IS 'pd:personal — Deployment command target binding';
COMMENT ON COLUMN public.deployment_operator_receipts.database_name IS 'pd:personal — Deployment command target binding';
COMMENT ON COLUMN public.deployment_operator_receipts.database_oid IS 'pd:personal — Deployment command target binding';
COMMENT ON COLUMN public.deployment_operator_receipts.account_id IS 'pd:personal — Account deployment designation';
COMMENT ON COLUMN public.deployment_operator_receipts.kind IS 'pd:personal — Account deployment designation';
COMMENT ON COLUMN public.deployment_operator_receipts.expected_revision IS 'pd:personal — Deployment command history';
COMMENT ON COLUMN public.deployment_operator_receipts.revision IS 'pd:personal — Deployment command history';
COMMENT ON COLUMN public.deployment_operator_receipts.expected_security_generation IS 'pd:personal — Account security generation binding';
COMMENT ON COLUMN public.deployment_operator_receipts.reason IS 'pd:personal,undeclared — Operator-authored deployment revocation explanation; content is not constrained';
COMMENT ON COLUMN public.deployment_operator_receipts.recorded_at IS 'pd:personal — Deployment command audit time';
COMMENT ON COLUMN public.deployment_operator_head.singleton IS 'pd:personal — Deployment designation head';
COMMENT ON COLUMN public.deployment_operator_head.system_identifier IS 'pd:personal — Deployment designation target binding';
COMMENT ON COLUMN public.deployment_operator_head.database_name IS 'pd:personal — Deployment designation target binding';
COMMENT ON COLUMN public.deployment_operator_head.database_oid IS 'pd:personal — Deployment designation target binding';
COMMENT ON COLUMN public.deployment_operator_head.account_id IS 'pd:personal — Account deployment designation';
COMMENT ON COLUMN public.deployment_operator_head.revision IS 'pd:personal — Deployment designation revision';
COMMENT ON COLUMN public.deployment_operator_head.receipt_id IS 'pd:personal — Deployment designation receipt';
