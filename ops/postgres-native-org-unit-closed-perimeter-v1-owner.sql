-- Generated UNINSTALLED native OrgUnit closed-perimeter source; not a custody finalizer.
-- No finalized profile or installation is authorized by this artifact.
-- source: ops/native-org-unit/closed-perimeter-v1.sql
-- UNINSTALLED closed OrgUnit perimeter candidate. Install only through a
-- separately reviewed declared-artifact fixture and measured successor custody.
-- This source permits no native OrgUnit mutation and grants no table/role rights.
CREATE FUNCTION public.native_org_unit_closed_guard_v1()
RETURNS pg_catalog.trigger LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='100ms'
AS $body$
DECLARE
 candidates pg_catalog.jsonb[];
 candidate pg_catalog.jsonb;
 provenance pg_catalog.text;
 shared_receipt pg_catalog.bool:=TG_TABLE_NAME='ont_action_command_receipts';
BEGIN
 IF TG_TABLE_SCHEMA IS DISTINCT FROM 'public' OR TG_WHEN IS DISTINCT FROM 'BEFORE'
  OR TG_TABLE_NAME NOT IN('org_units','org_unit_revisions',
   'org_unit_source_bindings','ont_action_command_receipts') THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.invalid_guard_target';
 END IF;
 IF TG_OP='TRUNCATE' AND TG_LEVEL='STATEMENT' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.truncate_denied';
 END IF;
 IF TG_LEVEL IS DISTINCT FROM 'ROW' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.invalid_guard_target';
 END IF;

 -- Do not access an absent trigger record. UPDATE retains both actual origins,
 -- even when owner, typed target, JSON result target or Company is retagged.
 IF TG_OP='INSERT' THEN
  candidates:=ARRAY[pg_catalog.to_jsonb(NEW)];
 ELSIF TG_OP='DELETE' THEN
  candidates:=ARRAY[pg_catalog.to_jsonb(OLD)];
 ELSIF TG_OP='UPDATE' THEN
  candidates:=ARRAY[pg_catalog.to_jsonb(OLD),pg_catalog.to_jsonb(NEW)];
 ELSE
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.invalid_guard_target';
 END IF;

 FOREACH candidate IN ARRAY candidates LOOP
  -- Select OLD and NEW independently. Historical receipts have ontology.action
  -- ownership and a NULL typed target, so their JSON target also selects them.
  IF shared_receipt AND (
   candidate->>'owner'='org_unit'
   OR candidate->>'target' IN('organization.create_org_unit','organization.revise_org_unit')
   OR candidate->'receipt'->>'target' IN('organization.create_org_unit','organization.revise_org_unit')
  ) IS NOT TRUE THEN
   CONTINUE;
  END IF;
  IF pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.isolation_unsupported';
  END IF;

  -- The Account-owned classifier supplies and retains the actual provenance
  -- locks. OLD/NEW need no table reads or broader Account-owner row privileges.
  -- An ordinary classifier failure is unavailable, never a busy/retry promise.
  BEGIN
   provenance:=public.account_company_provenance_v1((candidate->>'org_id')::pg_catalog.uuid);
  EXCEPTION WHEN OTHERS THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.company_provenance_unavailable';
  END;
  IF provenance='NATIVE' THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.native_company_legacy_write_denied';
  ELSIF provenance IS DISTINCT FROM 'LEGACY' THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_org_unit.company_provenance_unavailable';
  END IF;

  -- Positively legacy selected receipts retain only the canonical matching pair
  -- or the exact historical default pair. Do not rewrite receipt/digest bytes.
  IF shared_receipt AND (
   pg_catalog.jsonb_typeof(candidate->'receipt') IS DISTINCT FROM 'object'
   OR pg_catalog.jsonb_typeof(candidate->'receipt'->'target') IS DISTINCT FROM 'string'
   OR (candidate->'receipt'->>'target' IN(
    'organization.create_org_unit','organization.revise_org_unit')) IS NOT TRUE
   OR (
    (candidate->>'owner'='org_unit'
     AND candidate->>'target' IN('organization.create_org_unit','organization.revise_org_unit')
     AND candidate->>'target'=candidate->'receipt'->>'target')
    OR (candidate->>'owner'='ontology.action' AND candidate->>'target' IS NULL)
   ) IS NOT TRUE
  ) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_org_unit.receipt_shape_invalid';
  END IF;
 END LOOP;

 IF TG_OP='DELETE' THEN RETURN OLD; ELSE RETURN NEW; END IF;
END
$body$;

-- These names sort before the existing People and immutable row triggers, so a
-- selected native origin reaches its exact refusal before another row guard.
CREATE TRIGGER native_org_unit_closed_row_v1
 BEFORE INSERT OR UPDATE OR DELETE ON public.org_units
 FOR EACH ROW EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.org_units ENABLE ALWAYS TRIGGER native_org_unit_closed_row_v1;
CREATE TRIGGER native_org_unit_closed_row_v1
 BEFORE INSERT OR UPDATE OR DELETE ON public.org_unit_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.org_unit_revisions ENABLE ALWAYS TRIGGER native_org_unit_closed_row_v1;
CREATE TRIGGER native_org_unit_closed_row_v1
 BEFORE INSERT OR UPDATE OR DELETE ON public.org_unit_source_bindings
 FOR EACH ROW EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.org_unit_source_bindings ENABLE ALWAYS TRIGGER native_org_unit_closed_row_v1;
CREATE TRIGGER native_org_unit_closed_row_v1
 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_action_command_receipts
 FOR EACH ROW EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER native_org_unit_closed_row_v1;

-- Keep immutable function bodies and event sets intact under replica mode.
-- Source-binding DELETE remains available when provenance is positively LEGACY.
ALTER TABLE public.org_unit_revisions ENABLE ALWAYS TRIGGER trg_org_unit_revisions_immutable;
ALTER TABLE public.org_unit_source_bindings ENABLE ALWAYS TRIGGER trg_org_unit_source_bindings_immutable;
ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER trg_ont_action_command_receipts_immutable;

-- Statement triggers also cover TRUNCATE CASCADE and session_replication_role.
CREATE TRIGGER native_org_unit_closed_truncate_v1
 BEFORE TRUNCATE ON public.org_units
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.org_units ENABLE ALWAYS TRIGGER native_org_unit_closed_truncate_v1;
CREATE TRIGGER native_org_unit_closed_truncate_v1
 BEFORE TRUNCATE ON public.org_unit_revisions
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.org_unit_revisions ENABLE ALWAYS TRIGGER native_org_unit_closed_truncate_v1;
CREATE TRIGGER native_org_unit_closed_truncate_v1
 BEFORE TRUNCATE ON public.org_unit_source_bindings
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.org_unit_source_bindings ENABLE ALWAYS TRIGGER native_org_unit_closed_truncate_v1;
CREATE TRIGGER native_org_unit_closed_truncate_v1
 BEFORE TRUNCATE ON public.ont_action_command_receipts
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_org_unit_closed_guard_v1();
ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER native_org_unit_closed_truncate_v1;

ALTER FUNCTION public.native_org_unit_closed_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_org_unit_closed_guard_v1() FROM PUBLIC;
-- Preserve owner-only invocation even if the installer has default function ACLs.
DO $acl$
DECLARE grantee_name pg_catalog.name;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_catalog.pg_proc p
  CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a
  JOIN pg_catalog.pg_roles r ON r.oid=a.grantee
  WHERE p.oid='public.native_org_unit_closed_guard_v1()'::pg_catalog.regprocedure
   AND r.rolname<>'console_account_owner'
 LOOP
  EXECUTE pg_catalog.format(
   'REVOKE ALL ON FUNCTION public.native_org_unit_closed_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
