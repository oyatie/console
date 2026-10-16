-- Historical closure checks require retained Group/command guards but never
-- current designation or live topology. Own receipts survive their loss.
-- Install only with the complete independently reviewed owner successor.
CREATE FUNCTION public.native_group_process_accept_snapshot_v1(i public.native_group_process_inputs_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT jsonb_build_object('protocol','GROUP_PROCESS_V1','command_id',i.command_id::text,
  'group_id',i.group_id::text,'group_incarnation',i.group_incarnation::text,'operation',i.operation,
  'input_digest',encode(i.input_digest,'hex'),'intake_receipt_id',i.intake_receipt_id::text,
  'session_id',i.accepted_session_id::text)
$body$;

CREATE FUNCTION public.native_group_process_complete_snapshot_v1(r public.native_group_process_results_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT jsonb_build_object('protocol','GROUP_PROCESS_V1','command_id',r.command_id::text,
  'group_id',r.group_id::text,'group_incarnation',r.group_incarnation::text,'operation',r.operation,
  'input_digest',encode(r.input_digest,'hex'),'intake_receipt_id',r.intake_receipt_id::text,
  'result_receipt_id',r.result_receipt_id::text,'session_id',r.execution_session_id::text,
  'terminal_code',r.terminal_code)
$body$;

CREATE FUNCTION public.native_group_process_assert_input_v1(p_actor uuid,p_command uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; decoded jsonb;
BEGIN
 SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 decoded:=public.native_group_process_decode_v1(i.input_bytes);
 IF i.codec_version<>1 OR i.input_digest IS DISTINCT FROM sha256(i.input_bytes)
  OR (decoded->>'actor_account_id',decoded->>'command_id',decoded->>'group_id',decoded->>'group_incarnation',
    (decoded->>'operation')::smallint)
   IS DISTINCT FROM (i.actor_account_id::text,i.command_id::text,i.group_id::text,i.group_incarnation::text,i.operation)
  OR NOT EXISTS(SELECT 1 FROM public.audit_events a WHERE a.id=i.audit_id
    AND a.org_id IS NULL AND a.actor=i.actor_account_id AND a.action='identity.group_process.accept'
    AND a.target_type='native_group_process_inputs_v1' AND a.target_id=i.intake_receipt_id::text
    AND a.occurred_at=i.accepted_at AND a.before_snap IS NULL AND a.branch_id IS NULL
    AND a.after_snap=public.native_group_process_accept_snapshot_v1(i))
  OR (SELECT count(*) FROM public.audit_events a WHERE a.action='identity.group_process.accept'
    AND a.target_id=i.intake_receipt_id::text)<>1 THEN
  RAISE EXCEPTION 'native_group_process.input_closure_invalid';
 END IF;
 IF i.operation=1 AND ((decoded->>'expiry_us')::numeric<=public.native_group_process_micros_v1(i.accepted_at)
  OR (decoded->>'expiry_us')::numeric-public.native_group_process_micros_v1(i.accepted_at)>31536000000000) THEN
  RAISE EXCEPTION 'native_group_process.input_closure_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_assert_result_v1(p_actor uuid,p_command uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; r public.native_group_process_results_v1;
 f public.native_group_process_effects_v1; policy public.native_group_identity_policy_heads_v1;
 activation public.native_group_process_results_v1; adoption public.native_group_process_results_v1;
 prior public.native_group_process_head_revisions_v1; successor public.native_group_process_head_revisions_v1;
 version_row public.native_group_process_versions_v1; decoded jsonb; content jsonb;
 stale boolean; expected_code text; committed boolean;
BEGIN
 PERFORM public.native_group_process_assert_input_v1(p_actor,p_command);
 SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 SELECT x.* INTO STRICT r FROM public.native_group_process_results_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 SELECT x.* INTO STRICT f FROM public.native_group_process_effects_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 decoded:=public.native_group_process_decode_v1(i.input_bytes);
 IF to_jsonb(r)-ARRAY['layout_version','result_bytes','result_digest','audit_id'] IS DISTINCT FROM to_jsonb(f)
  OR r.layout_version<>2 OR r.result_bytes IS DISTINCT FROM public.native_group_process_result_bytes_v2(r)
  OR r.result_digest IS DISTINCT FROM sha256(r.result_bytes)
  OR (r.policy_before_tag=0)<>(r.before_process_id IS NULL)
  OR (r.policy_after_tag=0)<>(r.after_process_id IS NULL)
  OR (r.policy_before_tag=0 AND (r.operation<>1
    OR (decoded->>'expected_policy_revision')::bigint IS DISTINCT FROM 0
    OR (decoded->>'expected_prior_head_revision')::bigint IS DISTINCT FROM 0))
  OR (r.group_id,r.group_incarnation,r.operation,r.input_digest,r.intake_receipt_id,r.accepted_at)
   IS DISTINCT FROM (i.group_id,i.group_incarnation,i.operation,i.input_digest,i.intake_receipt_id,i.accepted_at)
  OR r.requested_process_id::text IS DISTINCT FROM decoded->>'process_id'
  OR r.effect_xid=i.acceptance_xid
  OR NOT EXISTS(SELECT 1 FROM public.audit_events a WHERE a.id=r.audit_id
    AND a.org_id IS NULL AND a.actor=r.actor_account_id AND a.action='identity.group_process.complete'
    AND a.target_type='native_group_process_results_v1' AND a.target_id=r.result_receipt_id::text
    AND a.occurred_at=r.executed_at AND a.before_snap IS NULL AND a.branch_id IS NULL
    AND a.after_snap=public.native_group_process_complete_snapshot_v1(r))
  OR (SELECT count(*) FROM public.audit_events a WHERE a.action='identity.group_process.complete'
    AND a.target_id=r.result_receipt_id::text)<>1 THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 -- Source identity is historical metadata. Current serving-source checks are
 -- separate, and never rewrite the original result or command bytes.
 IF (r.schema_id,r.schema_digest,r.policy_digest,r.codec_contract_digest,r.registration_manifest_version,
   r.registration_manifest_digest,r.cedar_sdk_version,r.cedar_language_version)
  IS DISTINCT FROM (i.schema_id,i.schema_digest,i.policy_digest,i.codec_contract_digest,i.registration_manifest_version,
   i.registration_manifest_digest,i.cedar_sdk_version,i.cedar_language_version) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 IF r.policy_after_tag=1 THEN
  SELECT x.* INTO STRICT policy FROM public.native_group_identity_policy_heads_v1 x
   WHERE x.group_id=r.group_id AND x.group_incarnation=r.group_incarnation;
  IF (policy.revision,policy.head_digest) IS DISTINCT FROM (r.policy_after_revision,r.policy_after_head_digest)
   OR policy.head_digest IS DISTINCT FROM sha256(public.native_group_process_policy_head_bytes_v1(policy))
   OR policy.registration_manifest_digest IS DISTINCT FROM sha256(public.native_group_process_registration_bytes_v1(
     policy.group_id,policy.group_incarnation,policy.schema_id,policy.schema_digest,policy.policy_digest,
     policy.codec_contract_digest,policy.registered_actions)) THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
  IF r.terminal_code<>'ADOPTED' THEN
   SELECT x.* INTO STRICT activation FROM public.native_group_process_results_v1 x
    WHERE x.actor_account_id=policy.first_actor_account_id AND x.command_id=policy.first_command_id;
   -- Check the closed terminal role before recursion: activation is always
   -- the first adoption, so corrupt self-links cannot form a recursion cycle.
   IF activation.terminal_code<>'ADOPTED' OR activation.operation<>1
    OR (activation.group_id,activation.group_incarnation,activation.result_receipt_id,activation.executed_at,
      activation.input_digest,activation.policy_after_revision,activation.policy_after_head_digest)
     IS DISTINCT FROM (policy.group_id,policy.group_incarnation,policy.activation_receipt_id,policy.activated_at,
      policy.first_input_digest,policy.revision,policy.head_digest)
    OR activation.executed_at>r.executed_at THEN
    RAISE EXCEPTION 'native_group_process.result_closure_invalid';
   END IF;
   PERFORM public.native_group_process_assert_result_v1(activation.actor_account_id,activation.command_id);
  END IF;
 END IF;
 IF r.before_process_id IS NOT NULL THEN
  SELECT x.* INTO STRICT prior FROM public.native_group_process_head_revisions_v1 x
   WHERE x.group_id=r.group_id AND x.group_incarnation=r.group_incarnation AND x.head_revision=r.before_head_revision;
  IF (prior.process_id,prior.content_version,prior.content_digest,prior.head_digest,prior.state,prior.expires_at)
   IS DISTINCT FROM (r.before_process_id,r.before_content_version,r.before_content_digest,r.before_head_digest,r.before_state,r.before_expires_at)
   OR prior.head_digest IS DISTINCT FROM sha256(public.native_group_process_head_bytes_v1(prior))
   OR prior.updated_at>r.executed_at THEN RAISE EXCEPTION 'native_group_process.result_closure_invalid'; END IF;
 END IF;
 stale:=(decoded->>'expected_group_revision')::bigint<>r.observed_group_revision
  OR (decoded->>'expected_policy_revision')::bigint<>coalesce(r.policy_before_revision,0);
 IF r.operation=1 THEN
  stale:=stale OR (decoded->>'expected_prior_head_revision')::bigint<>coalesce(r.before_head_revision,0)
   OR (r.before_process_id IS NOT NULL AND r.before_process_id<>r.requested_process_id);
 ELSE
  stale:=stale OR r.before_process_id IS NULL OR r.before_process_id<>r.requested_process_id
   OR (decoded->>'content_version')::bigint IS DISTINCT FROM r.before_content_version
   OR (decoded->>'content_digest')::bytea IS DISTINCT FROM r.before_content_digest
   OR (decoded->>'expected_head_revision')::bigint IS DISTINCT FROM r.before_head_revision
   OR (decoded->>'expected_head_digest')::bytea IS DISTINCT FROM r.before_head_digest;
 END IF;
 IF stale THEN expected_code:='REJECTED_STALE_EXPECTATION';
 ELSIF r.operation=1 THEN
  IF (decoded->>'expiry_us')::bigint<=public.native_group_process_micros_v1(r.executed_at) THEN
   expected_code:='REJECTED_PROCESS_EXPIRED';
  ELSIF r.before_process_id IS NULL THEN expected_code:='ADOPTED';
  ELSE expected_code:='REPLACED'; END IF;
 ELSIF r.before_state='SUSPENDED' THEN expected_code:='REJECTED_ALREADY_SUSPENDED';
 ELSIF r.before_expires_at<=r.executed_at THEN expected_code:='REJECTED_PROCESS_EXPIRED';
 ELSE expected_code:='SUSPENDED'; END IF;
 IF r.terminal_code IS DISTINCT FROM expected_code THEN RAISE EXCEPTION 'native_group_process.result_closure_invalid'; END IF;
 committed:=r.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED');
 IF NOT committed THEN
  IF EXISTS(SELECT 1 FROM public.native_group_process_head_revisions_v1 h WHERE h.result_receipt_id=r.result_receipt_id)
   OR EXISTS(SELECT 1 FROM public.native_group_process_versions_v1 v WHERE v.actor_account_id=p_actor AND v.adopt_command_id=p_command)
   OR EXISTS(SELECT 1 FROM public.native_group_identity_policy_heads_v1 p WHERE p.activation_receipt_id=r.result_receipt_id) THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
  RETURN;
 END IF;
 SELECT x.* INTO STRICT successor FROM public.native_group_process_head_revisions_v1 x
  WHERE x.group_id=r.group_id AND x.group_incarnation=r.group_incarnation AND x.head_revision=r.after_head_revision;
 IF (successor.process_id,successor.content_version,successor.content_digest,successor.head_digest,successor.state,successor.expires_at,
   successor.last_actor_account_id,successor.last_command_id,successor.last_input_digest,successor.result_receipt_id,successor.updated_at,successor.before_head_digest)
  IS DISTINCT FROM (r.after_process_id,r.after_content_version,r.after_content_digest,r.after_head_digest,r.after_state,r.after_expires_at,
   r.actor_account_id,r.command_id,r.input_digest,r.result_receipt_id,r.executed_at,r.before_head_digest)
  OR successor.head_digest IS DISTINCT FROM sha256(public.native_group_process_head_bytes_v1(successor)) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT version_row FROM public.native_group_process_versions_v1 x WHERE
  (x.group_id,x.group_incarnation,x.process_id,x.version)=(r.group_id,r.group_incarnation,r.after_process_id,r.after_content_version);
 IF (version_row.content_digest,version_row.expires_at,version_row.policy_revision,version_row.policy_head_digest)
  IS DISTINCT FROM (r.after_content_digest,r.after_expires_at,r.policy_after_revision,r.policy_after_head_digest)
  OR version_row.content_digest IS DISTINCT FROM sha256(public.native_group_process_version_bytes_v1(version_row)) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 IF r.operation=1 THEN
  content:=jsonb_build_object('title',version_row.title,'method',version_row.method,
   'intended_claimant_matching_procedure',version_row.intended_claimant_matching_procedure,
   'account_possession_procedure',version_row.account_possession_procedure,
   'physical_human_evidence_procedure',version_row.physical_human_evidence_procedure,
   'duplicate_contradictory_claim_procedure',version_row.duplicate_contradictory_claim_procedure,
   'qualification_criteria_instruction',version_row.qualification_criteria_instruction,
   'escalation_adjudication_procedure',version_row.escalation_adjudication_procedure,
   'evidence_minimization_retention_description',version_row.evidence_minimization_retention_description,
   'recipient_responsibility',version_row.recipient_responsibility);
  IF content IS DISTINCT FROM decoded->'content'
   OR public.native_group_process_micros_v1(version_row.expires_at) IS DISTINCT FROM (decoded->>'expiry_us')::bigint
   OR (version_row.actor_account_id,version_row.designation_receipt_id,version_row.designation_revision,
    version_row.adopt_command_id,version_row.input_digest,version_row.admitted_at,version_row.operator_responsibility)
    IS DISTINCT FROM (r.actor_account_id,r.designation_receipt_id,r.designation_revision,r.command_id,r.input_digest,i.accepted_at,1::smallint) THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
 ELSE
  SELECT x.* INTO STRICT adoption FROM public.native_group_process_results_v1 x
   WHERE x.actor_account_id=version_row.actor_account_id AND x.command_id=version_row.adopt_command_id;
  -- A suspension retains the original version and its original admission
  -- attribution. Resolve that adoption rather than attribute it to suspension.
  IF adoption.operation<>1 OR adoption.terminal_code NOT IN ('ADOPTED','REPLACED')
   OR (adoption.group_id,adoption.group_incarnation,adoption.after_process_id,adoption.after_content_version,
     adoption.after_content_digest,adoption.after_expires_at,adoption.policy_after_revision,adoption.policy_after_head_digest)
    IS DISTINCT FROM (version_row.group_id,version_row.group_incarnation,version_row.process_id,version_row.version,
     version_row.content_digest,version_row.expires_at,version_row.policy_revision,version_row.policy_head_digest)
   OR adoption.executed_at>r.executed_at THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
  PERFORM public.native_group_process_assert_result_v1(adoption.actor_account_id,adoption.command_id);
 END IF;
 IF r.terminal_code='ADOPTED' AND (policy.first_actor_account_id,policy.first_command_id,policy.first_input_digest,
  policy.activation_receipt_id,policy.activated_at,policy.schema_id,policy.schema_digest,policy.policy_digest,policy.codec_contract_digest,
  policy.registration_manifest_version,policy.registration_manifest_digest)
  IS DISTINCT FROM (r.actor_account_id,r.command_id,r.input_digest,r.result_receipt_id,r.executed_at,r.schema_id,r.schema_digest,
   r.policy_digest,r.codec_contract_digest,r.registration_manifest_version,r.registration_manifest_digest) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_assert_current_v1(p_group uuid,p_incarnation uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE policy public.native_group_identity_policy_heads_v1; head public.native_group_process_heads_v1;
 historical public.native_group_process_head_revisions_v1; source jsonb; count_value bigint; latest bigint; version_count bigint;
BEGIN
 SELECT x.* INTO policy FROM public.native_group_identity_policy_heads_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation;
 SELECT x.* INTO head FROM public.native_group_process_heads_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation;
 SELECT count(*),max(h.head_revision) INTO count_value,latest FROM public.native_group_process_head_revisions_v1 h
  WHERE h.group_id=p_group AND h.group_incarnation=p_incarnation;
 SELECT count(*) INTO version_count FROM public.native_group_process_versions_v1 v
  WHERE v.group_id=p_group AND v.group_incarnation=p_incarnation;
 IF policy.group_id IS NULL THEN
  IF head.group_id IS NOT NULL OR count_value<>0 OR version_count<>0 THEN
   RAISE EXCEPTION 'native_group_process.current_closure_invalid';
  END IF;
  RETURN;
 END IF;
 source:=public.native_group_process_source_v1(p_group,p_incarnation);
 IF policy.revision<>1 OR head.group_id IS NULL OR count_value<>latest OR latest IS DISTINCT FROM head.head_revision
  OR version_count<>head.content_version
  OR (to_jsonb(policy)-ARRAY['group_id','group_incarnation','revision','first_actor_account_id','first_command_id',
    'first_input_digest','activation_receipt_id','activated_at','head_digest'])
   IS DISTINCT FROM (source-ARRAY['cedar_sdk_version','cedar_language_version'])
  OR policy.head_digest IS DISTINCT FROM sha256(public.native_group_process_policy_head_bytes_v1(policy)) THEN
  RAISE EXCEPTION 'native_group_process.current_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT historical FROM public.native_group_process_head_revisions_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation AND x.head_revision=latest;
 IF head IS DISTINCT FROM historical THEN RAISE EXCEPTION 'native_group_process.current_closure_invalid'; END IF;
 FOR historical IN SELECT x.* FROM public.native_group_process_head_revisions_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation ORDER BY x.head_revision LOOP
  IF historical.process_id IS DISTINCT FROM head.process_id THEN
   RAISE EXCEPTION 'native_group_process.current_closure_invalid';
  END IF;
  PERFORM public.native_group_process_assert_result_v1(historical.last_actor_account_id,historical.last_command_id);
 END LOOP;
 IF EXISTS(SELECT 1 FROM public.native_group_process_versions_v1 v WHERE v.group_id=p_group AND v.group_incarnation=p_incarnation
  AND (v.process_id<>head.process_id OR NOT EXISTS(SELECT 1 FROM public.native_group_process_head_revisions_v1 h
   WHERE h.group_id=v.group_id AND h.group_incarnation=v.group_incarnation AND h.process_id=v.process_id
    AND h.content_version=v.version AND h.content_digest=v.content_digest AND h.state='ACTIVE')))
  OR EXISTS(SELECT 1 FROM public.native_group_process_effects_v1 f WHERE f.group_id=p_group AND f.group_incarnation=p_incarnation
   AND NOT EXISTS(SELECT 1 FROM public.native_group_process_results_v1 r WHERE r.actor_account_id=f.actor_account_id AND r.command_id=f.command_id)) THEN
  RAISE EXCEPTION 'native_group_process.current_closure_invalid';
 END IF;
END
$body$;
