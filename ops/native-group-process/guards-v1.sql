-- Uninstalled Group owner perimeter. A has intake/audit only; B participants
-- attest an incomplete same-transaction frame, then deferred closure proves the
-- complete reciprocal result. No guard turns a selected UUID into Auth/Cedar.
CREATE FUNCTION public.native_group_process_validate_frame_v1(f public.native_group_process_effects_v1) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; m jsonb; source jsonb; d jsonb;
BEGIN
 PERFORM public.native_group_process_group_guard_v1(f.group_id,true);
 m:=public.native_group_process_current_context_v1(f.actor_account_id,f.execution_session_id,f.group_id,f.group_incarnation,true);
 PERFORM public.native_group_process_command_guard_v1(f.actor_account_id,f.command_id,true);
 SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x
  WHERE x.actor_account_id=f.actor_account_id AND x.command_id=f.command_id;
 PERFORM public.native_group_process_assert_input_v1(f.actor_account_id,f.command_id);
 source:=m->'source'; d:=public.native_group_process_decode_v1(i.input_bytes);
 IF (f.group_id,f.group_incarnation,f.operation,f.input_digest,f.intake_receipt_id,f.accepted_at)
   IS DISTINCT FROM (i.group_id,i.group_incarnation,i.operation,i.input_digest,i.intake_receipt_id,i.accepted_at)
  OR f.effect_xid IS DISTINCT FROM pg_current_xact_id() OR f.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR f.effect_xid=i.acceptance_xid OR f.executed_at<transaction_timestamp() OR f.executed_at>clock_timestamp()
  OR f.executed_at<i.accepted_at
  OR f.account_security_generation IS DISTINCT FROM (m#>>'{account,account_security_generation}')::bigint
  OR (f.designation_receipt_id,f.designation_revision,f.observed_group_revision)
   IS DISTINCT FROM ((m#>>'{designation,receipt_id}')::uuid,(m#>>'{designation,revision}')::bigint,(m#>>'{topology,revision}')::bigint)
  OR (f.schema_id,f.schema_digest,f.policy_digest,f.codec_contract_digest,f.registration_manifest_version,
    f.registration_manifest_digest,f.cedar_sdk_version,f.cedar_language_version)
   IS DISTINCT FROM (source->>'schema_id',(source->>'schema_digest')::bytea,(source->>'policy_digest')::bytea,
    (source->>'codec_contract_digest')::bytea,(source->>'registration_manifest_version')::bigint,
    (source->>'registration_manifest_digest')::bytea,source->>'cedar_sdk_version',source->>'cedar_language_version')
  OR (f.schema_id,f.schema_digest,f.policy_digest,f.codec_contract_digest,f.registration_manifest_version,
    f.registration_manifest_digest,f.cedar_sdk_version,f.cedar_language_version)
   IS DISTINCT FROM (i.schema_id,i.schema_digest,i.policy_digest,i.codec_contract_digest,i.registration_manifest_version,
    i.registration_manifest_digest,i.cedar_sdk_version,i.cedar_language_version)
  OR f.requested_process_id IS DISTINCT FROM (d->>'process_id')::uuid
  OR f.terminal_code IS DISTINCT FROM public.native_group_process_classify_v1(f,d)
  OR (f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') AND f.after_expires_at<=clock_timestamp()) THEN
  RAISE EXCEPTION 'native_group_process.effect_frame_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_input_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE m jsonb; source jsonb; d jsonb;
BEGIN
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_group_process.input_immutable'; END IF;
 m:=public.identity_native_group_process_material_v1(NEW.actor_account_id,NEW.accepted_session_id,
  NEW.group_id,NEW.group_incarnation,NEW.command_id,3::smallint,NEW.input_bytes);
 source:=m->'source'; d:=public.native_group_process_decode_v1(NEW.input_bytes);
 IF NEW.acceptance_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.acceptance_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR NEW.accepted_at<transaction_timestamp() OR NEW.accepted_at>clock_timestamp()
  OR NEW.codec_version<>1 OR NEW.input_digest IS DISTINCT FROM sha256(NEW.input_bytes)
  OR NEW.operation IS DISTINCT FROM (d->>'operation')::smallint
  OR NEW.account_security_generation IS DISTINCT FROM (m#>>'{account,account_security_generation}')::bigint
  OR (NEW.designation_receipt_id,NEW.designation_revision,NEW.observed_group_revision)
   IS DISTINCT FROM ((m#>>'{designation,receipt_id}')::uuid,(m#>>'{designation,revision}')::bigint,(m#>>'{topology,revision}')::bigint)
  OR (NEW.schema_id,NEW.schema_digest,NEW.policy_digest,NEW.codec_contract_digest,NEW.registration_manifest_version,
    NEW.registration_manifest_digest,NEW.cedar_sdk_version,NEW.cedar_language_version)
   IS DISTINCT FROM (source->>'schema_id',(source->>'schema_digest')::bytea,(source->>'policy_digest')::bytea,
    (source->>'codec_contract_digest')::bytea,(source->>'registration_manifest_version')::bigint,
    (source->>'registration_manifest_digest')::bytea,source->>'cedar_sdk_version',source->>'cedar_language_version')
  OR (NEW.accepted_policy_tag,NEW.accepted_policy_revision,NEW.accepted_policy_head_digest)
   IS DISTINCT FROM (CASE WHEN m->'policy'='null'::jsonb THEN 0::smallint ELSE 1::smallint END,
    (m#>>'{policy,revision}')::bigint,(m#>>'{policy,head_digest}')::bytea)
  OR (d->>'expected_group_revision')::bigint IS DISTINCT FROM NEW.observed_group_revision
  OR (d->>'expected_policy_revision')::bigint IS DISTINCT FROM coalesce(NEW.accepted_policy_revision,0)
  OR (NEW.operation=1 AND ((d->>'expected_prior_head_revision')::bigint IS DISTINCT FROM coalesce((m#>>'{head,head_revision}')::bigint,0)
    OR (m->'head'<>'null'::jsonb AND (d->>'process_id')::uuid IS DISTINCT FROM (m#>>'{head,process_id}')::uuid)
    OR (d->>'expiry_us')::numeric<=public.native_group_process_micros_v1(NEW.accepted_at)
    OR (d->>'expiry_us')::numeric-public.native_group_process_micros_v1(NEW.accepted_at)>31536000000000))
  OR (NEW.operation=6 AND (m->'head'='null'::jsonb
    OR (d->>'process_id',d->>'content_version',d->>'content_digest',d->>'expected_head_revision',d->>'expected_head_digest')
     IS DISTINCT FROM (m#>>'{head,process_id}',m#>>'{head,content_version}',m#>>'{head,content_digest}',m#>>'{head,head_revision}',m#>>'{head,head_digest}'))) THEN
  RAISE EXCEPTION 'native_group_process.input_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_effect_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE head public.native_group_process_heads_v1; policy public.native_group_identity_policy_heads_v1;
BEGIN
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_group_process.effect_immutable'; END IF;
 PERFORM public.native_group_process_validate_frame_v1(NEW);
 PERFORM public.native_group_process_assert_current_v1(NEW.group_id,NEW.group_incarnation);
 SELECT x.* INTO head FROM public.native_group_process_heads_v1 x WHERE x.group_id=NEW.group_id AND x.group_incarnation=NEW.group_incarnation;
 SELECT x.* INTO policy FROM public.native_group_identity_policy_heads_v1 x WHERE x.group_id=NEW.group_id AND x.group_incarnation=NEW.group_incarnation;
 IF (NEW.policy_before_tag,NEW.policy_before_revision,NEW.policy_before_head_digest)
   IS DISTINCT FROM (CASE WHEN policy.group_id IS NULL THEN 0::smallint ELSE 1::smallint END,policy.revision,policy.head_digest)
  OR (NEW.before_process_id,NEW.before_head_revision,NEW.before_content_version,NEW.before_content_digest,
    NEW.before_head_digest,NEW.before_state,NEW.before_expires_at)
   IS DISTINCT FROM (head.process_id,head.head_revision,head.content_version,head.content_digest,head.head_digest,head.state,head.expires_at) THEN
  RAISE EXCEPTION 'native_group_process.effect_preimage_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_participant_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_json jsonb; f public.native_group_process_effects_v1; actor uuid; command uuid; receipt uuid;
BEGIN
 IF TG_OP='TRUNCATE' THEN RAISE EXCEPTION 'native_group_process.truncate_denied'; END IF;
 IF TG_OP='DELETE' OR (TG_OP='UPDATE' AND TG_TABLE_NAME<>'native_group_process_heads_v1') THEN
  RAISE EXCEPTION 'native_group_process.participant_immutable';
 END IF;
 row_json:=to_jsonb(NEW);
 IF TG_TABLE_NAME='native_group_process_results_v1' THEN
  actor:=NEW.actor_account_id; command:=NEW.command_id;
 ELSIF TG_TABLE_NAME='native_group_identity_policy_heads_v1' THEN
  actor:=NEW.first_actor_account_id; command:=NEW.first_command_id;
 ELSIF TG_TABLE_NAME='native_group_process_versions_v1' THEN
  actor:=NEW.actor_account_id; command:=NEW.adopt_command_id;
 ELSIF TG_TABLE_NAME IN ('native_group_process_head_revisions_v1','native_group_process_heads_v1') THEN
  actor:=NEW.last_actor_account_id; command:=NEW.last_command_id;
 ELSE RAISE EXCEPTION 'native_group_process.guard_unavailable'; END IF;
 SELECT x.* INTO STRICT f FROM public.native_group_process_effects_v1 x WHERE x.actor_account_id=actor AND x.command_id=command;
 PERFORM public.native_group_process_validate_frame_v1(f);
 IF (row_json->>'group_id',row_json->>'group_incarnation') IS DISTINCT FROM (f.group_id::text,f.group_incarnation::text) THEN
  RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
 END IF;
 IF TG_TABLE_NAME='native_group_process_results_v1' THEN
  IF row_json-ARRAY['layout_version','result_bytes','result_digest','audit_id'] IS DISTINCT FROM to_jsonb(f)
   OR NEW.layout_version<>2 OR NEW.result_bytes IS DISTINCT FROM public.native_group_process_result_bytes_v2(NEW)
   OR NEW.result_digest IS DISTINCT FROM sha256(NEW.result_bytes) THEN RAISE EXCEPTION 'native_group_process.participant_frame_invalid'; END IF;
 ELSIF TG_TABLE_NAME='native_group_identity_policy_heads_v1' THEN
  IF f.terminal_code<>'ADOPTED' OR NEW.activation_receipt_id IS DISTINCT FROM f.result_receipt_id
   OR NEW.head_digest IS DISTINCT FROM f.policy_after_head_digest
   OR NEW.head_digest IS DISTINCT FROM sha256(public.native_group_process_policy_head_bytes_v1(NEW)) THEN
   RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
  END IF;
 ELSIF TG_TABLE_NAME='native_group_process_versions_v1' THEN
  IF f.terminal_code NOT IN ('ADOPTED','REPLACED')
   OR (NEW.process_id,NEW.version,NEW.content_digest,NEW.expires_at)
    IS DISTINCT FROM (f.after_process_id,f.after_content_version,f.after_content_digest,f.after_expires_at)
   OR NEW.content_digest IS DISTINCT FROM sha256(public.native_group_process_version_bytes_v1(NEW)) THEN
   RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
  END IF;
 ELSE
  IF f.terminal_code NOT IN ('ADOPTED','REPLACED','SUSPENDED')
   OR (NEW.process_id,NEW.head_revision,NEW.content_version,NEW.content_digest,NEW.head_digest,NEW.state,NEW.expires_at,
     NEW.last_input_digest,NEW.result_receipt_id,NEW.updated_at,NEW.before_head_digest)
    IS DISTINCT FROM (f.after_process_id,f.after_head_revision,f.after_content_version,f.after_content_digest,f.after_head_digest,
     f.after_state,f.after_expires_at,f.input_digest,f.result_receipt_id,f.executed_at,f.before_head_digest) THEN
   RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
  END IF;
  IF TG_TABLE_NAME='native_group_process_heads_v1' THEN
   IF (TG_OP='INSERT' AND f.terminal_code<>'ADOPTED') OR (TG_OP='UPDATE' AND
     (f.terminal_code='ADOPTED' OR (OLD.group_id,OLD.group_incarnation,OLD.process_id,OLD.head_revision,OLD.content_version,
       OLD.content_digest,OLD.head_digest,OLD.state,OLD.expires_at)
      IS DISTINCT FROM (f.group_id,f.group_incarnation,f.before_process_id,f.before_head_revision,f.before_content_version,
       f.before_content_digest,f.before_head_digest,f.before_state,f.before_expires_at))) THEN
    RAISE EXCEPTION 'native_group_process.head_preimage_invalid';
   END IF;
  END IF;
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_deferred_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_json jsonb:=to_jsonb(NEW); actor uuid; command uuid;
BEGIN
 IF TG_TABLE_NAME='native_group_identity_policy_heads_v1' THEN actor:=NEW.first_actor_account_id; command:=NEW.first_command_id;
 ELSIF TG_TABLE_NAME='native_group_process_versions_v1' THEN actor:=NEW.actor_account_id; command:=NEW.adopt_command_id;
 ELSIF TG_TABLE_NAME IN ('native_group_process_head_revisions_v1','native_group_process_heads_v1') THEN actor:=NEW.last_actor_account_id; command:=NEW.last_command_id;
 ELSE actor:=NEW.actor_account_id; command:=NEW.command_id; END IF;
 PERFORM public.native_group_process_assert_input_v1(actor,command);
 IF TG_TABLE_NAME<>'native_group_process_inputs_v1' THEN
  PERFORM public.native_group_process_assert_result_v1(actor,command);
  PERFORM public.native_group_process_assert_current_v1((row_json->>'group_id')::uuid,(row_json->>'group_incarnation')::uuid);
 END IF;
 RETURN NULL;
END
$body$;

CREATE FUNCTION public.native_group_process_immutable_statement_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$ BEGIN RAISE EXCEPTION 'native_group_process.statement_denied'; END $body$;

-- Row admission protects every actual head change. The transition table also
-- rejects zero-row or multi-row UPDATE attempts, which row triggers cannot see.
CREATE FUNCTION public.native_group_process_head_statement_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF (SELECT count(*) FROM native_group_old_heads)<>1 OR (SELECT count(*) FROM native_group_new_heads)<>1 THEN
  RAISE EXCEPTION 'native_group_process.head_statement_invalid';
 END IF;
 RETURN NULL;
END
$body$;

DO $triggers$
DECLARE relation text; before_function text;
BEGIN
 FOREACH relation IN ARRAY ARRAY['native_group_process_inputs_v1','native_group_process_effects_v1','native_group_process_results_v1',
  'native_group_identity_policy_heads_v1','native_group_process_versions_v1','native_group_process_head_revisions_v1','native_group_process_heads_v1'] LOOP
  before_function:=CASE relation WHEN 'native_group_process_inputs_v1' THEN 'native_group_process_input_guard_v1'
   WHEN 'native_group_process_effects_v1' THEN 'native_group_process_effect_guard_v1' ELSE 'native_group_process_participant_guard_v1' END;
  EXECUTE format('CREATE TRIGGER native_group_process_row_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.%I FOR EACH ROW EXECUTE FUNCTION public.%I()',relation,before_function);
  EXECUTE format('CREATE TRIGGER native_group_process_truncate_guard_v1 BEFORE TRUNCATE ON public.%I FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_participant_guard_v1()',relation);
  IF relation='native_group_process_heads_v1' THEN
   EXECUTE format('CREATE TRIGGER native_group_process_immutable_statement_v1 BEFORE DELETE ON public.%I FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_immutable_statement_v1()',relation);
   EXECUTE format('CREATE TRIGGER native_group_process_head_statement_v1 AFTER UPDATE ON public.%I REFERENCING OLD TABLE AS native_group_old_heads NEW TABLE AS native_group_new_heads FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_head_statement_v1()',relation);
   EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_head_statement_v1',relation);
  ELSE
   EXECUTE format('CREATE TRIGGER native_group_process_immutable_statement_v1 BEFORE UPDATE OR DELETE ON public.%I FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_immutable_statement_v1()',relation);
  END IF;
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_immutable_statement_v1',relation);
  EXECUTE format('CREATE CONSTRAINT TRIGGER native_group_process_closure_v1 AFTER INSERT OR UPDATE ON public.%I DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_group_process_deferred_guard_v1()',relation);
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_row_guard_v1',relation);
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_truncate_guard_v1',relation);
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_closure_v1',relation);
 END LOOP;
END
$triggers$;
