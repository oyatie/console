-- Additive, uninstalled Group writers. Explicit family selectors follow the
-- reviewed ABI amendment; current Auth/Cedar/proof remain adapter-owned. Owner
-- results are provisional until the shared audit and consuming finish commit.
CREATE FUNCTION public.identity_native_group_process_prepare_v1(
 p_actor uuid,p_family uuid,p_command uuid,p_input bytea)
RETURNS TABLE(inserted boolean,accepted_input public.native_group_process_inputs_v1,terminal public.native_group_process_results_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d jsonb; m jsonb; source jsonb; at_time timestamptz;
BEGIN
 d:=public.native_group_process_decode_v1(p_input);
 IF (d->>'actor_account_id',d->>'command_id') IS DISTINCT FROM (p_actor::text,p_command::text) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 m:=public.identity_native_group_process_material_v1(p_actor,p_family,(d->>'group_id')::uuid,
  (d->>'group_incarnation')::uuid,p_command,3::smallint,p_input);
 IF m->'original'<>'null'::jsonb THEN
  SELECT i.* INTO STRICT accepted_input FROM public.native_group_process_inputs_v1 i
   WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
  IF accepted_input.input_bytes IS DISTINCT FROM p_input THEN RAISE EXCEPTION 'native_group_process.conflict'; END IF;
  SELECT r.* INTO terminal FROM public.native_group_process_results_v1 r WHERE r.actor_account_id=p_actor AND r.command_id=p_command;
  inserted:=false; RETURN NEXT; RETURN;
 END IF;
 source:=m->'source'; at_time:=clock_timestamp();
 accepted_input:=jsonb_populate_record(NULL::public.native_group_process_inputs_v1,source-'registered_actions'||jsonb_build_object(
  'actor_account_id',p_actor,'command_id',p_command,'group_id',d->>'group_id','group_incarnation',d->>'group_incarnation',
  'operation',(d->>'operation')::smallint,'codec_version',1,'input_bytes',p_input,'input_digest',sha256(p_input),
  'intake_receipt_id',gen_random_uuid(),'accepted_at',at_time,'accepted_session_id',p_family,
  'account_security_generation',(m#>>'{account,account_security_generation}')::bigint,
  'designation_receipt_id',m#>>'{designation,receipt_id}','designation_revision',(m#>>'{designation,revision}')::bigint,
  'observed_group_revision',(m#>>'{topology,revision}')::bigint,
  'accepted_policy_tag',CASE WHEN m->'policy'='null'::jsonb THEN 0 ELSE 1 END,
  'accepted_policy_revision',m#>>'{policy,revision}','accepted_policy_head_digest',m#>>'{policy,head_digest}',
  'acceptance_xid',pg_current_xact_id()::text,'acceptance_backend_pid',pg_backend_pid(),'audit_id',gen_random_uuid()));
 INSERT INTO public.native_group_process_inputs_v1 SELECT accepted_input.*;
 -- The adapter appends the exact returned audit UUID before any closure read.
 inserted:=true; RETURN NEXT;
END
$body$;

CREATE FUNCTION public.identity_native_group_process_execute_v1(p_actor uuid,p_family uuid,p_command uuid)
RETURNS TABLE(inserted boolean,accepted_input public.native_group_process_inputs_v1,terminal public.native_group_process_results_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE m jsonb; source jsonb; d jsonb; at_time timestamptz; expiry_value timestamptz;
 f public.native_group_process_effects_v1; policy public.native_group_identity_policy_heads_v1;
 head public.native_group_process_heads_v1; version_row public.native_group_process_versions_v1;
 history public.native_group_process_head_revisions_v1; policy_json jsonb; head_json jsonb; content jsonb;
BEGIN
 -- Namespace discovery is a selector only. Rows are immutable; their Group and
 -- actor binding is reloaded after the deterministic Group-first lock plan.
 SELECT i.* INTO STRICT accepted_input FROM public.native_group_process_inputs_v1 i
  WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
 m:=public.identity_native_group_process_material_v1(p_actor,p_family,accepted_input.group_id,
  accepted_input.group_incarnation,p_command,4::smallint,NULL);
 SELECT i.* INTO STRICT accepted_input FROM public.native_group_process_inputs_v1 i
  WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
 IF accepted_input.acceptance_xid=pg_current_xact_id() THEN RAISE EXCEPTION 'native_group_process.separate_commit_required'; END IF;
 IF m#>'{original,terminal}'<>'null'::jsonb THEN
  -- Terminal replay belongs to the retained read-own scope, never this writer.
  RAISE EXCEPTION 'native_group_process.pending_required';
 END IF;
 source:=m->'source'; d:=public.native_group_process_decode_v1(accepted_input.input_bytes);
 at_time:=clock_timestamp(); policy_json:=m->'policy'; head_json:=m->'head';
 f:=jsonb_populate_record(NULL::public.native_group_process_effects_v1,source-'registered_actions'||jsonb_build_object(
  'actor_account_id',p_actor,'command_id',p_command,'group_id',accepted_input.group_id,'group_incarnation',accepted_input.group_incarnation,
  'operation',accepted_input.operation,'input_digest',accepted_input.input_digest,'intake_receipt_id',accepted_input.intake_receipt_id,
  'effect_id',gen_random_uuid(),'result_receipt_id',gen_random_uuid(),'accepted_at',accepted_input.accepted_at,
  'executed_at',at_time,'execution_session_id',p_family,
  'account_security_generation',(m#>>'{account,account_security_generation}')::bigint,
  'designation_receipt_id',m#>>'{designation,receipt_id}','designation_revision',(m#>>'{designation,revision}')::bigint,
  'observed_group_revision',(m#>>'{topology,revision}')::bigint,
  'policy_before_tag',CASE WHEN policy_json='null'::jsonb THEN 0 ELSE 1 END,
  'policy_before_revision',policy_json->>'revision','policy_before_head_digest',policy_json->>'head_digest',
  'requested_process_id',d->>'process_id','before_process_id',head_json->>'process_id',
  'before_head_revision',head_json->>'head_revision','before_content_version',head_json->>'content_version',
  'before_content_digest',head_json->>'content_digest','before_head_digest',head_json->>'head_digest',
  'before_state',head_json->>'state','before_expires_at',head_json->>'expires_at',
  'effect_xid',pg_current_xact_id()::text,'effect_backend_pid',pg_backend_pid()));
 f.terminal_code:=public.native_group_process_classify_v1(f,d);
 f.policy_after_tag:=f.policy_before_tag; f.policy_after_revision:=f.policy_before_revision; f.policy_after_head_digest:=f.policy_before_head_digest;
 f.after_process_id:=f.before_process_id; f.after_head_revision:=f.before_head_revision; f.after_content_version:=f.before_content_version;
 f.after_content_digest:=f.before_content_digest; f.after_head_digest:=f.before_head_digest; f.after_state:=f.before_state; f.after_expires_at:=f.before_expires_at;
 IF f.terminal_code='ADOPTED' THEN
  policy:=jsonb_populate_record(NULL::public.native_group_identity_policy_heads_v1,source-ARRAY['cedar_sdk_version','cedar_language_version']||jsonb_build_object(
   'group_id',f.group_id,'group_incarnation',f.group_incarnation,'revision',1,'first_actor_account_id',p_actor,
   'first_command_id',p_command,'first_input_digest',f.input_digest,'activation_receipt_id',f.result_receipt_id,'activated_at',at_time));
  policy.head_digest:=sha256(public.native_group_process_policy_head_bytes_v1(policy));
  f.policy_after_tag:=1; f.policy_after_revision:=1; f.policy_after_head_digest:=policy.head_digest;
 END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED') THEN
  content:=d->'content';
  -- Parse integer seconds through PostgreSQL's interval grammar, never a float
  -- to_timestamp. Exact microsecond roundtrip is mandatory before persistence.
  expiry_value:=((timestamp '1970-01-01'+(((d->>'expiry_us')::bigint/1000000)::text||' seconds')::interval) AT TIME ZONE 'UTC');
  IF public.native_group_process_micros_v1(expiry_value) IS DISTINCT FROM (d->>'expiry_us')::bigint THEN
   RAISE EXCEPTION 'native_group_process.expiry_unavailable';
  END IF;
  version_row:=jsonb_populate_record(NULL::public.native_group_process_versions_v1,content||jsonb_build_object(
   'group_id',f.group_id,'group_incarnation',f.group_incarnation,'process_id',f.requested_process_id,
   'version',coalesce(f.before_content_version,0)+1,'schema_id','GROUP_VERIFIER_PROCESS_V1',
   'actor_account_id',p_actor,'designation_receipt_id',f.designation_receipt_id,'designation_revision',f.designation_revision,
   'policy_revision',f.policy_after_revision,'policy_head_digest',f.policy_after_head_digest,
   'adopt_command_id',p_command,'input_digest',f.input_digest,'admitted_at',accepted_input.accepted_at,
   'expires_at',expiry_value,'operator_responsibility',1));
  version_row.content_digest:=sha256(public.native_group_process_version_bytes_v1(version_row));
  f.after_process_id:=f.requested_process_id; f.after_content_version:=version_row.version;
  f.after_content_digest:=version_row.content_digest; f.after_expires_at:=expiry_value; f.after_state:='ACTIVE';
 END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN
  f.after_head_revision:=coalesce(f.before_head_revision,0)+1;
  IF f.terminal_code='SUSPENDED' THEN f.after_state:='SUSPENDED'; END IF;
  history:=jsonb_populate_record(NULL::public.native_group_process_head_revisions_v1,jsonb_build_object(
   'group_id',f.group_id,'group_incarnation',f.group_incarnation,'process_id',f.after_process_id,'head_revision',f.after_head_revision,
   'content_version',f.after_content_version,'content_digest',f.after_content_digest,'state',f.after_state,'expires_at',f.after_expires_at,
   'last_actor_account_id',p_actor,'last_command_id',p_command,'last_input_digest',f.input_digest,
   'result_receipt_id',f.result_receipt_id,'updated_at',at_time,'before_head_digest',f.before_head_digest));
  history.head_digest:=sha256(public.native_group_process_head_bytes_v1(history)); f.after_head_digest:=history.head_digest;
 END IF;
 f.effect_census:=jsonb_build_object('policy_heads',CASE WHEN f.terminal_code='ADOPTED' THEN 1 ELSE 0 END,
  'versions',CASE WHEN f.terminal_code IN ('ADOPTED','REPLACED') THEN 1 ELSE 0 END,
  'heads',CASE WHEN f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN 1 ELSE 0 END,
  'head_revisions',CASE WHEN f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN 1 ELSE 0 END,
  'effects',1,'results',1,'audits',1);
 terminal:=jsonb_populate_record(NULL::public.native_group_process_results_v1,to_jsonb(f)||jsonb_build_object('layout_version',2,'audit_id',gen_random_uuid()));
 terminal.result_bytes:=public.native_group_process_result_bytes_v2(terminal); terminal.result_digest:=sha256(terminal.result_bytes);
 -- Frame precedes provisional participants. All reciprocal FKs are deferred;
 -- no complete material/closure is read until the adapter appends shared audit.
 INSERT INTO public.native_group_process_effects_v1 SELECT f.*;
 INSERT INTO public.native_group_process_results_v1 SELECT terminal.*;
 IF f.terminal_code='ADOPTED' THEN INSERT INTO public.native_group_identity_policy_heads_v1 SELECT policy.*; END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED') THEN INSERT INTO public.native_group_process_versions_v1 SELECT version_row.*; END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN
  INSERT INTO public.native_group_process_head_revisions_v1 SELECT history.*;
  IF f.terminal_code='ADOPTED' THEN
   INSERT INTO public.native_group_process_heads_v1 SELECT history.*;
  ELSE
   UPDATE public.native_group_process_heads_v1 h SET
    head_revision=history.head_revision,content_version=history.content_version,content_digest=history.content_digest,
    state=history.state,expires_at=history.expires_at,last_actor_account_id=history.last_actor_account_id,
    last_command_id=history.last_command_id,last_input_digest=history.last_input_digest,result_receipt_id=history.result_receipt_id,
    updated_at=history.updated_at,head_digest=history.head_digest,before_head_digest=history.before_head_digest
   WHERE h.group_id=f.group_id AND h.group_incarnation=f.group_incarnation
    AND (h.process_id,h.head_revision,h.content_version,h.content_digest,h.head_digest,h.state,h.expires_at)
     IS NOT DISTINCT FROM (f.before_process_id,f.before_head_revision,f.before_content_version,f.before_content_digest,
      f.before_head_digest,f.before_state,f.before_expires_at);
   IF NOT FOUND THEN RAISE EXCEPTION 'native_group_process.head_conflict'; END IF;
  END IF;
 END IF;
 inserted:=true; RETURN NEXT;
EXCEPTION WHEN no_data_found OR too_many_rows THEN RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;
