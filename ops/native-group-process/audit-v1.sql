-- Uninstalled Group audit source, part of the complete owner successor only.
-- Group has no Company scope. Read visibility is limited to its exact protected
-- frames; it does not expose unrelated NULL-org Account audit events.
GRANT SELECT(id) ON public.audit_events TO console_account_owner;
CREATE POLICY native_group_process_account_audit_read_v1 ON public.audit_events
 FOR SELECT TO console_account_owner USING (
  org_id IS NULL AND branch_id IS NULL AND before_snap IS NULL AND (
   (action='identity.group_process.accept' AND target_type='native_group_process_inputs_v1'
    AND EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1 i WHERE i.audit_id=audit_events.id
     AND i.actor_account_id=audit_events.actor AND i.intake_receipt_id::text=audit_events.target_id
     AND i.accepted_at=audit_events.occurred_at AND audit_events.after_snap=public.native_group_process_accept_snapshot_v1(i)))
   OR (action='identity.group_process.complete' AND target_type='native_group_process_results_v1'
    AND EXISTS(SELECT 1 FROM public.native_group_process_results_v1 r WHERE r.audit_id=audit_events.id
     AND r.actor_account_id=audit_events.actor AND r.result_receipt_id::text=audit_events.target_id
     AND r.executed_at=audit_events.occurred_at AND audit_events.after_snap=public.native_group_process_complete_snapshot_v1(r)))
  )
 );

CREATE FUNCTION public.native_group_process_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; r public.native_group_process_results_v1;
 frame_group uuid; frame_actor uuid; frame_session uuid; frame_generation bigint;
 frame_xid xid8; frame_pid integer; frame_target text; frame_receipt uuid; frame_time timestamptz;
 frame_payload jsonb; account_material jsonb;
BEGIN
 IF TG_OP IN ('UPDATE','DELETE') AND OLD.action IN ('identity.group_process.accept','identity.group_process.complete') THEN
  RAISE EXCEPTION 'native_group_process.audit_immutable';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 IF NEW.action NOT IN ('identity.group_process.accept','identity.group_process.complete') THEN RETURN NEW; END IF;
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_group_process.audit_immutable'; END IF;
 -- These selectors confer no authority. The exact same-transaction frame and
 -- fixed Group-first locks are checked before any protected event is admitted.
 IF NEW.action='identity.group_process.accept' THEN
  SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x WHERE x.audit_id=NEW.id;
  frame_group:=i.group_id; frame_actor:=i.actor_account_id; frame_session:=i.accepted_session_id;
  frame_generation:=i.account_security_generation; frame_xid:=i.acceptance_xid; frame_pid:=i.acceptance_backend_pid;
  frame_target:='native_group_process_inputs_v1'; frame_receipt:=i.intake_receipt_id; frame_time:=i.accepted_at;
  frame_payload:=public.native_group_process_accept_snapshot_v1(i);
 ELSE
  SELECT x.* INTO STRICT r FROM public.native_group_process_results_v1 x WHERE x.audit_id=NEW.id;
  frame_group:=r.group_id; frame_actor:=r.actor_account_id; frame_session:=r.execution_session_id;
  frame_generation:=r.account_security_generation; frame_xid:=r.effect_xid; frame_pid:=r.effect_backend_pid;
  frame_target:='native_group_process_results_v1'; frame_receipt:=r.result_receipt_id; frame_time:=r.executed_at;
  frame_payload:=public.native_group_process_complete_snapshot_v1(r);
 END IF;
 IF frame_xid IS DISTINCT FROM pg_current_xact_id() OR frame_pid IS DISTINCT FROM pg_backend_pid() THEN
  RAISE EXCEPTION 'native_group_process.audit_frame_invalid';
 END IF;
 PERFORM public.native_group_process_group_guard_v1(frame_group,true);
 PERFORM 1 FROM public.group_authority_lock_shared_v1(frame_group);
 account_material:=public.native_group_process_account_material_v1(frame_actor,frame_session);
 IF (account_material->>'account_security_generation')::bigint IS DISTINCT FROM frame_generation
  OR (NEW.actor,NEW.target_type,NEW.target_id,NEW.occurred_at,NEW.after_snap)
   IS DISTINCT FROM (frame_actor,frame_target,frame_receipt::text,frame_time,frame_payload)
  OR NEW.org_id IS NOT NULL OR NEW.branch_id IS NOT NULL OR NEW.before_snap IS NOT NULL
  OR num_nonnulls(NEW.ip,NEW.user_agent,NEW.auth_method,NEW.device,NEW.classification_badges,NEW.anomaly,NEW.reason)<>0
  OR NEW.trace_id IS NULL OR octet_length(NEW.trace_id)<>32 OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
  OR NEW.span_id IS NULL OR octet_length(NEW.span_id)<>16 OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
  RAISE EXCEPTION 'native_group_process.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_audit_truncate_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1)
  OR EXISTS(SELECT 1 FROM public.native_group_process_results_v1) THEN
  RAISE EXCEPTION 'native_group_process.audit_immutable';
 END IF;
 RETURN NULL;
END
$body$;

CREATE TRIGGER native_group_process_audit_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_audit_guard_v1();
CREATE TRIGGER native_group_process_audit_truncate_guard_v1 BEFORE TRUNCATE ON public.audit_events
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_audit_truncate_guard_v1();
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_group_process_audit_guard_v1;
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_group_process_audit_truncate_guard_v1;
ALTER FUNCTION public.native_group_process_audit_guard_v1() OWNER TO console_account_owner;
ALTER FUNCTION public.native_group_process_audit_truncate_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_group_process_audit_guard_v1(),public.native_group_process_audit_truncate_guard_v1()
 FROM PUBLIC,console_app,console_rt,console_auth_rt;
