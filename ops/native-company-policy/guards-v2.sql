-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_input_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE material record; d record;
BEGIN
 IF TG_TABLE_NAME<>'native_company_policy_inputs_v1' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION 'native_company_policy.input_frame_invalid';
 END IF;
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(NEW.actor_account_id,
  NEW.accepting_session_id,NEW.org_id,NEW.command_id,NEW.operation,NEW.input_bytes);
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(NEW.codec_version,NEW.input_bytes);
 IF NEW.acceptance_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.acceptance_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR NEW.accepted_at<transaction_timestamp() OR NEW.accepted_at>clock_timestamp()
  OR NEW.input_digest IS DISTINCT FROM sha256(NEW.input_bytes) OR NEW.codec_version IS DISTINCT FROM d.codec_version
  OR public.native_company_policy_operation_check_v1(NEW.org_id,NEW.input_bytes,NEW.accepted_at) IS NOT NULL THEN
  RAISE EXCEPTION 'native_company_policy.input_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_receipt_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE material record; accepted public.native_company_policy_inputs_v1; failure text; d record;
BEGIN
 IF TG_TABLE_NAME<>'native_company_policy_receipts_v1' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION 'native_company_policy.receipt_frame_invalid';
 END IF;
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(NEW.actor_account_id,
  NEW.execution_session_id,NEW.org_id,NEW.command_id,NEW.operation,NULL);
 SELECT i.* INTO STRICT accepted FROM public.native_company_policy_inputs_v1 i
  WHERE i.org_id=NEW.org_id AND i.actor_account_id=NEW.actor_account_id AND i.command_id=NEW.command_id;
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(accepted.codec_version,accepted.input_bytes);
 IF NEW.executed_at>=accepted.execution_not_after THEN failure:='intake_expired';
 ELSE failure:=public.native_company_policy_operation_check_v1(NEW.org_id,accepted.input_bytes,NEW.executed_at); END IF;
 IF NEW.codec_version IS DISTINCT FROM d.codec_version OR NEW.catalog_version IS DISTINCT FROM d.catalog_version
  OR NEW.manifest_digest IS DISTINCT FROM d.manifest_digest
  OR (NEW.actor_account_id,NEW.org_id,NEW.command_id,NEW.operation)
   IS DISTINCT FROM (d.actor_account_id,d.org_id,d.command_id,d.operation)
  OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR accepted.acceptance_xid=NEW.effect_xid OR NEW.executed_at<transaction_timestamp() OR NEW.executed_at>clock_timestamp()
  OR NEW.executed_at<accepted.accepted_at OR NEW.epoch_before IS DISTINCT FROM material.company_epoch
  OR NEW.predecessor_receipt_id IS DISTINCT FROM material.current_policy_receipt_id
  OR NEW.outcome IS DISTINCT FROM (CASE WHEN failure IS NULL THEN 'COMMITTED' ELSE 'REJECTED' END)
  OR NEW.result_code IS DISTINCT FROM coalesce(failure,(ARRAY['installed','granted','revoked'])[NEW.operation])
  OR (NEW.outcome='COMMITTED' AND NEW.operation IN (2,3) AND
   (NEW.recipient_account_id IS DISTINCT FROM material.administrative_account_id
    OR (d.assignment_id IS NOT NULL AND (NEW.assignment_id,NEW.assignment_revision_before)
     IS DISTINCT FROM (d.assignment_id,d.expected_assignment_revision))
    OR (d.assignment_id IS NULL AND NEW.assignment_revision_before IS NOT NULL)
    OR (NEW.operation=2 AND (NEW.assignment_valid_from,NEW.assignment_valid_until)
     IS DISTINCT FROM (NEW.executed_at,d.expires_at)))) THEN
  RAISE EXCEPTION 'native_company_policy.receipt_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_participant_receipt_v1(p_table text,p_row jsonb) RETURNS uuid
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE receipt uuid; company uuid:=(p_row->>'org_id')::uuid;
BEGIN
 IF p_table IN ('policy_roles','policy_role_revisions','user_role_assignments','policy_assignment_revisions',
   'native_company_catalog_installs','ont_builtin_catalog_installs','ont_object_types') THEN
  receipt:=(p_row->>'policy_receipt_id')::uuid;
 ELSIF p_table='company_authority_heads' THEN receipt:=(p_row->>'current_policy_receipt_id')::uuid;
 ELSIF p_table IN ('policy_capability_clauses','policy_capability_clause_fields') THEN
  SELECT r.policy_receipt_id INTO receipt FROM public.policy_role_revisions r WHERE r.org_id=company
   AND r.role_id=(p_row->>'role_id')::uuid AND r.revision=(p_row->>'role_revision')::bigint;
 ELSIF p_table IN ('native_company_object_refs','native_company_action_refs','native_company_property_refs') THEN
  SELECT i.policy_receipt_id INTO receipt FROM public.native_company_catalog_installs i
   WHERE i.org_id=company AND i.catalog_version=p_row->>'catalog_version';
 ELSIF p_table IN ('ont_property_defs','ont_action_types') THEN
  SELECT o.policy_receipt_id INTO receipt FROM public.ont_object_types o
   WHERE o.org_id=company AND o.id=(p_row->>'object_type_id')::uuid;
 ELSIF p_table='ont_object_type_key_revisions' THEN
  IF p_row->>'stable_key' NOT IN ('pay_run','person') OR p_row->>'stable_key' IS NULL THEN RETURN NULL; END IF;
  IF NOT EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=company) THEN RETURN NULL; END IF;
  SELECT r.receipt_id INTO STRICT receipt FROM public.native_company_policy_receipts_v1 r WHERE r.org_id=company
   AND r.operation=1 AND r.outcome='COMMITTED'
   AND ((r.codec_version=1 AND p_row->>'stable_key'='pay_run') OR (r.codec_version=2 AND p_row->>'stable_key'='person'))
   AND r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid();
 ELSIF p_table NOT IN ('ont_link_types','ont_analytics','cedar_policy_catalog_entries','ont_object_policies') THEN
  RAISE EXCEPTION 'native_company_policy.participant_unavailable';
 END IF;
 RETURN receipt;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_participant_admit_v1(p_table text,p_op text,p_old jsonb,p_new jsonb) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE receipt uuid; r public.native_company_policy_receipts_v1; origin public.company_authority_heads;
 company uuid:=coalesce((p_new->>'org_id')::uuid,(p_old->>'org_id')::uuid); previous public.policy_assignment_revisions;
BEGIN
 IF p_op NOT IN ('INSERT','UPDATE','DELETE') THEN RAISE EXCEPTION 'native_company_policy.participant_unavailable'; END IF;
 receipt:=public.native_company_policy_participant_receipt_v1(p_table,coalesce(p_new,p_old));
 IF p_table='user_role_assignments' AND p_op='UPDATE' AND receipt IS NOT NULL THEN
  SELECT v.policy_receipt_id INTO STRICT receipt FROM public.policy_assignment_revisions v WHERE v.org_id=company
   AND v.assignment_id=(p_new->>'id')::uuid AND v.revision=(p_new->>'native_current_revision')::bigint;
 END IF;
 IF receipt IS NULL THEN RETURN false; END IF;
 r:=public.native_company_policy_effect_frame_v1(company,receipt);
 SELECT h.* INTO STRICT origin FROM public.company_authority_heads h WHERE h.org_id=company;
 IF p_op='DELETE' THEN RAISE EXCEPTION 'native_company_policy.immutable'; END IF;
 IF p_op='UPDATE' THEN
  IF p_table='company_authority_heads' THEN
   IF p_old-'epoch'-'current_policy_receipt_id' IS DISTINCT FROM p_new-'epoch'-'current_policy_receipt_id'
    OR (p_old->>'epoch')::bigint IS DISTINCT FROM r.epoch_before
    OR (p_old->>'current_policy_receipt_id')::uuid IS DISTINCT FROM r.predecessor_receipt_id
    OR (p_new->>'epoch')::bigint IS DISTINCT FROM r.epoch_after THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
  ELSIF p_table='user_role_assignments' THEN
   IF p_old-'native_current_revision' IS DISTINCT FROM p_new-'native_current_revision'
    OR (p_new->>'id')::uuid IS DISTINCT FROM r.assignment_id
    OR (p_old->>'native_current_revision')::bigint IS DISTINCT FROM r.assignment_revision_before
    OR (p_new->>'native_current_revision')::bigint IS DISTINCT FROM r.assignment_revision_after THEN
    RAISE EXCEPTION 'native_company_policy.effect_invalid';
   END IF;
  ELSE RAISE EXCEPTION 'native_company_policy.immutable'; END IF;
  RETURN true;
 END IF;
 IF p_table='company_authority_heads' THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 IF p_new?'origin_account_id' AND
  ((p_new->>'origin_account_id')::uuid,(p_new->>'origin_command_id')::uuid,(p_new->>'origin_receipt_id')::uuid)
   IS DISTINCT FROM (origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id) THEN
  RAISE EXCEPTION 'native_company_policy.origin_invalid';
 END IF;
 IF p_table IN ('policy_roles','policy_role_revisions','user_role_assignments','policy_assignment_revisions',
  'policy_capability_clauses','policy_capability_clause_fields') THEN
  IF r.operation NOT IN (2,3) OR (p_table<>'policy_assignment_revisions' AND r.assignment_revision_before IS NOT NULL)
   OR (p_new?'role_id' AND (p_new->>'role_id')::uuid IS DISTINCT FROM r.role_id)
   OR (p_table='policy_roles' AND (p_new->>'id')::uuid IS DISTINCT FROM r.role_id)
   OR (p_table='user_role_assignments' AND (p_new->>'id')::uuid IS DISTINCT FROM r.assignment_id)
   OR (p_table='policy_assignment_revisions' AND ((p_new->>'assignment_id')::uuid IS DISTINCT FROM r.assignment_id
     OR (p_new->>'revision')::bigint IS DISTINCT FROM r.assignment_revision_after)) THEN
   RAISE EXCEPTION 'native_company_policy.effect_invalid';
  END IF;
 ELSE
  IF r.operation<>1 OR (p_new?'object_type_id' AND (p_new->>'object_type_id')::uuid IS DISTINCT FROM r.installed_object_type_id)
   OR (p_table='ont_object_types' AND (p_new->>'id')::uuid IS DISTINCT FROM r.installed_object_type_id)
   OR (p_new?'catalog_version' AND p_new->>'catalog_version' IS DISTINCT FROM r.catalog_version)
   OR (p_table='ont_object_type_key_revisions' AND (p_new->>'stable_key' IS DISTINCT FROM (CASE r.codec_version WHEN 1 THEN 'pay_run' WHEN 2 THEN 'person' END) OR (p_new->>'revision')::bigint<>1)) THEN
   RAISE EXCEPTION 'native_company_policy.effect_invalid';
  END IF;
 END IF;
 IF (p_new?'created_at' AND (p_new->>'created_at')::timestamptz IS DISTINCT FROM r.executed_at)
  OR (p_new?'actor_account_id' AND (p_new->>'actor_account_id')::uuid IS DISTINCT FROM r.actor_account_id)
  OR (p_new?'session_id' AND (p_new->>'session_id')::uuid IS DISTINCT FROM r.execution_session_id) THEN
  RAISE EXCEPTION 'native_company_policy.effect_invalid';
 END IF;
 RETURN true;
END
$body$;
