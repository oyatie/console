-- DESIGN PROPOSAL ONLY. Not admitted, installed, executed or release accepted.
-- Root owns migration numbering, source mount, Buck declarations and finalizer.
-- Proposed numbered230: compatible expansion; pair with complete protected activation.
-- This numbered step remains closed until the same release activates reviewed guards.
-- This cannot activate a native writer: actor/provenance gates permit only
-- historical forms, and both new tables carry explicit CHECK(false) gates.
-- All historical numbered files, rows, bytes, defaults and User FKs survive.
-- Execute transactionally through production run_migrations, never psql alone.

CREATE TABLE public.native_people_inputs_v1 (
    org_id uuid NOT NULL
        -- Company FK is installed atomically before activation opens CHECK(false).
        -- Dormant230 must not change historical organizations custody triggers.
        CHECK(org_id NOT IN ('00000000-0000-0000-0000-000000000000'::uuid,'00000000-0000-0000-0000-00000000face'::uuid)),
    command_id uuid NOT NULL CHECK(command_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    actor_account_id uuid NOT NULL CHECK(actor_account_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    employee_id uuid NOT NULL CHECK(employee_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    legal_name text NOT NULL CHECK(char_length(legal_name) BETWEEN 1 AND 200 AND octet_length(legal_name)<=800),
    employee_number text NOT NULL CHECK(char_length(employee_number) BETWEEN 1 AND 64 AND octet_length(employee_number)<=256),
    expected_company_epoch bigint NOT NULL CHECK(expected_company_epoch>=1),
    expected_object_type_id uuid NOT NULL CHECK(expected_object_type_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    expected_action_type_id uuid NOT NULL CHECK(expected_action_type_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    expected_action_revision bigint NOT NULL CHECK(expected_action_revision>=1),
    expected_schema_revision bigint NOT NULL CHECK(expected_schema_revision>=1),
    legal_name_property_id uuid NOT NULL CHECK(legal_name_property_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    employee_number_property_id uuid NOT NULL CHECK(employee_number_property_id<>'00000000-0000-0000-0000-000000000000'::uuid AND employee_number_property_id<>legal_name_property_id),
    manifest_digest bytea NOT NULL CHECK(manifest_digest=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')),
    codec_version smallint NOT NULL CHECK(codec_version=1),
    input_bytes bytea NOT NULL CHECK(octet_length(input_bytes) BETWEEN 226 AND 1280
      AND substring(input_bytes FROM 1 FOR 36)=decode('636f6e736f6c652e70656f706c652e6469726563746f72792d7265676973746572000001','hex')
      AND substring(input_bytes FROM 189 FOR 32)=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')),
    input_digest bytea NOT NULL CHECK(input_digest=pg_catalog.sha256(input_bytes)),
    intake_receipt_id uuid NOT NULL UNIQUE
        CHECK(intake_receipt_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    accepted_at timestamptz NOT NULL CHECK(isfinite(accepted_at)),
    execution_not_after timestamptz NOT NULL
        CHECK(isfinite(execution_not_after) AND execution_not_after=accepted_at+interval '168 hours'),
    accepting_session_id uuid NOT NULL CHECK(accepting_session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    acceptance_xid xid8 NOT NULL,
    acceptance_backend_pid integer NOT NULL CHECK(acceptance_backend_pid>0),
    accepting_assignment_id uuid NOT NULL,
    accepting_assignment_revision bigint NOT NULL CHECK(accepting_assignment_revision>=1),
    PRIMARY KEY(org_id,command_id),
    UNIQUE(org_id,command_id,employee_id),
    UNIQUE(org_id,command_id,actor_account_id,intake_receipt_id,input_digest),
    CONSTRAINT native_people_inputs_staged_closed_v1 CHECK(false)
);
CREATE INDEX native_people_inputs_outstanding_v1
    ON public.native_people_inputs_v1(org_id,execution_not_after,actor_account_id,command_id);

CREATE TABLE public.native_people_terminals_v1 (
    org_id uuid NOT NULL,
    command_id uuid NOT NULL,
    actor_account_id uuid NOT NULL,
    intake_receipt_id uuid NOT NULL,
    input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32),
    outcome text NOT NULL CHECK(outcome IN ('COMMITTED','REJECTED','CANCELLED','EXPIRED')),
    result_code text NOT NULL,
    CHECK((outcome,result_code) IN (('COMMITTED','registered'),('REJECTED','revision_conflict'),
      ('REJECTED','employee_number_conflict'),('REJECTED','command_conflict'),
      ('CANCELLED','cancelled'),('EXPIRED','intake_expired'))),
    transition_kind text NOT NULL CHECK(transition_kind IN ('EXECUTE','CANCEL','STATUS')),
    CHECK((transition_kind='EXECUTE' AND outcome<>'CANCELLED')
      OR (transition_kind='CANCEL' AND outcome IN ('CANCELLED','EXPIRED'))
      OR (transition_kind='STATUS' AND outcome='EXPIRED')),
    execution_session_id uuid NOT NULL CHECK(execution_session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    effect_xid xid8 NOT NULL,
    effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
    source_company_epoch bigint NOT NULL CHECK(source_company_epoch>=1),
    source_policy_receipt_id uuid NOT NULL,
    source_assignment_id uuid NOT NULL,
    source_assignment_revision bigint NOT NULL CHECK(source_assignment_revision>=1),
    source_valid_from timestamptz NOT NULL CHECK(isfinite(source_valid_from)),
    source_valid_until timestamptz NOT NULL CHECK(isfinite(source_valid_until) AND source_valid_until>source_valid_from),
    terminal_at timestamptz NOT NULL CHECK(isfinite(terminal_at)),
    employee_id uuid,
    person_id uuid,
    canonical_command_id uuid,
    PRIMARY KEY(org_id,command_id),
    FOREIGN KEY(org_id,command_id,actor_account_id,intake_receipt_id,input_digest)
        REFERENCES public.native_people_inputs_v1(org_id,command_id,actor_account_id,intake_receipt_id,input_digest)
        ON UPDATE RESTRICT ON DELETE RESTRICT,
    -- MATCH SIMPLE intentionally skips the committed-only link on other outcomes.
    -- The explicit IS TRUE check below closes the nullable-shape loophole.
    FOREIGN KEY(org_id,command_id,employee_id)
        REFERENCES public.native_people_inputs_v1(org_id,command_id,employee_id)
        ON UPDATE RESTRICT ON DELETE RESTRICT,
    FOREIGN KEY(org_id,canonical_command_id)
        REFERENCES public.ont_action_command_receipts(org_id,command_id)
        ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(employee_id,org_id) REFERENCES public.employees(id,org_id)
        ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY(org_id,person_id) REFERENCES public.persons(org_id,id)
        ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT native_people_terminal_shape_v1 CHECK((
        (outcome='COMMITTED' AND employee_id IS NOT NULL AND person_id=employee_id
            AND canonical_command_id=command_id)
        OR (outcome IN ('REJECTED','CANCELLED','EXPIRED')
            AND employee_id IS NULL AND person_id IS NULL AND canonical_command_id IS NULL)
    ) IS TRUE),
    CONSTRAINT native_people_terminals_staged_closed_v1 CHECK(false)
);
-- Canonical receipt target/actor/payload, Person revision/binding and terminal
-- pairing still require a reviewed same-transaction closure guard; FK alone is
-- expressly not native ownership/authorization/effect-completeness proof.

ALTER TABLE public.native_people_inputs_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_people_inputs_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_people_terminals_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_people_terminals_v1 FORCE ROW LEVEL SECURITY;
-- Immutability blocks UPDATE/DELETE/TRUNCATE independently of ordinary ACLs.
-- No append authority is granted; the insert gate remains closed.
CREATE FUNCTION public.native_people_history_immutable_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp AS $body$
BEGIN
    RAISE EXCEPTION 'people.directory.history_immutable' USING ERRCODE='P0001';
END;
$body$;
REVOKE ALL ON FUNCTION public.native_people_history_immutable_v1() FROM PUBLIC;
CREATE TRIGGER native_people_inputs_immutable_v1
    BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_people_inputs_v1
    FOR EACH STATEMENT EXECUTE FUNCTION public.native_people_history_immutable_v1();
CREATE TRIGGER native_people_terminals_immutable_v1
    BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_people_terminals_v1
    FOR EACH STATEMENT EXECUTE FUNCTION public.native_people_history_immutable_v1();
ALTER TABLE public.native_people_inputs_v1 ENABLE ALWAYS TRIGGER native_people_inputs_immutable_v1;
ALTER TABLE public.native_people_terminals_v1 ENABLE ALWAYS TRIGGER native_people_terminals_immutable_v1;
-- No policy or DML grant during staging. Existing company_actors migration0227
-- is the precedent: remove ALL inherited table ACLs including owner self-grants.
DO $custody$
DECLARE relation_name text; grantee_name text;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY['native_people_inputs_v1','native_people_terminals_v1'] LOOP
        EXECUTE format('REVOKE ALL ON TABLE public.%I FROM PUBLIC',relation_name);
        FOR grantee_name IN
            SELECT DISTINCT role.rolname FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace ns ON ns.oid=relation.relnamespace
            CROSS JOIN LATERAL pg_catalog.aclexplode(relation.relacl) acl
            JOIN pg_catalog.pg_roles role ON role.oid=acl.grantee
            WHERE ns.nspname='public' AND relation.relname=relation_name
        LOOP
            EXECUTE format('REVOKE ALL ON TABLE public.%I FROM %I',relation_name,grantee_name);
        END LOOP;
    END LOOP;
END;
$custody$;

COMMENT ON COLUMN public.native_people_inputs_v1.org_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.command_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.actor_account_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.employee_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.legal_name IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.employee_number IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.expected_company_epoch IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.codec_version IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.input_bytes IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.input_digest IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.intake_receipt_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.accepted_at IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.execution_not_after IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.org_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.command_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.actor_account_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.intake_receipt_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.input_digest IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.outcome IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.result_code IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.terminal_at IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.employee_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.person_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_terminals_v1.canonical_command_id IS 'pd:personal — native directory accepted request or outcome identifying a natural person';
COMMENT ON COLUMN public.native_people_inputs_v1.expected_object_type_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.expected_action_type_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.expected_action_revision IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.expected_schema_revision IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.legal_name_property_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.employee_number_property_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.manifest_digest IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.accepting_session_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.acceptance_xid IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.acceptance_backend_pid IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.accepting_assignment_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_inputs_v1.accepting_assignment_revision IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.transition_kind IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.execution_session_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.effect_xid IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.effect_backend_pid IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.source_company_epoch IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.source_policy_receipt_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.source_assignment_id IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.source_assignment_revision IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.source_valid_from IS 'pd:personal — native directory accepted authority and effect provenance';
COMMENT ON COLUMN public.native_people_terminals_v1.source_valid_until IS 'pd:personal — native directory accepted authority and effect provenance';
