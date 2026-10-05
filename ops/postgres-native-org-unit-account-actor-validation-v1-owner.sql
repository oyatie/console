-- Generated UNINSTALLED native OrgUnit Account actor validation source; not a custody finalizer.
-- No phase profile, installation or serving readiness is asserted by this artifact.
-- source: ops/native-org-unit/account-actor-validation-v1.sql
-- UNINSTALLED OrgUnit V12 Account actor CLOSED validation module v1.
-- Exact staged-profile precondition and independently measured limits belong to
-- the outer reviewed schema owner. Ordinary legacy/People/Group work continues.
-- Validation is a separate transaction from expansion; all three convalidated
-- flags commit together, or transaction rollback leaves the exact staged phase.
ALTER TABLE public.org_unit_revisions
    VALIDATE CONSTRAINT org_unit_revisions_actor_protocol_v1;
ALTER TABLE public.org_unit_revisions
    VALIDATE CONSTRAINT org_unit_revisions_native_actor_v1;
ALTER TABLE public.ont_action_command_receipts
    VALIDATE CONSTRAINT ont_action_receipts_actor_protocol_v2;
