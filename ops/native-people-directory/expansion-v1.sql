-- First module of the atomic native People successor, never a standalone installer.
-- The outer owner must verify and lock the exact old custody plus closed230
-- before this expansion. All remaining modules and final capture share its transaction.
-- Historical canonical ALTER statements and column comments are retained verbatim.

ALTER TABLE public.person_revisions
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ALTER COLUMN actor_id DROP NOT NULL,
    ADD CONSTRAINT person_revisions_actor_staged_user_v1
        CHECK (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL);
ALTER TABLE public.employee_person_bindings
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ALTER COLUMN actor_id DROP NOT NULL,
    ADD CONSTRAINT employee_person_bindings_actor_staged_user_v1
        CHECK (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL);
ALTER TABLE public.ont_action_command_receipts
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ALTER COLUMN actor_id DROP NOT NULL,
    ADD CONSTRAINT ont_action_receipts_actor_staged_user_v1
        CHECK (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL);
-- Preserve old actor_id FKs, shared Company+command PK, owner/target checks,
-- all old table ACLs/RLS/immutability, and0229 UPDATE(employee_id) row-lock grant.
-- Native company_actors FKs are deliberately NOT installed by this migration:
-- finalized Account custody removes migration-role REFERENCES privileges.

ALTER TABLE public.employees
    ADD COLUMN source_kind text NOT NULL DEFAULT 'LEGACY',
    ADD COLUMN native_command_id uuid,
    ALTER COLUMN source_filename DROP NOT NULL,
    ALTER COLUMN source_sheet DROP NOT NULL,
    ALTER COLUMN source_row DROP NOT NULL,
    ADD CONSTRAINT employees_provenance_staged_legacy_v1 CHECK(
        source_kind='LEGACY' AND native_command_id IS NULL
        AND source_filename IS NOT NULL AND source_sheet IS NOT NULL AND source_row IS NOT NULL
        AND btrim(source_filename)<>'' AND btrim(source_sheet)<>'' AND source_row>0),
    ADD CONSTRAINT employees_native_intake_v1
        FOREIGN KEY(org_id,native_command_id,id)
        REFERENCES public.native_people_inputs_v1(org_id,command_id,employee_id)
        ON UPDATE RESTRICT ON DELETE RESTRICT;
-- Historical inline nonblank/positive checks remain; native NULL acceptance is
-- still impossible. Existing employee-number/import behavior is unchanged while
-- dormant. Do not open provenance gate until successor guards are complete.

COMMENT ON COLUMN public.person_revisions.actor_kind IS 'pd:personal — canonical Person actor attribution protocol';
COMMENT ON COLUMN public.person_revisions.actor_account_id IS 'pd:personal — canonical Person Account actor identity';
COMMENT ON COLUMN public.employee_person_bindings.actor_kind IS 'pd:personal — employee Person binding actor attribution protocol';
COMMENT ON COLUMN public.employee_person_bindings.actor_account_id IS 'pd:personal — employee Person binding Account actor identity';
COMMENT ON COLUMN public.ont_action_command_receipts.actor_kind IS 'pd:personal — canonical command actor attribution protocol';
COMMENT ON COLUMN public.ont_action_command_receipts.actor_account_id IS 'pd:personal — canonical command Account actor identity';
COMMENT ON COLUMN public.employees.source_kind IS 'pd:personal — directory identity provenance category';
COMMENT ON COLUMN public.employees.native_command_id IS 'pd:personal — accepted native directory provenance command';
