-- Versioned trigger successors. Every historical branch is copied verbatim
-- from the pinned Company owner source; only the finite checked early branch
-- and function name differ. Original functions remain installed unchanged.

CREATE FUNCTION public.identity_native_root_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_value jsonb; b record;
BEGIN
 IF current_user IN ('console_account_owner','console_ontology_writer') THEN
 IF public.native_company_policy_participant_admit_v1(TG_TABLE_NAME,TG_OP,
   CASE WHEN TG_OP<>'INSERT' THEN to_jsonb(OLD) END,CASE WHEN TG_OP<>'DELETE' THEN to_jsonb(NEW) END) THEN
  RETURN NEW;
 END IF;
 END IF;
 IF TG_OP<>'INSERT' AND OLD.subject_protocol='NATIVE_ACCOUNT' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 IF TG_OP='UPDATE' AND NEW.subject_protocol IS DISTINCT FROM OLD.subject_protocol THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 IF NEW.subject_protocol='LEGACY_USER' THEN RETURN NEW; END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 row_value:=to_jsonb(NEW);
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.origin_account_id,NEW.origin_command_id);
 IF b.request_state<>'PENDING' OR NEW.org_id IS DISTINCT FROM b.org_id
  OR NEW.origin_receipt_id IS DISTINCT FROM b.receipt_id OR NEW.native_current_revision<>1
  OR NEW.created_at IS DISTINCT FROM b.started_at
  OR (TG_TABLE_NAME='policy_roles' AND ((row_value->>'role_key') IS DISTINCT FROM 'native_company_administration'
    OR (row_value->>'display_name') IS DISTINCT FROM '회사 초기 관리자' OR (row_value->>'description') IS NOT NULL
    OR (row_value->>'status') IS DISTINCT FROM 'ACTIVE' OR (row_value->>'is_system')::boolean IS DISTINCT FROM true
    OR (row_value->>'created_by_account_id')::uuid IS DISTINCT FROM b.account_id OR (row_value->>'updated_by_account_id')::uuid IS DISTINCT FROM b.account_id
    OR (row_value->>'updated_at')::timestamptz IS DISTINCT FROM b.started_at))
  OR (TG_TABLE_NAME='user_role_assignments' AND ((row_value->>'account_id')::uuid IS DISTINCT FROM b.administrative_account_id
    OR (row_value->>'assigned_by_account_id')::uuid IS DISTINCT FROM b.account_id)) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.identity_native_birth_row_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE value jsonb:=to_jsonb(NEW); parent record; b record; account uuid; command uuid; receipt uuid;
BEGIN
 IF current_user IN ('console_account_owner','console_ontology_writer') THEN
 IF public.native_company_policy_participant_admit_v1(TG_TABLE_NAME,TG_OP,
   CASE WHEN TG_OP<>'INSERT' THEN to_jsonb(OLD) END,CASE WHEN TG_OP<>'DELETE' THEN to_jsonb(NEW) END) THEN
  RETURN NEW;
 END IF;
 END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 IF TG_TABLE_NAME IN ('policy_capability_clauses','policy_capability_clause_fields') THEN
  SELECT r.* INTO STRICT parent FROM public.policy_role_revisions r
   WHERE r.org_id=NEW.org_id AND r.role_id=NEW.role_id AND r.revision=NEW.role_revision;
  account:=parent.origin_account_id; command:=parent.origin_command_id; receipt:=parent.origin_receipt_id;
 ELSE
  account:=NEW.origin_account_id; command:=NEW.origin_command_id; receipt:=NEW.origin_receipt_id;
 END IF;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(account,command);
 IF b.request_state<>'PENDING' OR b.org_id IS DISTINCT FROM NEW.org_id OR b.receipt_id IS DISTINCT FROM receipt
  OR (value?'revision' AND (value->>'revision')::bigint IS DISTINCT FROM 1)
  OR (value?'role_revision' AND (value->>'role_revision')::bigint IS DISTINCT FROM 1)
  OR (value?'epoch' AND (value->>'epoch')::bigint IS DISTINCT FROM 1)
  OR (value?'subject_protocol' AND value->>'subject_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT')
  OR (value?'state' AND value->>'state' IS DISTINCT FROM 'ACTIVE')
  OR (value?'account_id' AND (value->>'account_id')::uuid IS DISTINCT FROM b.administrative_account_id)
  OR (value?'actor_account_id' AND (value->>'actor_account_id')::uuid IS DISTINCT FROM b.account_id)
  OR (value?'session_id' AND (value->>'session_id')::uuid IS DISTINCT FROM b.session_id)
  OR (value?'created_at' AND (value->>'created_at')::timestamptz IS DISTINCT FROM b.started_at)
  OR (value?'valid_from' AND (value->>'valid_from')::timestamptz IS DISTINCT FROM b.started_at)
  OR (value?'valid_until' AND value->'valid_until' IS DISTINCT FROM 'null'::jsonb) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
END
$body$;

CREATE FUNCTION public.identity_native_birth_closure_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); value jsonb:=to_jsonb(NEW); account uuid; command uuid;
BEGIN
 IF public.native_company_policy_participant_closed_v1(TG_TABLE_NAME,to_jsonb(NEW)) THEN RETURN NEW; END IF;
 IF value?'subject_protocol' AND value->>'subject_protocol'='LEGACY_USER' THEN RETURN NEW; END IF;
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 IF TG_TABLE_NAME IN ('policy_capability_clauses','policy_capability_clause_fields') THEN
  SELECT r.origin_account_id,r.origin_command_id INTO STRICT account,command
   FROM public.policy_role_revisions r
   WHERE r.org_id=NEW.org_id AND r.role_id=NEW.role_id AND r.revision=NEW.role_revision;
 ELSE
  account:=NEW.origin_account_id; command:=NEW.origin_command_id;
 END IF;
 PERFORM public.company_enrollment_assert_closure_v1(account,command);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_company_catalog_birth_row_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE installation public.native_company_catalog_installs%ROWTYPE; b record; value jsonb:=to_jsonb(NEW); actual_digest bytea; actual_key text; parent public.ont_object_types%ROWTYPE;
BEGIN
 IF current_user IN ('console_account_owner','console_ontology_writer') THEN
 IF public.native_company_policy_participant_admit_v1(TG_TABLE_NAME,TG_OP,
   CASE WHEN TG_OP<>'INSERT' THEN to_jsonb(OLD) END,CASE WHEN TG_OP<>'DELETE' THEN to_jsonb(NEW) END) THEN
  RETURN NEW;
 END IF;
 END IF;
 IF current_user<>'console_ontology_writer' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_owner_required';
 END IF;
 IF TG_TABLE_NAME='native_company_catalog_installs' THEN
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.origin_account_id,NEW.origin_command_id);
  IF NEW.origin_receipt_id IS DISTINCT FROM b.receipt_id OR NEW.installed_at IS DISTINCT FROM b.started_at THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
 ELSE
  SELECT x.* INTO STRICT installation FROM public.native_company_catalog_installs x
   WHERE x.org_id=NEW.org_id AND x.catalog_version=NEW.catalog_version;
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(installation.origin_account_id,installation.origin_command_id);
  IF installation.origin_receipt_id IS DISTINCT FROM b.receipt_id OR installation.installed_at IS DISTINCT FROM b.started_at THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
 END IF;
 IF b.request_state IS DISTINCT FROM 'PENDING' OR NEW.org_id IS DISTINCT FROM b.org_id
  OR NEW.catalog_version IS DISTINCT FROM b.catalog_version OR NEW.manifest_digest IS DISTINCT FROM b.manifest_digest
  OR (value?'schema_revision' AND (value->>'schema_revision')::bigint IS DISTINCT FROM 1)
  OR (value?'registration_revision' AND (value->>'registration_revision')::bigint IS DISTINCT FROM 1) THEN
  RAISE EXCEPTION 'identity_native.invalid_birth';
 END IF;
 IF TG_TABLE_NAME<>'native_company_catalog_installs' THEN
  SELECT o.* INTO STRICT parent FROM public.ont_object_types o WHERE o.org_id=NEW.org_id AND o.id=NEW.object_type_id;
  IF parent.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR parent.schema_version IS DISTINCT FROM 1
   OR parent.lifecycle_state IS DISTINCT FROM 'published' OR parent.origin_account_id IS DISTINCT FROM b.account_id
   OR parent.origin_command_id IS DISTINCT FROM b.command_id OR parent.origin_receipt_id IS DISTINCT FROM b.receipt_id THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
  IF TG_TABLE_NAME='native_company_object_refs' THEN
   actual_key:=parent.stable_key;
   actual_digest:=sha256(convert_to(jsonb_build_object('stable_key',parent.stable_key,'title',parent.title,
    'title_property_key',parent.title_property_key,'backing_kind',parent.backing_kind,'backing_table',parent.backing_table,
    'primary_key_property',parent.primary_key_property,'schema_version',parent.schema_version,'lifecycle_state',parent.lifecycle_state)::text,'UTF8'));
  ELSIF TG_TABLE_NAME='native_company_action_refs' THEN
   SELECT a.dispatch_target,sha256(convert_to(jsonb_build_object('stable_key',a.stable_key,'title',a.title,
    'params_schema',a.params_schema,'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,
    'dispatch',a.dispatch,'dispatch_target',a.dispatch_target,'control_points',a.control_points)::text,'UTF8'))
   INTO STRICT actual_key,actual_digest FROM public.ont_action_types a
    WHERE a.org_id=NEW.org_id AND a.object_type_id=NEW.object_type_id AND a.id=NEW.action_type_id;
  ELSE
   SELECT CASE WHEN parent.stable_key='company_workspace' THEN 'company.' ELSE 'assignment.' END||p.key,
    sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,'config',p.config,
     'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8'))
   INTO STRICT actual_key,actual_digest FROM public.ont_property_defs p
    WHERE p.org_id=NEW.org_id AND p.object_type_id=NEW.object_type_id AND p.id=NEW.property_id;
  END IF;
  IF NEW.content_digest IS DISTINCT FROM actual_digest
   OR coalesce(value->>'object_key',value->>'action_key',value->>'property_key') IS DISTINCT FROM actual_key THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
 END IF;
 -- The complete fixed-manifest comparison and cardinality proof is deferred
 -- until all maps exist; no label is accepted in place of a physical identity.
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'identity_native.invalid_birth';
END
$body$;

CREATE FUNCTION public.native_company_catalog_birth_closure_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); installation public.native_company_catalog_installs%ROWTYPE;
BEGIN
 IF public.native_company_policy_participant_closed_v1(TG_TABLE_NAME,to_jsonb(NEW)) THEN RETURN NEW; END IF;
 PERFORM set_config('app.current_org',NEW.org_id::text,true); SELECT x.* INTO STRICT installation FROM public.native_company_catalog_installs x
  WHERE x.org_id=NEW.org_id AND x.catalog_version=NEW.catalog_version;
 PERFORM public.company_enrollment_assert_closure_v1(installation.origin_account_id,installation.origin_command_id);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION ontology_api.native_catalog_attribution_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE value jsonb; previous jsonb; organization public.organizations%ROWTYPE;
 parent public.ont_object_types%ROWTYPE; b record; previous_binding record; native boolean:=false; previous_native boolean:=false;
BEGIN
 IF current_user IN ('console_account_owner','console_ontology_writer') THEN
 IF public.native_company_policy_participant_admit_v1(TG_TABLE_NAME,TG_OP,
   CASE WHEN TG_OP<>'INSERT' THEN to_jsonb(OLD) END,CASE WHEN TG_OP<>'DELETE' THEN to_jsonb(NEW) END) THEN
  RETURN NEW;
 END IF;
 END IF;
 IF TG_OP<>'INSERT' THEN previous:=to_jsonb(OLD); END IF;
 IF TG_OP='DELETE' THEN value:=previous; ELSE value:=to_jsonb(NEW); END IF;
 IF current_user='console_ontology_writer' THEN
  SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1((value->>'org_id')::uuid);
  native:=b.account_id IS NOT NULL;
  IF TG_OP<>'INSERT' THEN
   SELECT * INTO previous_binding FROM public.company_enrollment_catalog_binding_v1((previous->>'org_id')::uuid);
   previous_native:=previous_binding.account_id IS NOT NULL;
  END IF;
 ELSE
  -- Retained direct legacy writers already have scoped organizations SELECT.
  -- They cannot execute the private binding helper or produce native rows.
  SELECT o.* INTO STRICT organization FROM public.organizations o WHERE o.id=(value->>'org_id')::uuid;
  native:=organization.origin_account_id IS NOT NULL;
  IF TG_OP<>'INSERT' THEN
   SELECT EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=(previous->>'org_id')::uuid
    AND o.origin_account_id IS NOT NULL) INTO previous_native;
  END IF;
 END IF;
 IF TG_OP<>'INSERT' AND (
   previous->>'attribution_protocol'='NATIVE_ACCOUNT' OR previous_native
   OR (value?'attribution_protocol' AND (
    value->'attribution_protocol' IS DISTINCT FROM previous->'attribution_protocol'
    OR value->'origin_account_id' IS DISTINCT FROM previous->'origin_account_id'
    OR value->'origin_command_id' IS DISTINCT FROM previous->'origin_command_id'
    OR value->'origin_receipt_id' IS DISTINCT FROM previous->'origin_receipt_id'))) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_operation_unavailable';
 END IF;
 IF NOT native THEN
  IF value?'attribution_protocol' AND value->>'attribution_protocol' IS DISTINCT FROM 'LEGACY_USER' THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
  IF TG_OP='DELETE' THEN RETURN OLD; END IF;
  RETURN NEW;
 END IF;
 IF current_user<>'console_ontology_writer' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_owner_required';
 END IF;
 IF b.request_state IS DISTINCT FROM 'PENDING' OR b.org_id IS DISTINCT FROM (value->>'org_id')::uuid
  OR (value?'created_at' AND (value->>'created_at')::timestamptz IS DISTINCT FROM b.started_at)
  OR (value?'updated_at' AND (value->>'updated_at')::timestamptz IS DISTINCT FROM b.started_at) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
 END IF;
 IF TG_TABLE_NAME='ont_object_type_key_revisions' THEN
  IF NEW.stable_key NOT IN ('company_workspace','company_policy_assignment') OR NEW.revision<>1
   OR NEW.validator_id IS NULL OR NEW.validator_id='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
 ELSIF TG_TABLE_NAME IN ('ont_property_defs','ont_link_types','ont_action_types','ont_analytics') THEN
  SELECT x.* INTO STRICT parent FROM public.ont_object_types x WHERE x.org_id=NEW.org_id AND x.id=NEW.object_type_id;
  IF parent.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
   OR parent.origin_account_id IS DISTINCT FROM b.account_id OR parent.origin_command_id IS DISTINCT FROM b.command_id
   OR parent.origin_receipt_id IS DISTINCT FROM b.receipt_id OR parent.created_by_account_id IS DISTINCT FROM b.account_id
   OR parent.created_at IS DISTINCT FROM b.started_at OR parent.updated_at IS DISTINCT FROM b.started_at
   OR parent.schema_version IS DISTINCT FROM 1 OR parent.lifecycle_state IS DISTINCT FROM 'published'
   OR parent.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
 ELSE
  IF value->>'attribution_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT'
   OR (value->>'origin_account_id')::uuid IS DISTINCT FROM b.account_id
   OR (value->>'origin_command_id')::uuid IS DISTINCT FROM b.command_id
   OR (value->>'origin_receipt_id')::uuid IS DISTINCT FROM b.receipt_id
   OR (value?'created_by' AND value->'created_by' IS DISTINCT FROM 'null'::jsonb)
   OR (value?'updated_by' AND value->'updated_by' IS DISTINCT FROM 'null'::jsonb)
   OR (value?'installed_by' AND value->'installed_by' IS DISTINCT FROM 'null'::jsonb)
   OR (value?'created_by_account_id' AND (value->>'created_by_account_id')::uuid IS DISTINCT FROM b.account_id)
   OR (value?'updated_by_account_id' AND (value->>'updated_by_account_id')::uuid IS DISTINCT FROM b.account_id)
   OR (value?'installed_by_account_id' AND (value->>'installed_by_account_id')::uuid IS DISTINCT FROM b.account_id) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
  IF TG_TABLE_NAME='ont_object_types' THEN
   IF NEW.stable_key NOT IN ('company_workspace','company_policy_assignment') OR NEW.schema_version<>1
    OR NEW.lifecycle_state IS DISTINCT FROM 'published' THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
   END IF;
  ELSIF TG_TABLE_NAME='ont_builtin_catalog_installs' THEN
   IF NEW.catalog_version IS DISTINCT FROM b.catalog_version OR NEW.manifest_digest IS DISTINCT FROM b.manifest_digest
    OR NEW.installed_at IS DISTINCT FROM b.started_at
    OR (SELECT count(*) FROM public.ont_object_types o WHERE o.org_id=b.org_id)<>2 THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
   END IF;
  ELSIF TG_TABLE_NAME NOT IN ('cedar_policy_catalog_entries','ont_object_policies') THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
END
$body$;

CREATE FUNCTION public.company_enrollment_ontology_audit_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; native_target boolean;
BEGIN
 -- The separate ALWAYS native policy audit guard independently requires its exact live frame.
 IF NEW.action='ontology.object_type.builtin_install' AND NEW.after_snap->>'catalog_version'='native-payroll-collection-read-v1' THEN RETURN NEW; END IF;
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

CREATE FUNCTION ontology_api.protected_audit_writer_guard_v2()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_invoker NAME := ontology_api.invoker_role();
    v_stable_key TEXT;
    b record; o record; snapshot jsonb; target text;
    native_object uuid; native_action text;
    prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF public.native_company_policy_audit_admit_v1(NEW) THEN RETURN NEW; END IF;
    IF NEW.action IN ('ontology.object_type.create','ontology.object_type.stage_revision',
        'ontology.object_type.transition','ontology.object_type.builtin_install','ontology.object_policy.attach') THEN
      SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
      IF b.account_id IS NOT NULL THEN
       BEGIN
        IF NEW.action NOT IN ('ontology.object_type.builtin_install','ontology.object_policy.attach')
          OR b.request_state IS DISTINCT FROM 'PENDING' OR b.org_id IS DISTINCT FROM NEW.org_id
          OR NEW.actor IS DISTINCT FROM b.account_id OR NEW.occurred_at IS DISTINCT FROM b.started_at
          OR NEW.before_snap IS NOT NULL OR NEW.branch_id IS NOT NULL
          OR NEW.trace_id IS NULL OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
          OR NEW.span_id IS NULL OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        native_object:=NEW.target_id::uuid; native_action:=NEW.action;
        PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=native_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 IF native_action='ontology.object_type.builtin_install' THEN
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 ELSE
  target:='ont_object_policies';
  SELECT c.normalized_row INTO STRICT snapshot
   FROM public.ont_object_policies p JOIN public.cedar_policy_catalog_entries c ON c.org_id=p.org_id AND c.id=p.cedar_policy_id
   WHERE p.org_id=b.org_id AND p.object_type_id=native_object AND p.attribution_protocol='NATIVE_ACCOUNT'
    AND p.origin_account_id=b.account_id AND p.origin_command_id=b.command_id AND p.origin_receipt_id=b.receipt_id
    AND p.created_by_account_id=b.account_id AND p.created_at=b.started_at
    AND c.attribution_protocol='NATIVE_ACCOUNT' AND c.origin_account_id=b.account_id AND c.origin_command_id=b.command_id
    AND c.origin_receipt_id=b.receipt_id AND c.created_by_account_id=b.account_id AND c.updated_by_account_id=b.account_id
    AND c.created_at=b.started_at AND c.updated_at=b.started_at;
  IF snapshot IS DISTINCT FROM jsonb_build_object('effect','forbid','action','view','resource_type',o.stable_key,'conditions','[]'::jsonb) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 END IF;
        IF NEW.target_type IS DISTINCT FROM target OR NEW.after_snap IS DISTINCT FROM
          snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
            'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)) THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
       EXCEPTION WHEN OTHERS THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
       END;
      ELSIF coalesce(NEW.after_snap?'enrollment',false) THEN
        RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
      END IF;
    END IF;
    IF NEW.action <> ALL (ARRAY[
        'ontology.object_type.create',
        'ontology.object_type.stage_revision',
        'ontology.object_type.transition',
        'ontology.object_type.builtin_install'
    ]::TEXT[]) THEN
        RETURN NEW;
    END IF;

    -- Direct command credentials cannot INSERT audit_events. When an approved
    -- command reaches this trigger it is nested inside the writer-owned
    -- SECURITY DEFINER routine, while the old compatibility path arrives as
    -- console_rt and must prove a matching parent mutation in this transaction.
    IF v_invoker = 'console_rt'::NAME THEN
        IF NEW.action = 'ontology.object_type.builtin_install'
           AND public.platform_legacy_catalog_live_audit_v1(NEW) THEN
            RETURN NEW;
        END IF;
        IF NEW.action = 'ontology.object_type.builtin_install'
           OR NEW.target_type <> 'ont_object_types'
           OR NOT EXISTS (
               SELECT 1
               FROM public.users u
               WHERE u.id = NEW.actor AND u.org_id = NEW.org_id AND u.is_active
           ) THEN
            RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_audit.command_required';
        END IF;

        SELECT parent_type.stable_key INTO v_stable_key
        FROM public.ont_object_types parent_type
        WHERE parent_type.org_id = NEW.org_id
          AND parent_type.id::TEXT = NEW.target_id
          AND parent_type.updated_at = NEW.occurred_at
          AND parent_type.xmin = pg_catalog.pg_current_xact_id()::xid
          AND (
              (NEW.action = 'ontology.object_type.create' AND parent_type.schema_version = 1 AND parent_type.created_at = NEW.occurred_at)
              OR (NEW.action = 'ontology.object_type.stage_revision' AND parent_type.schema_version > 1 AND parent_type.created_at = NEW.occurred_at)
              OR NEW.action = 'ontology.object_type.transition'
          );
        IF v_stable_key IS NULL THEN
            RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_audit.command_required';
        END IF;
        IF NEW.action IN ('ontology.object_type.stage_revision', 'ontology.object_type.transition') THEN
            UPDATE public.ont_object_type_key_revisions k
               SET revision = k.revision + 1, updated_at = NEW.occurred_at
             WHERE k.org_id = NEW.org_id AND k.stable_key = v_stable_key;
            IF NOT FOUND THEN
                RAISE EXCEPTION USING ERRCODE = '23503', MESSAGE = 'ontology_legacy.key_revision_missing';
            END IF;
        END IF;
    ELSIF v_invoker <> 'console_ontology_cmd'::NAME THEN
        RAISE EXCEPTION USING
            ERRCODE = '42501',
            MESSAGE = 'ontology_audit.command_required';
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION ontology_api.require_current_transaction_audit_v2()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_parent_id UUID;
    v_org_id UUID;
    v_stable_key TEXT;
    v_parent_is_current BOOLEAN;
    b record; o record; snapshot jsonb; target text;
    native_object uuid; native_action text;
    prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF public.native_company_policy_participant_closed_v1(TG_TABLE_NAME,to_jsonb(NEW)) THEN RETURN NEW; END IF;
    -- Scope comes from the persisted trigger row, never the caller's ambient Company.
    PERFORM set_config('app.current_org',NEW.org_id::text,true);
    SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
    IF b.account_id IS NOT NULL THEN
        IF b.request_state IS DISTINCT FROM 'COMMITTED' OR b.org_id IS DISTINCT FROM NEW.org_id THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        IF TG_TABLE_NAME='ont_object_types' THEN native_object:=NEW.id;
        ELSE native_object:=NEW.object_type_id; END IF;
        PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=native_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
        IF (SELECT count(*) FROM public.audit_events e WHERE e.org_id=b.org_id
          AND e.action='ontology.object_type.builtin_install' AND e.target_id=native_object::text
          AND e.target_type=target AND e.actor=b.account_id AND e.occurred_at=b.started_at
          AND e.before_snap IS NULL AND e.branch_id IS NULL
          AND e.after_snap=snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
            'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)))<>1 THEN
          RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='ontology_write.exactly_one_current_transaction_audit_required';
        END IF;
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
    END IF;
    IF TG_TABLE_NAME = 'ont_object_types' THEN
        v_parent_id := NEW.id;
        v_org_id := NEW.org_id;
        v_stable_key := NEW.stable_key;
        v_parent_is_current := TRUE;
    ELSE
        v_parent_id := NEW.object_type_id;
        v_org_id := NEW.org_id;
        SELECT parent_type.stable_key,
               parent_type.xmin = pg_catalog.pg_current_xact_id()::xid
          INTO v_stable_key, v_parent_is_current
          FROM public.ont_object_types parent_type
         WHERE parent_type.id = v_parent_id AND parent_type.org_id = v_org_id;
    END IF;

    IF COALESCE(v_parent_is_current, FALSE) AND TG_TABLE_NAME <> 'ont_object_types' THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
    END IF;
    IF (
        SELECT COUNT(*) = 1
        FROM public.audit_events e
        LEFT JOIN public.ont_object_types target
          ON target.org_id = e.org_id AND target.id::TEXT = e.target_id
        WHERE e.org_id = v_org_id
          AND e.action = ANY (ARRAY[
              'ontology.object_type.create',
              'ontology.object_type.stage_revision',
              'ontology.object_type.transition',
              'ontology.object_type.builtin_install'
          ]::TEXT[])
          AND (e.xmin = pg_catalog.pg_current_xact_id()::xid
               OR (e.action = 'ontology.object_type.builtin_install'
                   AND public.platform_legacy_catalog_receipt_audit_v1(e)))
          AND (
              e.target_id = v_parent_id::TEXT
              OR (e.action = 'ontology.object_type.transition' AND target.stable_key = v_stable_key)
          )
    ) THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
    END IF;
    RAISE EXCEPTION USING
        ERRCODE = '23514',
        MESSAGE = 'ontology_write.exactly_one_current_transaction_audit_required';
EXCEPTION WHEN OTHERS THEN
    PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END;
$$;
