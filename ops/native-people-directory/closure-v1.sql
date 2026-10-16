-- UNEXECUTED exact protected audit frames and deferred new-command closure.
CREATE FUNCTION public.native_people_accept_snapshot_v1(i public.native_people_inputs_v1)
RETURNS jsonb LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT jsonb_build_object('protocol','NATIVE_PEOPLE_DIRECTORY_V1','command_id',i.command_id::text,
  'intake_receipt_id',i.intake_receipt_id::text,'input_digest',encode(i.input_digest,'hex'),'session_id',i.accepting_session_id::text)
$body$;
CREATE FUNCTION public.native_people_terminal_snapshot_v1(t public.native_people_terminals_v1)
RETURNS jsonb LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT jsonb_build_object('protocol','NATIVE_PEOPLE_DIRECTORY_V1','command_id',t.command_id::text,
  'intake_receipt_id',t.intake_receipt_id::text,'input_digest',encode(t.input_digest,'hex'),'session_id',t.execution_session_id::text,
  'outcome',t.outcome,'result_code',t.result_code,'employee_id',t.employee_id::text,
  'person_id',t.person_id::text,'canonical_command_id',t.canonical_command_id::text)
$body$;

CREATE FUNCTION public.native_people_audit_material_v1(p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_action text)
RETURNS TABLE(target_type text,target_id text,occurred_at timestamptz,payload jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); i public.native_people_inputs_v1; t public.native_people_terminals_v1;
BEGIN
 PERFORM public.native_people_current_v1(p_actor,p_family,p_org);
 PERFORM set_config('app.current_org',p_org::text,true);
 IF p_action='people.directory.prepare' THEN
  SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
  IF (i.actor_account_id,i.accepting_session_id,i.acceptance_xid,i.acceptance_backend_pid)
   IS DISTINCT FROM (p_actor,p_family,pg_current_xact_id(),pg_backend_pid()) THEN RAISE EXCEPTION 'people.directory.audit_frame_invalid'; END IF;
  target_type:='native_people_inputs_v1'; target_id:=i.intake_receipt_id::text; occurred_at:=i.accepted_at;
  payload:=public.native_people_accept_snapshot_v1(i);
 ELSIF p_action='people.directory.register' THEN
  SELECT x.* INTO STRICT t FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
  IF (t.actor_account_id,t.execution_session_id,t.effect_xid,t.effect_backend_pid)
   IS DISTINCT FROM (p_actor,p_family,pg_current_xact_id(),pg_backend_pid()) THEN RAISE EXCEPTION 'people.directory.audit_frame_invalid'; END IF;
  target_type:='native_people_terminals_v1'; target_id:=t.command_id::text; occurred_at:=t.terminal_at;
  payload:=public.native_people_terminal_snapshot_v1(t);
 ELSE RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_audit_action'; END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_people_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE frame record;
BEGIN
 IF NEW.action NOT IN ('people.directory.prepare','people.directory.register') THEN RETURN NEW; END IF;
 SELECT * INTO STRICT frame FROM public.native_people_audit_material_v1(NEW.actor,
  (NEW.after_snap->>'session_id')::uuid,NEW.org_id,(NEW.after_snap->>'command_id')::uuid,NEW.action);
 IF (NEW.target_type,NEW.target_id,NEW.occurred_at,NEW.after_snap) IS DISTINCT FROM
  (frame.target_type,frame.target_id,frame.occurred_at,frame.payload)
  OR NEW.before_snap IS NOT NULL OR NEW.branch_id IS NOT NULL
  OR NEW.trace_id IS NULL OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
  OR NEW.span_id IS NULL OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
  RAISE EXCEPTION 'people.directory.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_input_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE m jsonb;
BEGIN
 IF TG_OP<>'INSERT' OR TG_TABLE_NAME<>'native_people_inputs_v1'
  OR NEW.acceptance_xid<>pg_current_xact_id() OR NEW.acceptance_backend_pid<>pg_backend_pid()
  OR NEW.accepted_at>clock_timestamp() OR clock_timestamp()>=NEW.execution_not_after
  OR NEW.input_bytes IS DISTINCT FROM public.native_people_encode_v1(NEW)
  OR NEW.input_digest IS DISTINCT FROM sha256(NEW.input_bytes) THEN RAISE EXCEPTION 'people.directory.input_frame_invalid'; END IF;
 PERFORM 1 FROM public.native_people_decode_v1(NEW.codec_version,NEW.input_bytes);
 m:=public.native_people_current_v1(NEW.actor_account_id,NEW.accepting_session_id,NEW.org_id);
 IF NOT public.native_people_expectations_match_v1(NEW,m)
  OR (NEW.accepting_assignment_id,NEW.accepting_assignment_revision)
   IS DISTINCT FROM ((m->>'assignment_id')::uuid,(m->>'assignment_revision')::bigint) THEN
  RAISE EXCEPTION 'people.directory.input_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_terminal_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE i public.native_people_inputs_v1; m jsonb;
BEGIN
 IF TG_OP<>'INSERT' OR TG_TABLE_NAME<>'native_people_terminals_v1'
  OR NEW.effect_xid<>pg_current_xact_id() OR NEW.effect_backend_pid<>pg_backend_pid() THEN RAISE EXCEPTION 'people.directory.terminal_frame_invalid'; END IF;
 SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=NEW.org_id AND x.command_id=NEW.command_id;
 IF i.acceptance_xid=pg_current_xact_id() OR NEW.terminal_at<i.accepted_at OR NEW.terminal_at>clock_timestamp()
  OR (NEW.outcome='EXPIRED' AND NEW.terminal_at<i.execution_not_after)
  OR (NEW.outcome<>'EXPIRED' AND (NEW.terminal_at>=i.execution_not_after OR clock_timestamp()>=i.execution_not_after)) THEN
  RAISE EXCEPTION 'people.directory.terminal_frame_invalid';
 END IF;
 m:=public.native_people_current_v1(NEW.actor_account_id,NEW.execution_session_id,NEW.org_id);
 IF (NEW.source_company_epoch,NEW.source_policy_receipt_id,NEW.source_assignment_id,NEW.source_assignment_revision,NEW.source_valid_from,NEW.source_valid_until)
  IS DISTINCT FROM ((m->>'company_epoch')::bigint,(m->>'current_policy_receipt_id')::uuid,(m->>'assignment_id')::uuid,
    (m->>'assignment_revision')::bigint,(m->>'assignment_valid_from')::timestamptz,(m->>'assignment_valid_until')::timestamptz)
  OR (NEW.outcome='COMMITTED' AND NOT public.native_people_expectations_match_v1(i,m)) THEN
  RAISE EXCEPTION 'people.directory.terminal_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_assert_closed_v1(p_org uuid,p_command uuid,p_stage text) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE i public.native_people_inputs_v1; t public.native_people_terminals_v1; m jsonb;
 e public.employees; digest bytea; result jsonb; frame record; actor_value uuid; family_value uuid; action_value text;
BEGIN
 SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
 IF p_stage='PREPARE' THEN
  IF i.acceptance_xid<>pg_current_xact_id() OR i.acceptance_backend_pid<>pg_backend_pid() THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
  actor_value:=i.actor_account_id; family_value:=i.accepting_session_id; action_value:='people.directory.prepare';
  IF clock_timestamp()>=i.execution_not_after OR EXISTS(SELECT 1 FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command) THEN
   RAISE EXCEPTION 'people.directory.closure_invalid';
  END IF;
  m:=public.native_people_current_v1(actor_value,family_value,p_org);
  IF NOT public.native_people_expectations_match_v1(i,m) THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
 ELSIF p_stage='TERMINAL' THEN
  SELECT x.* INTO STRICT t FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
  IF t.effect_xid<>pg_current_xact_id() OR t.effect_backend_pid<>pg_backend_pid() THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
  actor_value:=t.actor_account_id; family_value:=t.execution_session_id; action_value:='people.directory.register';
  m:=public.native_people_current_v1(actor_value,family_value,p_org);
  IF (t.source_company_epoch,t.source_policy_receipt_id,t.source_assignment_id,t.source_assignment_revision,t.source_valid_from,t.source_valid_until)
   IS DISTINCT FROM ((m->>'company_epoch')::bigint,(m->>'current_policy_receipt_id')::uuid,(m->>'assignment_id')::uuid,
    (m->>'assignment_revision')::bigint,(m->>'assignment_valid_from')::timestamptz,(m->>'assignment_valid_until')::timestamptz)
   OR (t.outcome<>'EXPIRED' AND clock_timestamp()>=i.execution_not_after) THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
 ELSE RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
 SELECT * INTO STRICT frame FROM public.native_people_audit_material_v1(actor_value,family_value,p_org,p_command,action_value);
 IF (SELECT count(*) FROM public.audit_events a WHERE a.org_id=p_org AND a.action=action_value AND a.target_id=frame.target_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.audit_events a WHERE a.org_id=p_org AND a.actor=actor_value AND a.action=action_value
   AND a.target_type=frame.target_type AND a.target_id=frame.target_id AND a.occurred_at=frame.occurred_at
   AND a.before_snap IS NULL AND a.branch_id IS NULL AND a.after_snap=frame.payload AND a.xmin=pg_current_xact_id()::xid) THEN
  RAISE EXCEPTION 'people.directory.audit_closure_invalid';
 END IF;
 -- Prohibit registration from fabricating Employment/profile/lifecycle/leave facts.
 IF EXISTS(SELECT 1 FROM public.employee_employment_profiles x WHERE x.org_id=p_org AND x.employee_id=i.employee_id)
  OR EXISTS(SELECT 1 FROM public.employee_lifecycle_events x WHERE x.org_id=p_org AND x.employee_id=i.employee_id)
  OR EXISTS(SELECT 1 FROM public.employment_source_bindings x WHERE x.org_id=p_org AND x.employee_id=i.employee_id)
  OR EXISTS(SELECT 1 FROM public.employment_revisions x WHERE x.org_id=p_org AND x.command_id=p_command)
  OR EXISTS(SELECT 1 FROM public.leave_balance_import_receipts x WHERE x.org_id=p_org AND x.employee_id=i.employee_id) THEN
  RAISE EXCEPTION 'people.directory.prohibited_effect';
 END IF;
 IF p_stage='PREPARE' OR t.outcome<>'COMMITTED' THEN
  IF EXISTS(SELECT 1 FROM public.employees x WHERE x.org_id=p_org AND (x.id=i.employee_id OR x.native_command_id=p_command))
   OR EXISTS(SELECT 1 FROM public.persons x WHERE x.org_id=p_org AND x.id=i.employee_id)
   OR EXISTS(SELECT 1 FROM public.person_revisions x WHERE x.org_id=p_org AND (x.person_id=i.employee_id OR (x.command_id=p_command AND x.actor_kind='ACCOUNT')))
   OR EXISTS(SELECT 1 FROM public.employee_person_bindings x WHERE x.org_id=p_org AND (x.employee_id=i.employee_id OR x.person_id=i.employee_id))
   OR EXISTS(SELECT 1 FROM public.ont_action_command_receipts x WHERE x.org_id=p_org AND x.command_id=p_command AND x.actor_kind='ACCOUNT') THEN
   RAISE EXCEPTION 'people.directory.unexpected_effect';
  END IF;
  RETURN;
 END IF;
 PERFORM public.native_people_frame_v1(p_org,p_command); digest:=public.native_people_effect_digest_v1(i); result:=public.native_people_result_v1(i);
 SELECT (jsonb_populate_record(NULL::public.employees,to_jsonb(projected))).* INTO STRICT e
 FROM (SELECT x.id,x.org_id,x.company,x.name,x.employee_number,x.source_kind,x.native_command_id,x.source_key,x.raw_row,x.source_metadata,x.created_at,x.updated_at,x.employment_status,x.identity_resolution_strategy,x.identity_resolution_confidence,x.identity_review_required,x.identity_name_only_merge,x.source_filename,x.source_sheet,x.source_row,x.home_branch_id,x.hire_date,x.exit_date,x.org_unit,x.job,x.position,x.worksite_name,x.worksite_address,x.leave_accrued,x.leave_used,x.leave_remaining
 FROM public.employees x WHERE x.org_id=p_org AND x.id=i.employee_id AND x.xmin=pg_current_xact_id()::xid) projected;
 IF NOT public.native_people_employee_shape_v1(e,i,t)
  OR (SELECT count(*) FROM public.employees x WHERE x.org_id=p_org AND x.native_command_id=p_command)<>1
  OR NOT EXISTS(SELECT 1 FROM public.persons x WHERE x.org_id=p_org AND x.id=i.employee_id AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid)
  OR (SELECT count(*) FROM public.person_revisions x WHERE x.org_id=p_org AND x.person_id=i.employee_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.person_revisions x WHERE x.org_id=p_org AND x.person_id=i.employee_id AND x.version=1
   AND x.command_id=p_command AND x.actor_kind='ACCOUNT' AND x.actor_id IS NULL AND x.actor_account_id=i.actor_account_id
   AND x.payload_digest=digest AND x.attributes=jsonb_build_object('legal_name',i.legal_name) AND x.receipt=result
   AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid)
  OR (SELECT count(*) FROM public.employee_person_bindings x WHERE x.org_id=p_org AND x.person_id=i.employee_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.employee_person_bindings x WHERE x.org_id=p_org AND x.employee_id=i.employee_id AND x.person_id=i.employee_id
   AND x.actor_kind='ACCOUNT' AND x.actor_id IS NULL AND x.actor_account_id=i.actor_account_id AND x.payload_digest=digest
   AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid)
  OR NOT EXISTS(SELECT 1 FROM public.ont_action_command_receipts x WHERE x.org_id=p_org AND x.command_id=p_command
   AND x.actor_kind='ACCOUNT' AND x.actor_id IS NULL AND x.actor_account_id=i.actor_account_id AND x.payload_digest=digest
   AND x.owner='person' AND x.target='people.create_person' AND x.action_key='directory_create' AND x.object_type_id=i.expected_object_type_id
   AND x.receipt=result AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid) THEN
  RAISE EXCEPTION 'people.directory.effect_closure_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_people_deferred_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true);
BEGIN
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 PERFORM public.native_people_assert_closed_v1(NEW.org_id,NEW.command_id,
  CASE TG_TABLE_NAME WHEN 'native_people_inputs_v1' THEN 'PREPARE' WHEN 'native_people_terminals_v1' THEN 'TERMINAL' END);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NULL;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
