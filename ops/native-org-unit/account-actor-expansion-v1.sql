-- UNINSTALLED OrgUnit V12 Account actor CLOSED staging module v1.
-- Not a standalone installer. Execute only inside the separately reviewed
-- exact-profile, bounded isolated-owner finalizer transaction. No actor_id
-- relaxation, native writer, catalog install, receipt activation, or ACL change.
-- Preserve all historical columns, actor_id foreign keys, command bytes,
-- Company/People/Group source and closed ALWAYS guards.
ALTER TABLE public.org_unit_revisions
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ADD CONSTRAINT org_unit_revisions_actor_protocol_v1 CHECK((
        (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
        OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL)
    ) IS TRUE) NOT VALID,
    ADD CONSTRAINT org_unit_revisions_native_actor_v1
        FOREIGN KEY(org_id,actor_account_id)
        REFERENCES public.company_actors(org_id,account_id)
        ON UPDATE RESTRICT ON DELETE RESTRICT NOT VALID;

-- Retain ont_action_receipts_actor_protocol_v1 and the unchanged native People
-- receipt trigger. This additive successor CHECK cannot admit an OrgUnit receipt
-- while its predecessor CHECK and the closed guards remain installed.
-- The Person alternative below is copied literally from activation-v1.sql.
ALTER TABLE public.ont_action_command_receipts
    ADD CONSTRAINT ont_action_receipts_actor_protocol_v2 CHECK((
        (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
        OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL
            AND ((owner='person' AND target='people.create_person' AND action_key='directory_create' AND object_type_id IS NOT NULL)
                OR (owner='org_unit' AND target='organization.create_org_unit' AND action_key='create_site' AND object_type_id IS NOT NULL)
                OR (owner='org_unit' AND target='organization.revise_org_unit' AND action_key='correct_site_name' AND object_type_id IS NOT NULL)))
    ) IS TRUE) NOT VALID;

COMMENT ON COLUMN public.org_unit_revisions.actor_kind IS 'pd:personal — canonical OrgUnit actor attribution protocol';
COMMENT ON COLUMN public.org_unit_revisions.actor_account_id IS 'pd:personal — canonical OrgUnit Account actor identity';
