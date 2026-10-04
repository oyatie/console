-- New Group layout1 digests and result layout2. The preallocated result receipt
-- enters heads; the result digest never enters its own inputs. No history reseal.
CREATE FUNCTION public.native_group_process_policy_ref_v1(p_tag smallint,p_revision bigint,p_digest bytea) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF p_tag=0 AND p_revision IS NULL AND p_digest IS NULL THEN RETURN decode('00','hex'); END IF;
 IF p_tag=1 AND p_revision>0 AND octet_length(p_digest)=32 THEN
  RETURN decode('01','hex')||int8send(p_revision)||p_digest;
 END IF;
 RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_policy_reference';
END
$body$;
CREATE FUNCTION public.native_group_process_head_ref_v1(p_process uuid,p_head bigint,p_version bigint,
 p_content_digest bytea,p_head_digest bytea,p_state text,p_expiry timestamptz) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF num_nonnulls(p_process,p_head,p_version,p_content_digest,p_head_digest,p_state,p_expiry)=0 THEN
  RETURN decode('00','hex');
 END IF;
 IF (num_nonnulls(p_process,p_head,p_version,p_content_digest,p_head_digest,p_state,p_expiry)=7
  AND p_process<>'00000000-0000-0000-0000-000000000000'::uuid AND p_head>0 AND p_version>0
  AND octet_length(p_content_digest)=32 AND octet_length(p_head_digest)=32
  AND p_state IN ('ACTIVE','SUSPENDED') AND isfinite(p_expiry)) IS TRUE THEN
  RETURN decode('01','hex')||uuid_send(p_process)||int8send(p_head)||int8send(p_version)
   ||p_content_digest||p_head_digest||int2send(CASE p_state WHEN 'ACTIVE' THEN 1 ELSE 2 END::smallint)
   ||int8send(public.native_group_process_micros_v1(p_expiry));
 END IF;
 RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_head_reference';
END
$body$;

CREATE FUNCTION public.native_group_process_action_roster_v1() RETURNS jsonb
LANGUAGE sql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$ SELECT $roster$[{"key":"identity.verifier.process.adopt/1","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","revision":1,"fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","revision":1,"fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]$roster$::jsonb $body$;

CREATE FUNCTION public.native_group_process_registration_bytes_v1(p_group uuid,p_incarnation uuid,
 p_schema text,p_schema_digest bytea,p_policy_digest bytea,p_codec_digest bytea,p_actions jsonb) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE output bytea; item jsonb; field_value text; action_id uuid; seen uuid[]:=ARRAY[]::uuid[]; roster jsonb:=public.native_group_process_action_roster_v1(); action_index integer:=0;
BEGIN
 IF p_group='00000000-0000-0000-0000-000000000000'::uuid
  OR p_incarnation='00000000-0000-0000-0000-000000000000'::uuid
  OR p_schema<>'native-group-process-v1' OR octet_length(p_schema_digest)<>32
  OR octet_length(p_policy_digest)<>32 OR octet_length(p_codec_digest)<>32
  OR jsonb_typeof(p_actions)<>'array' OR jsonb_array_length(p_actions)<>4 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration';
 END IF;
 output:=convert_to('CONSOLE.IDENTITY.GROUP.PROCESS.REGISTRATION','UTF8')||decode('000001','hex')
  ||uuid_send(p_group)||uuid_send(p_incarnation)||int8send(1::bigint)
  ||public.native_group_process_text_v1(p_schema,128)||p_schema_digest||p_policy_digest||p_codec_digest||int2send(4::smallint);
 FOR item IN SELECT value FROM jsonb_array_elements(p_actions) WITH ORDINALITY x(value,position) ORDER BY position LOOP
  IF jsonb_typeof(item) IS DISTINCT FROM 'object'
   OR item-'action_id' IS DISTINCT FROM roster->action_index
   OR jsonb_typeof(item->'action_id') IS DISTINCT FROM 'string'
   OR (item->>'action_id') COLLATE "C" !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration';
  END IF;
  action_index:=action_index+1;
  action_id:=(item->>'action_id')::uuid;
  IF action_id IS NULL OR action_id='00000000-0000-0000-0000-000000000000'::uuid
   OR action_id=ANY(seen) OR (item->>'revision')::bigint IS DISTINCT FROM 1
   OR jsonb_typeof(item->'fields') IS DISTINCT FROM 'array'
   OR jsonb_array_length(item->'fields') NOT BETWEEN 1 AND 64 THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration';
  END IF;
  seen:=array_append(seen,action_id);
  output:=output||public.native_group_process_text_v1(item->>'key',128)||uuid_send(action_id)||int8send(1::bigint)
   ||int2send(jsonb_array_length(item->'fields')::smallint);
  FOR field_value IN SELECT value FROM jsonb_array_elements_text(item->'fields') WITH ORDINALITY x(value,position) ORDER BY position LOOP
   output:=output||public.native_group_process_text_v1(field_value,128);
  END LOOP;
 END LOOP;
 IF output IS NULL THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration'; END IF;
 RETURN output;
END
$body$;

CREATE FUNCTION public.native_group_process_policy_head_bytes_v1(r public.native_group_identity_policy_heads_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.GROUP.POLICY.HEAD','UTF8')||decode('000001','hex')
  ||uuid_send(r.group_id)||uuid_send(r.group_incarnation)||int8send(r.revision)
  ||public.native_group_process_text_v1(r.schema_id,128)||r.schema_digest||r.policy_digest||r.codec_contract_digest
  ||r.registration_manifest_digest||uuid_send(r.first_actor_account_id)||uuid_send(r.first_command_id)
  ||r.first_input_digest||uuid_send(r.activation_receipt_id)||int8send(public.native_group_process_micros_v1(r.activated_at))
$body$;
CREATE FUNCTION public.native_group_process_version_bytes_v1(r public.native_group_process_versions_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.PROCESS.VERSION','UTF8')||decode('000001','hex')
  ||uuid_send(r.group_id)||uuid_send(r.group_incarnation)||uuid_send(r.process_id)||int8send(r.version)
  ||public.native_group_process_text_v1(r.schema_id,128)||uuid_send(r.actor_account_id)||uuid_send(r.designation_receipt_id)
  ||int8send(r.designation_revision)||int8send(r.policy_revision)||r.policy_head_digest||uuid_send(r.adopt_command_id)
  ||r.input_digest||int8send(public.native_group_process_micros_v1(r.admitted_at))||int8send(public.native_group_process_micros_v1(r.expires_at))
  ||int2send(r.operator_responsibility)||public.native_group_process_text_v1(r.title,120)||int2send(1::smallint)
  ||public.native_group_process_text_v1(r.intended_claimant_matching_procedure,2048)
  ||public.native_group_process_text_v1(r.account_possession_procedure,2048)
  ||public.native_group_process_text_v1(r.physical_human_evidence_procedure,2048)
  ||public.native_group_process_text_v1(r.duplicate_contradictory_claim_procedure,2048)
  ||public.native_group_process_text_v1(r.qualification_criteria_instruction,2048)
  ||public.native_group_process_text_v1(r.escalation_adjudication_procedure,2048)
  ||public.native_group_process_text_v1(r.evidence_minimization_retention_description,2048)
  ||public.native_group_process_text_v1(r.recipient_responsibility,2048)
$body$;
CREATE FUNCTION public.native_group_process_head_bytes_v1(r public.native_group_process_head_revisions_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.PROCESS.HEAD','UTF8')||decode('000001','hex')
  ||uuid_send(r.group_id)||uuid_send(r.group_incarnation)||uuid_send(r.process_id)||int8send(r.head_revision)
  ||int8send(r.content_version)||r.content_digest||int2send(CASE r.state WHEN 'ACTIVE' THEN 1 WHEN 'SUSPENDED' THEN 2 END::smallint)
  ||int8send(public.native_group_process_micros_v1(r.expires_at))||uuid_send(r.last_actor_account_id)||uuid_send(r.last_command_id)
  ||r.last_input_digest||uuid_send(r.result_receipt_id)||int8send(public.native_group_process_micros_v1(r.updated_at))
$body$;
CREATE FUNCTION public.native_group_process_result_bytes_v2(r public.native_group_process_results_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.PROCESS.RESULT','UTF8')||decode('000002','hex')
  ||uuid_send(r.actor_account_id)||uuid_send(r.command_id)||uuid_send(r.group_id)||uuid_send(r.group_incarnation)
  ||int2send(r.operation)||r.input_digest||uuid_send(r.intake_receipt_id)||uuid_send(r.result_receipt_id)
  ||int2send(CASE r.terminal_code WHEN 'ADOPTED' THEN 1 WHEN 'REPLACED' THEN 2 WHEN 'SUSPENDED' THEN 3
   WHEN 'REJECTED_STALE_EXPECTATION' THEN 4 WHEN 'REJECTED_PROCESS_EXPIRED' THEN 5 WHEN 'REJECTED_ALREADY_SUSPENDED' THEN 6 END::smallint)
  ||int8send(public.native_group_process_micros_v1(r.accepted_at))||int8send(public.native_group_process_micros_v1(r.executed_at))
  ||uuid_send(r.execution_session_id)||int8send(r.account_security_generation)||uuid_send(r.designation_receipt_id)
  ||int8send(r.designation_revision)||int8send(r.observed_group_revision)
  ||public.native_group_process_policy_ref_v1(r.policy_before_tag,r.policy_before_revision,r.policy_before_head_digest)
  ||public.native_group_process_policy_ref_v1(r.policy_after_tag,r.policy_after_revision,r.policy_after_head_digest)
  ||public.native_group_process_text_v1(r.schema_id,128)||r.schema_digest||r.policy_digest||r.codec_contract_digest
  ||int8send(r.registration_manifest_version)||r.registration_manifest_digest
  ||public.native_group_process_text_v1(r.cedar_sdk_version,128)||public.native_group_process_text_v1(r.cedar_language_version,128)
  ||uuid_send(r.requested_process_id)
  ||public.native_group_process_head_ref_v1(r.before_process_id,r.before_head_revision,r.before_content_version,
   r.before_content_digest,r.before_head_digest,r.before_state,r.before_expires_at)
  ||public.native_group_process_head_ref_v1(r.after_process_id,r.after_head_revision,r.after_content_version,
   r.after_content_digest,r.after_head_digest,r.after_state,r.after_expires_at)
  ||xid8send(r.effect_xid)||int4send(r.effect_backend_pid)
$body$;
