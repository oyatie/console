-- UNEXECUTED SOURCE PROPOSAL. Protected intake and terminal allocation only.
-- Canonical employee/Person DML and common audit INSERT remain existing Rust owners.
-- Same retained adapter transaction authenticates original credentials, validates
-- CSRF and evaluates actual Cedar both before entry and after deferred closure.
CREATE FUNCTION public.native_people_expectations_match_v1(i public.native_people_inputs_v1,m jsonb)
RETURNS boolean LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT (i.expected_company_epoch=(m->>'company_epoch')::bigint
  AND i.expected_object_type_id=(m#>>'{action_reference,object_type_id}')::uuid
  AND i.expected_action_type_id=(m#>>'{action_reference,action_type_id}')::uuid
  AND i.expected_action_revision=(m#>>'{action_reference,registration_revision}')::bigint
  AND i.manifest_digest=decode(m#>>'{action_reference,manifest_digest}','hex')
  AND (SELECT count(*) FROM jsonb_array_elements(m->'named_properties'))=2
  AND EXISTS(SELECT 1 FROM jsonb_array_elements(m->'named_properties') p
   WHERE p->>'key'='person.legal_name' AND (p->>'org_id')::uuid=i.org_id
    AND (p->>'object_type_id')::uuid=i.expected_object_type_id
    AND (p->>'property_id')::uuid=i.legal_name_property_id
    AND (p->>'schema_revision')::bigint=i.expected_schema_revision)
  AND EXISTS(SELECT 1 FROM jsonb_array_elements(m->'named_properties') p
   WHERE p->>'key'='person.employee_number' AND (p->>'org_id')::uuid=i.org_id
    AND (p->>'object_type_id')::uuid=i.expected_object_type_id
    AND (p->>'property_id')::uuid=i.employee_number_property_id
    AND (p->>'schema_revision')::bigint=i.expected_schema_revision)) IS TRUE
$body$;

CREATE FUNCTION public.native_people_current_v1(p_actor uuid,p_family uuid,p_org uuid)
RETURNS jsonb LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE m jsonb;
BEGIN
 SELECT to_jsonb(s) INTO STRICT m FROM public.identity_company_people_projection_v1(
  p_actor,p_family,p_org,'people.directory.create') s;
 RETURN m;
EXCEPTION WHEN no_data_found THEN RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='people.directory.not_visible';
END
$body$;

CREATE FUNCTION public.native_people_prepare_v1(
 p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_expected_company_epoch bigint,
 p_object uuid,p_action uuid,p_action_revision bigint,p_schema_revision bigint,
 p_name_property uuid,p_number_property uuid,p_name text,p_number text)
RETURNS TABLE(inserted boolean,accepted_input public.native_people_inputs_v1,terminal public.native_people_terminals_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); m jsonb; at_time timestamptz;
BEGIN
 m:=public.native_people_current_v1(p_actor,p_family,p_org);
 PERFORM set_config('app.current_org',p_org::text,true);
 -- Company admission is bounded independently; unrelated Company work proceeds.
 PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.admission/1:'||p_org::text,0));
 PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.command/1:'||p_org::text||':'||p_command::text,0));
 SELECT i.* INTO accepted_input FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org AND i.command_id=p_command;
 IF FOUND THEN
  IF (accepted_input.actor_account_id,accepted_input.expected_company_epoch,accepted_input.expected_object_type_id,
    accepted_input.expected_action_type_id,accepted_input.expected_action_revision,accepted_input.expected_schema_revision,
    accepted_input.legal_name_property_id,accepted_input.employee_number_property_id,accepted_input.legal_name COLLATE "C",accepted_input.employee_number COLLATE "C")
   IS DISTINCT FROM (p_actor,p_expected_company_epoch,p_object,p_action,p_action_revision,p_schema_revision,p_name_property,p_number_property,p_name COLLATE "C",p_number COLLATE "C") THEN
   RAISE EXCEPTION 'people.directory.conflict';
  END IF;
  SELECT t.* INTO terminal FROM public.native_people_terminals_v1 t WHERE t.org_id=p_org AND t.command_id=p_command;
  inserted:=false;
 ELSE
  -- Never trim or normalize persisted inputs. Adapter validates user text first.
  IF public.native_people_text_valid_v1(p_name,200) IS DISTINCT FROM true
   OR public.native_people_text_valid_v1(p_number,64) IS DISTINCT FROM true THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_input';
  END IF;
  accepted_input.org_id:=p_org; accepted_input.command_id:=p_command; accepted_input.actor_account_id:=p_actor;
  accepted_input.employee_id:=gen_random_uuid(); accepted_input.legal_name:=p_name; accepted_input.employee_number:=p_number;
  accepted_input.expected_company_epoch:=p_expected_company_epoch; accepted_input.expected_object_type_id:=p_object;
  accepted_input.expected_action_type_id:=p_action; accepted_input.expected_action_revision:=p_action_revision;
  accepted_input.expected_schema_revision:=p_schema_revision; accepted_input.legal_name_property_id:=p_name_property;
  accepted_input.employee_number_property_id:=p_number_property;
  accepted_input.manifest_digest:=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex');
  accepted_input.codec_version:=1; accepted_input.input_bytes:=public.native_people_encode_v1(accepted_input);
  PERFORM 1 FROM public.native_people_decode_v1(1::smallint,accepted_input.input_bytes);
  m:=public.native_people_current_v1(p_actor,p_family,p_org);
  IF NOT public.native_people_expectations_match_v1(accepted_input,m) THEN RAISE EXCEPTION 'people.directory.revision_conflict'; END IF;
  IF EXISTS(SELECT 1 FROM public.ont_action_command_receipts r WHERE r.org_id=p_org AND r.command_id=p_command) THEN
   RAISE EXCEPTION 'people.directory.command_conflict';
  END IF;
  at_time:=clock_timestamp();
  IF (SELECT count(*) FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org
    AND i.execution_not_after>at_time AND NOT EXISTS(SELECT 1 FROM public.native_people_terminals_v1 t WHERE t.org_id=i.org_id AND t.command_id=i.command_id))>=256
   OR (SELECT count(*) FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org AND i.actor_account_id=p_actor
    AND i.execution_not_after>at_time AND NOT EXISTS(SELECT 1 FROM public.native_people_terminals_v1 t WHERE t.org_id=i.org_id AND t.command_id=i.command_id))>=16 THEN
   RAISE EXCEPTION 'people.directory.capacity';
  END IF;
  accepted_input.input_digest:=sha256(accepted_input.input_bytes); accepted_input.intake_receipt_id:=gen_random_uuid();
  accepted_input.accepted_at:=at_time; accepted_input.execution_not_after:=at_time+interval '168 hours';
  accepted_input.accepting_session_id:=p_family; accepted_input.acceptance_xid:=pg_current_xact_id();
  accepted_input.acceptance_backend_pid:=pg_backend_pid(); accepted_input.accepting_assignment_id:=(m->>'assignment_id')::uuid;
  accepted_input.accepting_assignment_revision:=(m->>'assignment_revision')::bigint;
  INSERT INTO public.native_people_inputs_v1 SELECT accepted_input.*;
  inserted:=true;
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_people_terminal_open_v1(p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_transition text)
RETURNS TABLE(inserted boolean,accepted_input public.native_people_inputs_v1,terminal public.native_people_terminals_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); m jsonb; at_time timestamptz; failure text;
BEGIN
 IF p_transition IS NULL OR p_transition NOT IN ('EXECUTE','CANCEL','STATUS') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_transition';
 END IF;
 m:=public.native_people_current_v1(p_actor,p_family,p_org);
 PERFORM set_config('app.current_org',p_org::text,true);
 PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.command/1:'||p_org::text||':'||p_command::text,0));
 SELECT i.* INTO accepted_input FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org AND i.command_id=p_command;
 IF NOT FOUND OR accepted_input.actor_account_id IS DISTINCT FROM p_actor THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT t.* INTO terminal FROM public.native_people_terminals_v1 t WHERE t.org_id=p_org AND t.command_id=p_command;
 IF FOUND THEN
  inserted:=false; PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT; RETURN;
 END IF;
 IF accepted_input.acceptance_xid=pg_current_xact_id() THEN RAISE EXCEPTION 'people.directory.separate_commit_required'; END IF;
 IF accepted_input.input_bytes IS DISTINCT FROM public.native_people_encode_v1(accepted_input)
  OR accepted_input.input_digest IS DISTINCT FROM sha256(accepted_input.input_bytes) THEN RAISE EXCEPTION 'people.directory.custody_invalid'; END IF;
 PERFORM 1 FROM public.native_people_decode_v1(accepted_input.codec_version,accepted_input.input_bytes);
 IF p_transition='EXECUTE' THEN
  -- Same historical key as legacy number writers; held through canonical INSERT.
  PERFORM pg_advisory_xact_lock(hashtext(p_org::text||':'||accepted_input.employee_number));
 END IF;
 m:=public.native_people_current_v1(p_actor,p_family,p_org); at_time:=clock_timestamp();
 IF at_time>=accepted_input.execution_not_after THEN failure:='intake_expired';
 ELSIF p_transition='STATUS' THEN
  inserted:=false; PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT; RETURN;
 ELSIF p_transition='CANCEL' THEN failure:='cancelled';
 ELSIF NOT public.native_people_expectations_match_v1(accepted_input,m) THEN failure:='revision_conflict';
 ELSIF EXISTS(SELECT 1 FROM public.ont_action_command_receipts r WHERE r.org_id=p_org AND r.command_id=p_command) THEN failure:='command_conflict';
 ELSIF EXISTS(SELECT 1 FROM public.employees e WHERE e.org_id=p_org AND e.employee_number COLLATE "C"=accepted_input.employee_number COLLATE "C") THEN failure:='employee_number_conflict';
 END IF;
 terminal.org_id:=p_org; terminal.command_id:=p_command; terminal.actor_account_id:=p_actor;
 terminal.intake_receipt_id:=accepted_input.intake_receipt_id; terminal.input_digest:=accepted_input.input_digest;
 terminal.outcome:=CASE failure WHEN 'intake_expired' THEN 'EXPIRED' WHEN 'cancelled' THEN 'CANCELLED'
  ELSE CASE WHEN failure IS NULL THEN 'COMMITTED' ELSE 'REJECTED' END END;
 terminal.result_code:=coalesce(failure,'registered'); terminal.transition_kind:=p_transition;
 terminal.execution_session_id:=p_family; terminal.effect_xid:=pg_current_xact_id(); terminal.effect_backend_pid:=pg_backend_pid();
 terminal.source_company_epoch:=(m->>'company_epoch')::bigint; terminal.source_policy_receipt_id:=(m->>'current_policy_receipt_id')::uuid;
 terminal.source_assignment_id:=(m->>'assignment_id')::uuid; terminal.source_assignment_revision:=(m->>'assignment_revision')::bigint;
 terminal.source_valid_from:=(m->>'assignment_valid_from')::timestamptz; terminal.source_valid_until:=(m->>'assignment_valid_until')::timestamptz;
 terminal.terminal_at:=at_time;
 IF failure IS NULL THEN terminal.employee_id:=accepted_input.employee_id; terminal.person_id:=accepted_input.employee_id; terminal.canonical_command_id:=p_command; END IF;
 -- Provisional only. Deferred closure requires all canonical rows/audit before COMMIT.
 INSERT INTO public.native_people_terminals_v1 SELECT terminal.*;
 inserted:=true; PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_people_preflight_v1(
 p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_expected_company_epoch bigint,
 p_object uuid,p_action uuid,p_action_revision bigint,p_schema_revision bigint,
 p_name_property uuid,p_number_property uuid,p_name text,p_number text)
RETURNS void LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); m jsonb; i public.native_people_inputs_v1;
BEGIN
 m:=public.native_people_current_v1(p_actor,p_family,p_org);
 IF p_command IS NULL OR p_command='00000000-0000-0000-0000-000000000000'::uuid
  OR public.native_people_text_valid_v1(p_name,200) IS DISTINCT FROM true
  OR public.native_people_text_valid_v1(p_number,64) IS DISTINCT FROM true THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_input';
 END IF;
 i.org_id:=p_org; i.expected_company_epoch:=p_expected_company_epoch; i.expected_object_type_id:=p_object;
 i.expected_action_type_id:=p_action; i.expected_action_revision:=p_action_revision;
 i.expected_schema_revision:=p_schema_revision; i.legal_name_property_id:=p_name_property;
 i.employee_number_property_id:=p_number_property;
 i.manifest_digest:=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex');
 IF NOT public.native_people_expectations_match_v1(i,m) THEN RAISE EXCEPTION 'people.directory.revision_conflict'; END IF;
 PERFORM set_config('app.current_org',p_org::text,true);
 IF EXISTS(SELECT 1 FROM public.ont_action_command_receipts r WHERE r.org_id=p_org AND r.command_id=p_command)
  OR EXISTS(SELECT 1 FROM public.native_people_inputs_v1 a WHERE a.org_id=p_org AND a.command_id=p_command) THEN
  RAISE EXCEPTION 'people.directory.command_conflict';
 END IF;
 IF EXISTS(SELECT 1 FROM public.employees e WHERE e.org_id=p_org AND e.employee_number COLLATE "C"=p_number COLLATE "C") THEN
  RAISE EXCEPTION 'people.directory.employee_number_conflict';
 END IF;
 -- Feasibility only; execute rechecks under shared number/command serialization.
 -- No identity allocation, intake/terminal, audit, or Auth proof side effect.
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
