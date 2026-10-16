-- Group-first guards also cover immutable locators after live topology is gone.
-- The finite topology fence is intentional: legacy writers may not change a
-- native Group while the Group process source is installed. A later canonical
-- topology writer must replace this fence with a separately reviewed protocol.
CREATE FUNCTION public.native_group_process_group_guard_v1(p_group uuid,p_write boolean) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE guard_key bigint;
BEGIN
 IF p_group IS NULL OR p_write IS NULL
  OR p_group='00000000-0000-0000-0000-000000000000'::uuid
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'native_group_process.guard_unavailable';
 END IF;
 guard_key:=hashtextextended('console.identity.group-process.group/1:'||p_group::text,0);
 IF p_write THEN PERFORM pg_advisory_xact_lock(guard_key);
 ELSE PERFORM pg_advisory_xact_lock_shared(guard_key); END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_command_guard_v1(p_actor uuid,p_command uuid,p_write boolean) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE guard_key bigint;
BEGIN
 IF p_actor IS NULL OR p_command IS NULL OR p_write IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_command)
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'native_group_process.guard_unavailable';
 END IF;
 guard_key:=hashtextextended('console.identity.group-process.command/1:'||p_actor::text||':'||p_command::text,0);
 IF p_write THEN PERFORM pg_advisory_xact_lock(guard_key);
 ELSE PERFORM pg_advisory_xact_lock_shared(guard_key); END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_topology_fence_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE old_row jsonb; new_row jsonb; affected uuid;
BEGIN
 IF TG_OP='TRUNCATE' THEN
  IF EXISTS(SELECT 1 FROM public.groups g WHERE g.origin_account_id IS NOT NULL)
   OR EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_group_process.topology_fenced';
  END IF;
  RETURN NULL;
 END IF;
 IF TG_TABLE_NAME NOT IN ('groups','group_authority_heads','group_memberships','group_membership_revisions')
  OR TG_OP NOT IN ('INSERT','UPDATE','DELETE') THEN
  RAISE EXCEPTION 'native_group_process.guard_unavailable';
 END IF;
 IF TG_OP<>'INSERT' THEN old_row:=to_jsonb(OLD); END IF;
 IF TG_OP<>'DELETE' THEN new_row:=to_jsonb(NEW); END IF;
 FOR affected IN SELECT DISTINCT value FROM unnest(ARRAY[
   (old_row->>CASE TG_TABLE_NAME WHEN 'groups' THEN 'id' ELSE 'group_id' END)::uuid,
   (new_row->>CASE TG_TABLE_NAME WHEN 'groups' THEN 'id' ELSE 'group_id' END)::uuid]) value
  WHERE value IS NOT NULL ORDER BY value
 LOOP
  -- UPDATE and DELETE are denied before waiting for any new owner guard; this
  -- prevents a legacy row lock from inverting the Group-first owner order.
  IF TG_OP IN ('UPDATE','DELETE') AND
   (EXISTS(SELECT 1 FROM public.groups g WHERE g.id=affected AND g.origin_account_id IS NOT NULL)
    OR EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1 i WHERE i.group_id=affected)
    OR (TG_TABLE_NAME='groups' AND old_row->>'origin_account_id' IS NOT NULL)) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_group_process.topology_fenced';
  END IF;
  IF TG_OP='INSERT' AND TG_TABLE_NAME='groups' THEN
   -- Native birth holds Account already. Never wait here on a Group-first
   -- reader; abort contended B, preserving A. Successful birth retains the
   -- exact exclusive identity guard through publication and existing closure.
   IF affected='00000000-0000-0000-0000-000000000000'::uuid
    OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
    RAISE EXCEPTION 'native_group_process.guard_unavailable';
   END IF;
   IF NOT pg_try_advisory_xact_lock(hashtextextended('console.identity.group-process.group/1:'||affected::text,0)) THEN
    RAISE EXCEPTION USING ERRCODE='55P03',MESSAGE='native_group_process.birth_guard_unavailable';
   END IF;
  ELSIF TG_OP='INSERT' AND
   EXISTS(SELECT 1 FROM public.groups g WHERE g.id=affected AND g.origin_account_id IS NOT NULL) THEN
   -- Only the original same-transaction Company birth may add its initial
   -- authority/membership rows. No legacy append may widen an existing Group.
   IF NOT EXISTS(SELECT 1 FROM public.groups g
    JOIN public.company_enrollment_effect_bindings b ON
     (b.account_id,b.command_id,b.receipt_id,b.group_id)=
     (g.origin_account_id,g.origin_command_id,g.origin_receipt_id,g.id)
    WHERE g.id=affected AND b.effect_xid=pg_current_xact_id()
     AND b.effect_backend_pid=pg_backend_pid()) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_group_process.topology_fenced';
   END IF;
  END IF;
 END LOOP;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 RETURN NEW;
END
$body$;

CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.groups
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_authority_heads
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_memberships
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_membership_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.groups
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.group_authority_heads
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.group_memberships
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.group_membership_revisions
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
