-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_ontology_snapshot_v1(r public.native_company_policy_receipts_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT jsonb_build_object('stable_key',CASE r.codec_version WHEN 1 THEN 'pay_run' WHEN 2 THEN 'person' END,'schema_version',1,'lifecycle_state','published',
 'catalog_version',r.catalog_version,'manifest_digest',encode(r.manifest_digest,'hex')) $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE admitted boolean;
BEGIN
 IF TG_TABLE_NAME<>'audit_events' OR TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_company_policy.audit_frame_invalid'; END IF;
 admitted:=public.native_company_policy_audit_admit_v1(NEW);
 IF NOT admitted AND (NEW.action IN ('policy.company_command.accept','policy.company_command.complete')
  OR (NEW.action='ontology.object_type.builtin_install'
    AND NEW.after_snap->>'catalog_version' IN ('native-payroll-collection-read-v1','native-people-directory-v1'))) THEN
  RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.company_enrollment_ontology_audit_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; native_target boolean;
BEGIN
 -- The separate ALWAYS native policy audit guard independently requires its exact live frame.
 IF NEW.action='ontology.object_type.builtin_install' AND NEW.after_snap->>'catalog_version' IN ('native-payroll-collection-read-v1','native-people-directory-v1') THEN RETURN NEW; END IF;
 IF NEW.action NOT IN ('ontology.object_type.builtin_install','ontology.object_policy.attach') THEN RETURN NEW; END IF;
 SELECT EXISTS(SELECT 1 FROM public.ont_object_types o WHERE o.org_id=NEW.org_id
  AND o.id::text=NEW.target_id AND o.attribution_protocol='NATIVE_ACCOUNT') INTO native_target;
 IF NOT native_target AND NOT coalesce(NEW.after_snap?'enrollment',false) THEN RETURN NEW; END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
 END IF;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.actor,(NEW.after_snap->'enrollment'->>'command_id')::uuid);
 IF b.org_id IS DISTINCT FROM NEW.org_id OR b.request_state<>'PENDING' OR NOT native_target
  OR NEW.occurred_at IS DISTINCT FROM b.started_at OR NEW.before_snap IS NOT NULL
  OR NEW.after_snap->'enrollment' IS DISTINCT FROM jsonb_build_object('account_id',b.account_id::text,
   'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)
  OR NEW.target_type IS DISTINCT FROM (CASE WHEN NEW.action='ontology.object_type.builtin_install' THEN 'ont_object_types' ELSE 'ont_object_policies' END) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
 END IF;
 RETURN NEW;
END
$body$;
