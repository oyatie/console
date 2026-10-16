-- App uses the common typed Account audit writer. These are only validators.
CREATE FUNCTION public.native_company_policy_ontology_snapshot_v1(r public.native_company_policy_receipts_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT jsonb_build_object('stable_key','pay_run','schema_version',1,'lifecycle_state','published',
 'catalog_version',r.catalog_version,'manifest_digest',encode(r.manifest_digest,'hex')) $body$;

CREATE FUNCTION public.native_company_policy_audit_admit_v1(e public.audit_events) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_company_policy_inputs_v1; r public.native_company_policy_receipts_v1;
 payload jsonb; at_time timestamptz; actor_id uuid; family_id uuid; target_kind text; control record; family record;
BEGIN
 IF e.action='policy.company_command.accept' THEN
  SELECT x.* INTO STRICT i FROM public.native_company_policy_inputs_v1 x WHERE x.org_id=e.org_id AND x.intake_receipt_id::text=e.target_id;
  IF i.acceptance_xid<>pg_current_xact_id() OR i.acceptance_backend_pid<>pg_backend_pid() THEN
   RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
  END IF;
  payload:=public.native_company_policy_accept_snapshot_v1(i); at_time:=i.accepted_at;
  actor_id:=i.actor_account_id; family_id:=i.accepting_session_id; target_kind:='native_company_policy_inputs_v1';
 ELSIF e.action='policy.company_command.complete' THEN
  SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=e.org_id AND x.receipt_id::text=e.target_id;
  payload:=public.native_company_policy_complete_snapshot_v1(r); at_time:=r.executed_at;
  actor_id:=r.actor_account_id; family_id:=r.execution_session_id; target_kind:='native_company_policy_receipts_v1';
 ELSIF e.action='ontology.object_type.builtin_install' THEN
  SELECT x.* INTO r FROM public.ont_object_types o JOIN public.native_company_policy_receipts_v1 x
   ON x.org_id=o.org_id AND x.receipt_id=o.policy_receipt_id
   WHERE o.org_id=e.org_id AND o.id::text=e.target_id;
  IF NOT FOUND THEN RETURN false; END IF;
  IF r.operation<>1 OR r.outcome<>'COMMITTED' OR r.installed_object_type_id::text IS DISTINCT FROM e.target_id THEN
   RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
  END IF;
  PERFORM public.native_company_policy_effect_frame_v1(e.org_id,r.receipt_id);
  payload:=public.native_company_policy_ontology_snapshot_v1(r); at_time:=r.executed_at;
  actor_id:=r.actor_account_id; family_id:=r.execution_session_id; target_kind:='ont_object_types';
 ELSE RETURN false;
 END IF;
 IF r.receipt_id IS NOT NULL AND (r.effect_xid<>pg_current_xact_id() OR r.effect_backend_pid<>pg_backend_pid()) THEN
  RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
 END IF;
 IF e.actor IS DISTINCT FROM actor_id OR e.target_type IS DISTINCT FROM target_kind OR e.before_snap IS NOT NULL
  OR e.after_snap IS DISTINCT FROM payload OR e.branch_id IS NOT NULL OR e.occurred_at IS DISTINCT FROM at_time
  OR e.trace_id IS NULL OR e.trace_id !~ '^[0-9a-f]{32}$' OR e.trace_id=repeat('0',32)
  OR e.span_id IS NULL OR e.span_id !~ '^[0-9a-f]{16}$' OR e.span_id=repeat('0',16) THEN
  RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
 END IF;
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(actor_id);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(actor_id,family_id);
 IF control.security_state<>'ACTIVE' OR family.user_id IS DISTINCT FROM actor_id
  OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1' OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation
  OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' THEN RAISE EXCEPTION 'account.authentication_invalid'; END IF;
 PERFORM 1 FROM public.account_login_consent_v1(actor_id);
 IF NOT public.account_company_setup_eligibility_v1(actor_id) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_company_policy.forbidden';
 END IF;
 RETURN true;
END
$body$;

CREATE FUNCTION public.native_company_policy_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE admitted boolean;
BEGIN
 IF TG_TABLE_NAME<>'audit_events' OR TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_company_policy.audit_frame_invalid'; END IF;
 admitted:=public.native_company_policy_audit_admit_v1(NEW);
 IF NOT admitted AND (NEW.action IN ('policy.company_command.accept','policy.company_command.complete')
  OR (NEW.action='ontology.object_type.builtin_install'
    AND NEW.after_snap->>'catalog_version'='native-payroll-collection-read-v1')) THEN
  RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_company_policy_assert_ontology_audit_v1(p_org uuid,p_receipt uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.native_company_policy_receipts_v1;
BEGIN
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=p_receipt;
 IF r.operation<>1 OR r.outcome<>'COMMITTED' OR r.effect_xid<>pg_current_xact_id() OR r.effect_backend_pid<>pg_backend_pid()
  OR (SELECT count(*) FROM public.audit_events e WHERE e.org_id=p_org AND e.action='ontology.object_type.builtin_install'
    AND e.target_id=r.installed_object_type_id::text)<>1
  OR NOT EXISTS(SELECT 1 FROM public.audit_events e WHERE e.org_id=p_org AND e.actor=r.actor_account_id
    AND e.action='ontology.object_type.builtin_install' AND e.target_type='ont_object_types'
    AND e.target_id=r.installed_object_type_id::text AND e.before_snap IS NULL AND e.branch_id IS NULL
    AND e.occurred_at=r.executed_at AND e.after_snap=public.native_company_policy_ontology_snapshot_v1(r)
    AND e.xmin=pg_current_xact_id()::xid) THEN
  RAISE EXCEPTION 'native_company_policy.ontology_audit_closure_invalid';
 END IF;
END
$body$;
