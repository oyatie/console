-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_accept_snapshot_v1(i public.native_company_policy_inputs_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT CASE i.codec_version WHEN 1 THEN jsonb_build_object('protocol','COMPANY_BUSINESS_POLICY_V1','command_id',i.command_id::text,
 'intake_receipt_id',i.intake_receipt_id::text,'operation',(ARRAY['InstallPayrollReadCatalogV1','GrantPayrollReadV1','RevokePayrollReadV1'])[i.operation],
 'input_digest',encode(i.input_digest,'hex'),'session_id',i.accepting_session_id::text)
 ELSE jsonb_build_object('protocol','COMPANY_PEOPLE_POLICY_V1','command_id',i.command_id::text,
  'intake_receipt_id',i.intake_receipt_id::text,
  'operation',(ARRAY['InstallPeopleDirectoryCatalogV1','GrantPeopleDirectoryV1','RevokePeopleDirectoryV1'])[i.operation],
  'action',(SELECT d.action_key FROM public.native_company_policy_decode_v2(i.codec_version,i.input_bytes) d),
  'input_digest',encode(i.input_digest,'hex'),'session_id',i.accepting_session_id::text) END $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_complete_snapshot_v1(r public.native_company_policy_receipts_v1) RETURNS jsonb
LANGUAGE sql STABLE STRICT SECURITY INVOKER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT CASE r.codec_version WHEN 1 THEN jsonb_build_object('protocol','COMPANY_BUSINESS_POLICY_V1','command_id',r.command_id::text,
 'intake_receipt_id',r.intake_receipt_id::text,'operation',(ARRAY['InstallPayrollReadCatalogV1','GrantPayrollReadV1','RevokePayrollReadV1'])[r.operation],
 'input_digest',encode(r.input_digest,'hex'),'session_id',r.execution_session_id::text,
 'receipt_id',r.receipt_id::text,'outcome',r.outcome,'result_code',r.result_code,
 'epoch_before',r.epoch_before::text,'epoch_after',r.epoch_after::text,'predecessor_receipt_id',r.predecessor_receipt_id::text)
 ELSE jsonb_build_object('protocol','COMPANY_PEOPLE_POLICY_V1','command_id',r.command_id::text,
  'intake_receipt_id',r.intake_receipt_id::text,
  'operation',(ARRAY['InstallPeopleDirectoryCatalogV1','GrantPeopleDirectoryV1','RevokePeopleDirectoryV1'])[r.operation],
  'action',(SELECT d.action_key FROM public.native_company_policy_inputs_v1 i
    CROSS JOIN LATERAL public.native_company_policy_decode_v2(i.codec_version,i.input_bytes) d
    WHERE (i.actor_account_id,i.command_id,i.org_id,i.codec_version)=(r.actor_account_id,r.command_id,r.org_id,r.codec_version)),
  'input_digest',encode(r.input_digest,'hex'),'session_id',r.execution_session_id::text,
  'receipt_id',r.receipt_id::text,'outcome',r.outcome,'result_code',r.result_code,
  'epoch_before',r.epoch_before::text,'epoch_after',r.epoch_after::text,
  'predecessor_receipt_id',r.predecessor_receipt_id::text) END $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_assert_current_head_v1(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE h public.company_authority_heads; r public.native_company_policy_receipts_v1; i public.native_company_policy_inputs_v1;
BEGIN
 SELECT x.* INTO STRICT h FROM public.company_authority_heads x WHERE x.org_id=p_org;
 IF h.epoch=1 THEN
  IF h.current_policy_receipt_id IS NOT NULL THEN RAISE EXCEPTION 'native_company_policy.head_custody_invalid'; END IF;
  RETURN;
 END IF;
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=h.current_policy_receipt_id;
 SELECT x.* INTO STRICT i FROM public.native_company_policy_inputs_v1 x WHERE x.org_id=p_org
  AND x.actor_account_id=r.actor_account_id AND x.command_id=r.command_id;
 IF r.outcome<>'COMMITTED' OR r.committed_epoch IS DISTINCT FROM h.epoch OR r.epoch_after<>h.epoch
  OR r.epoch_before::numeric+1<>r.epoch_after::numeric OR i.input_digest IS DISTINCT FROM sha256(i.input_bytes)
  OR (i.intake_receipt_id,i.operation,i.codec_version,i.input_digest)
    IS DISTINCT FROM (r.intake_receipt_id,r.operation,r.codec_version,r.input_digest)
  OR (SELECT count(*) FROM public.audit_events e WHERE e.org_id=p_org AND e.action='policy.company_command.complete'
    AND e.target_id=r.receipt_id::text)<>1
  OR NOT EXISTS(SELECT 1 FROM public.audit_events e WHERE e.org_id=p_org AND e.actor=r.actor_account_id
    AND e.action='policy.company_command.complete' AND e.target_type='native_company_policy_receipts_v1'
    AND e.target_id=r.receipt_id::text AND e.before_snap IS NULL AND e.branch_id IS NULL
    AND e.occurred_at=r.executed_at AND e.after_snap=public.native_company_policy_complete_snapshot_v1(r)) THEN
  RAISE EXCEPTION 'native_company_policy.head_custody_invalid';
 END IF;
 PERFORM 1 FROM public.native_company_policy_decode_v2(i.codec_version,i.input_bytes);
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_assert_effects_v1(p_org uuid,p_receipt uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.native_company_policy_receipts_v1; h public.company_authority_heads;
 a public.user_role_assignments; ar public.policy_assignment_revisions; prior public.policy_assignment_revisions;
 rr public.policy_role_revisions; role_row public.policy_roles; creator public.native_company_policy_receipts_v1;
 participants integer;
BEGIN
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=p_receipt;
 SELECT x.* INTO STRICT h FROM public.company_authority_heads x WHERE x.org_id=p_org;
 SELECT (SELECT count(*) FROM public.policy_roles x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.policy_role_revisions x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.user_role_assignments x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.policy_assignment_revisions x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.native_company_catalog_installs x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.ont_builtin_catalog_installs x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.ont_object_types x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt) INTO participants;
 IF r.outcome='REJECTED' THEN
  IF participants<>0 THEN RAISE EXCEPTION 'native_company_policy.effect_closure_invalid'; END IF;
  RETURN;
 END IF;
 IF r.operation=1 THEN
  IF participants<>3 THEN RAISE EXCEPTION 'native_company_policy.effect_closure_invalid'; END IF;
  IF r.codec_version=1 THEN PERFORM public.native_company_policy_assert_payroll_catalog_v1(p_org);
  ELSIF r.codec_version=2 THEN PERFORM public.native_company_policy_assert_people_catalog_v1(p_org);
  ELSE RAISE EXCEPTION 'native_company_policy.effect_closure_invalid'; END IF;
  RETURN;
 END IF;
 IF participants<>(CASE WHEN r.assignment_revision_before IS NULL THEN 4 ELSE 1 END) THEN
  RAISE EXCEPTION 'native_company_policy.effect_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT a FROM public.user_role_assignments x WHERE x.org_id=p_org AND x.id=r.assignment_id;
 SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x
  WHERE x.org_id=p_org AND x.assignment_id=r.assignment_id AND x.revision=r.assignment_revision_after;
 SELECT x.* INTO STRICT role_row FROM public.policy_roles x WHERE x.org_id=p_org AND x.id=r.role_id;
 SELECT x.* INTO STRICT rr FROM public.policy_role_revisions x WHERE x.org_id=p_org AND x.role_id=r.role_id AND x.revision=1;
 SELECT x.* INTO STRICT creator FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=a.policy_receipt_id;
 IF a.subject_protocol<>'NATIVE_ACCOUNT' OR a.account_id IS DISTINCT FROM r.recipient_account_id
  OR a.role_id IS DISTINCT FROM r.role_id OR a.created_at IS DISTINCT FROM creator.executed_at
  OR a.assigned_by_account_id IS DISTINCT FROM creator.actor_account_id
  OR a.native_current_revision<r.assignment_revision_after OR a.policy_receipt_id IS DISTINCT FROM role_row.policy_receipt_id
  OR (a.origin_account_id,a.origin_command_id,a.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR (role_row.origin_account_id,role_row.origin_command_id,role_row.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR (rr.origin_account_id,rr.origin_command_id,rr.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR (ar.origin_account_id,ar.origin_command_id,ar.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR ar.policy_receipt_id IS DISTINCT FROM r.receipt_id OR ar.subject_protocol<>'NATIVE_ACCOUNT'
  OR ar.account_id IS DISTINCT FROM r.recipient_account_id OR ar.role_id IS DISTINCT FROM r.role_id OR ar.role_revision<>1
  OR (ar.state,ar.valid_from,ar.valid_until,ar.actor_account_id,ar.session_id,ar.created_at)
    IS DISTINCT FROM (r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until,r.actor_account_id,r.execution_session_id,r.executed_at)
  OR ar.ceiling_digest IS DISTINCT FROM rr.clause_digest OR ar.valid_from<rr.valid_from
  OR (r.operation=2 AND (ar.valid_from<>r.executed_at OR ar.valid_until>r.executed_at+interval '30 days')) THEN
  RAISE EXCEPTION 'native_company_policy.effect_closure_invalid';
 END IF;
 IF r.assignment_revision_before IS NOT NULL THEN
  SELECT x.* INTO STRICT prior FROM public.policy_assignment_revisions x WHERE x.org_id=p_org
   AND x.assignment_id=r.assignment_id AND x.revision=r.assignment_revision_before;
  IF (prior.account_id,prior.role_id,prior.role_revision,prior.ceiling_digest)
    IS DISTINCT FROM (ar.account_id,ar.role_id,ar.role_revision,ar.ceiling_digest)
   OR (r.operation=3 AND (prior.state<>'ACTIVE' OR (prior.valid_from,prior.valid_until)
     IS DISTINCT FROM (ar.valid_from,ar.valid_until)))
   OR (r.operation=2 AND prior.state='ACTIVE' AND prior.valid_until>r.executed_at) THEN
   RAISE EXCEPTION 'native_company_policy.effect_closure_invalid';
  END IF;
 END IF;
 PERFORM public.native_company_policy_business_clauses_v1(p_org,r.role_id);
END
$body$;
