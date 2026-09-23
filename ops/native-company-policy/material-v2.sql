-- Additive dual-codec successor; installed only after exact predecessor custody verification.

CREATE OR REPLACE FUNCTION public.identity_native_policy_material_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_operation smallint,p_input bytea)
RETURNS TABLE(actor_account_id uuid,session_id uuid,account_security_generation bigint,
 designation_system_identifier text,designation_database_name text,designation_database_oid bigint,
 designation_revision bigint,designation_receipt_id uuid,org_id uuid,current_group_id uuid,
 group_revision bigint,group_incarnation uuid,membership_id uuid,membership_revision bigint,
 membership_incarnation uuid,company_epoch bigint,current_policy_receipt_id uuid,
 origin_account_id uuid,origin_command_id uuid,origin_receipt_id uuid,administrative_account_id uuid,
 company_actor_admission_receipt_id uuid,birth_assignment_id uuid,birth_role_id uuid,
 observed_at timestamptz,source_xid xid8,source_backend_pid integer)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid; planned_admin uuid;
 planned_recipient uuid; guarded uuid; candidate record; saved bytea; saved_codec smallint; source record; control record;
 family record; group_head record; membership record; head public.company_authority_heads;
 designation public.deployment_operator_head; birth public.company_enrollment_receipts;
 root_assignment public.user_role_assignments; root_role public.policy_roles;
 terminal public.native_company_policy_receipts_v1;
BEGIN
 IF current_setting('transaction_isolation')<>'read committed'
  OR p_account IS NULL OR p_family IS NULL OR p_company IS NULL OR p_command IS NULL
  OR p_operation IS NULL OR p_operation NOT BETWEEN 1 AND 3
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_account,p_family,p_company,p_command)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'native_company_policy.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.group_id,r.administrative_account_id INTO STRICT planned_group,planned_admin
  FROM public.organizations o JOIN public.company_enrollment_receipts r ON r.org_id=o.id
   AND r.account_id=o.origin_account_id AND r.command_id=o.origin_command_id AND r.receipt_id=o.origin_receipt_id
  WHERE o.id=p_company;
 IF p_input IS NOT NULL THEN
  SELECT * INTO STRICT candidate FROM public.native_company_policy_decode_v2(public.native_company_policy_codec_v2(p_input),p_input);
  IF (candidate.actor_account_id,candidate.org_id,candidate.command_id,candidate.operation)
    IS DISTINCT FROM (p_account,p_company,p_command,p_operation) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
 END IF;
 SELECT i.input_bytes,i.codec_version INTO saved,saved_codec FROM public.native_company_policy_inputs_v1 i
  WHERE i.actor_account_id=p_account AND i.command_id=p_command AND i.org_id=p_company;
 IF saved IS NOT NULL THEN
  SELECT d.recipient_account_id INTO planned_recipient FROM public.native_company_policy_decode_v2(saved_codec,saved) d;
 ELSIF p_input IS NOT NULL THEN planned_recipient:=candidate.recipient_account_id;
 END IF;
 SELECT * INTO STRICT group_head FROM public.group_authority_lock_exclusive_v1(planned_group);
 FOR guarded IN SELECT DISTINCT id FROM unnest(ARRAY[p_account,planned_admin,planned_recipient]) id
  WHERE id IS NOT NULL ORDER BY id
 LOOP
  PERFORM 1 FROM public.account_security_lock_exclusive_v1(guarded);
 END LOOP;
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation
  OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' OR family.auth_time IS NULL
  OR NOT isfinite(family.auth_time) OR NOT isfinite(family.created_at)
  OR family.auth_time>family.created_at OR family.created_at>clock_timestamp() THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 PERFORM 1 FROM public.account_login_consent_v1(p_account);
 IF NOT public.account_company_setup_eligibility_v1(p_account) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_company_policy.forbidden';
 END IF;
 SELECT h.* INTO STRICT designation FROM public.deployment_operator_head h WHERE h.singleton=1 AND h.account_id=p_account;
 -- Integrity is checked even when a distinct recipient has become ineligible.
 FOR guarded IN SELECT DISTINCT id FROM unnest(ARRAY[planned_admin,planned_recipient]) id
  WHERE id IS NOT NULL AND id<>p_account ORDER BY id
 LOOP
  PERFORM 1 FROM public.native_company_policy_registration_custody_v1(guarded);
 END LOOP;
 SELECT o.group_id,o.status,o.origin_account_id,o.origin_command_id,o.origin_receipt_id,
  r.administrative_account_id,r.receipt_id INTO STRICT source
  FROM public.organizations o JOIN public.company_enrollment_receipts r ON r.org_id=o.id
   AND r.account_id=o.origin_account_id AND r.command_id=o.origin_command_id AND r.receipt_id=o.origin_receipt_id
  WHERE o.id=p_company;
 IF source.group_id IS DISTINCT FROM planned_group OR source.administrative_account_id IS DISTINCT FROM planned_admin THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
 END IF;
 SELECT i.input_bytes,i.codec_version INTO saved,saved_codec FROM public.native_company_policy_inputs_v1 i
  WHERE i.actor_account_id=p_account AND i.command_id=p_command AND i.org_id=p_company;
 guarded:=NULL;
 IF saved IS NOT NULL THEN
  SELECT d.recipient_account_id INTO guarded FROM public.native_company_policy_decode_v2(saved_codec,saved) d;
 ELSIF p_input IS NOT NULL THEN guarded:=candidate.recipient_account_id;
 END IF;
 IF guarded IS DISTINCT FROM planned_recipient THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
 END IF;
 IF source.status<>'ACTIVE' OR group_head.state<>'ACTIVE'
  OR NOT EXISTS(SELECT 1 FROM public.groups g WHERE g.id=planned_group AND g.status='ACTIVE') THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_company_policy.forbidden';
 END IF;
 SELECT m.membership_id,m.current_revision,m.incarnation INTO STRICT membership
  FROM public.group_memberships m JOIN public.group_membership_revisions r ON
   (r.group_id,r.org_id,r.membership_id,r.revision,r.incarnation)=
   (m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation)
  WHERE m.group_id=planned_group AND m.org_id=p_company AND r.state='ACTIVE' AND r.to_time IS NULL;
 SELECT h.* INTO STRICT head FROM public.company_authority_heads h WHERE h.org_id=p_company FOR UPDATE;
 SELECT r.* INTO STRICT birth FROM public.company_enrollment_receipts r
  JOIN public.company_enrollment_requests q ON q.account_id=r.account_id AND q.command_id=r.command_id
   AND q.committed_receipt_id=r.receipt_id AND q.state='COMMITTED'
  WHERE (r.org_id,r.account_id,r.command_id,r.receipt_id)=
   (head.org_id,head.origin_account_id,head.origin_command_id,head.origin_receipt_id);
 IF (head.origin_account_id,head.origin_command_id,head.origin_receipt_id)
   IS DISTINCT FROM (source.origin_account_id,source.origin_command_id,source.origin_receipt_id)
  OR birth.administrative_account_id IS DISTINCT FROM planned_admin OR birth.root_revision<>1
  OR NOT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company AND a.account_id=planned_admin
    AND a.admission_receipt_id=birth.receipt_id) THEN
  RAISE EXCEPTION 'native_company_policy.material_unavailable';
 END IF;
 SELECT a.* INTO STRICT root_assignment FROM public.user_role_assignments a
  WHERE a.org_id=p_company AND a.id=birth.root_assignment_id FOR SHARE;
 SELECT r.* INTO STRICT root_role FROM public.policy_roles r
  WHERE r.org_id=p_company AND r.id=root_assignment.role_id FOR SHARE;
 IF root_assignment.subject_protocol<>'NATIVE_ACCOUNT' OR root_assignment.account_id<>planned_admin
  OR root_assignment.native_current_revision<>1 OR root_assignment.policy_receipt_id IS NOT NULL
  OR root_role.subject_protocol<>'NATIVE_ACCOUNT' OR root_role.native_current_revision<>1
  OR root_role.policy_receipt_id IS NOT NULL OR root_role.role_key<>'native_company_administration'
  OR (root_assignment.origin_account_id,root_assignment.origin_command_id,root_assignment.origin_receipt_id)
    IS DISTINCT FROM (birth.account_id,birth.command_id,birth.receipt_id)
  OR (root_role.origin_account_id,root_role.origin_command_id,root_role.origin_receipt_id)
    IS DISTINCT FROM (birth.account_id,birth.command_id,birth.receipt_id) THEN
  RAISE EXCEPTION 'native_company_policy.material_unavailable';
 END IF;
 PERFORM public.native_company_policy_assert_current_head_v1(p_company);
 PERFORM ontology_api.lock_native_company_catalog_current_v2(p_company);
 RETURN QUERY SELECT p_account,p_family,control.security_generation,designation.system_identifier,
  designation.database_name,designation.database_oid,designation.revision,designation.receipt_id,
  p_company,planned_group,group_head.revision,group_head.incarnation,membership.membership_id,
  membership.current_revision,membership.incarnation,head.epoch,head.current_policy_receipt_id,
  birth.account_id,birth.command_id,birth.receipt_id,planned_admin,birth.receipt_id,
  root_assignment.id,root_role.id,clock_timestamp(),pg_current_xact_id(),pg_backend_pid();
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'native_company_policy.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
