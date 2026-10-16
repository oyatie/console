-- Additive dual-codec successor; installed only after exact predecessor custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_operation_check_v1(p_company uuid,p_input bytea,p_at timestamptz)
RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d record; h public.company_authority_heads; b public.company_enrollment_receipts;
 a public.user_role_assignments; ar public.policy_assignment_revisions; r public.policy_roles;
BEGIN
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(public.native_company_policy_codec_v2(p_input),p_input);
 SELECT x.* INTO STRICT h FROM public.company_authority_heads x WHERE x.org_id=p_company;
 SELECT x.* INTO STRICT b FROM public.company_enrollment_receipts x WHERE x.org_id=p_company
  AND x.account_id=h.origin_account_id AND x.command_id=h.origin_command_id AND x.receipt_id=h.origin_receipt_id;
 IF d.org_id<>p_company THEN RAISE EXCEPTION 'native_company_policy.invalid_input'; END IF;
 IF d.expected_company_epoch<>h.epoch OR h.epoch=9223372036854775807 THEN RETURN 'revision_conflict'; END IF;
 IF d.operation=1 THEN
  IF EXISTS(SELECT 1 FROM public.native_company_catalog_installs x WHERE x.org_id=p_company
    AND x.catalog_version=d.catalog_version) THEN
   RAISE EXCEPTION 'native_company_policy.catalog_already_installed';
  END IF;
  RETURN NULL;
 END IF;
 IF NOT EXISTS(SELECT 1 FROM public.native_company_catalog_installs x WHERE x.org_id=p_company
   AND x.catalog_version=d.catalog_version) THEN
  RAISE EXCEPTION 'native_company_policy.catalog_required';
 END IF;
 SELECT x.* INTO r FROM public.policy_roles x WHERE x.org_id=p_company AND x.role_key=d.role_key;
 IF r.id IS NOT NULL THEN
  SELECT x.* INTO STRICT a FROM public.user_role_assignments x
   WHERE x.org_id=p_company AND x.role_id=r.id AND x.account_id=b.administrative_account_id;
  SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x
   WHERE x.org_id=p_company AND x.assignment_id=a.id AND x.revision=a.native_current_revision;
  IF r.subject_protocol<>'NATIVE_ACCOUNT' OR r.native_current_revision<>1 OR r.policy_receipt_id IS NULL
   OR a.subject_protocol<>'NATIVE_ACCOUNT' OR a.policy_receipt_id IS NULL
   OR ar.policy_receipt_id IS NULL OR ar.role_id<>r.id OR ar.role_revision<>1 THEN
   RAISE EXCEPTION 'native_company_policy.material_unavailable';
  END IF;
 END IF;
 IF d.assignment_id IS NULL THEN
  IF r.id IS NOT NULL THEN RETURN 'revision_conflict'; END IF;
 ELSE
  IF a.id IS DISTINCT FROM d.assignment_id OR a.native_current_revision IS DISTINCT FROM d.expected_assignment_revision
   OR a.native_current_revision=9223372036854775807 THEN RETURN 'revision_conflict'; END IF;
 END IF;
 IF d.operation=2 THEN
  IF d.recipient_account_id<>b.administrative_account_id THEN
   RAISE EXCEPTION 'native_company_policy.recipient_not_admitted';
  END IF;
  IF ar.state='ACTIVE' AND ar.valid_until>p_at THEN
   RAISE EXCEPTION 'native_company_policy.assignment_active';
  END IF;
  IF d.expires_at<=p_at OR d.expires_at>p_at+interval '30 days' THEN RETURN 'grant_expiry_invalid'; END IF;
  IF NOT EXISTS(SELECT 1 FROM public.account_security s WHERE s.account_id=b.administrative_account_id
     AND s.security_state='ACTIVE') THEN RETURN 'recipient_ineligible'; END IF;
  PERFORM 1 FROM public.account_login_consent_v1(b.administrative_account_id);
  IF NOT EXISTS(SELECT 1 FROM public.user_role_assignments x JOIN public.policy_assignment_revisions v
     ON v.org_id=x.org_id AND v.assignment_id=x.id AND v.revision=x.native_current_revision
    WHERE x.org_id=p_company AND x.id=b.root_assignment_id AND x.account_id=b.administrative_account_id
     AND x.native_current_revision=1 AND x.policy_receipt_id IS NULL AND v.state='ACTIVE'
     AND v.valid_from<=p_at AND (v.valid_until IS NULL OR v.valid_until>p_at)) THEN
   RETURN 'recipient_ineligible';
  END IF;
 ELSE
  IF ar.state IS DISTINCT FROM 'ACTIVE' THEN RAISE EXCEPTION 'native_company_policy.assignment_not_active'; END IF;
 END IF;
 RETURN NULL;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_prepare_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_operation smallint,p_input bytea)
RETURNS TABLE(inserted boolean,accepted_input public.native_company_policy_inputs_v1,terminal public.native_company_policy_receipts_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); material record; capacity record; d record;
 at_time timestamptz; failure text; error_schema text; error_table text; error_constraint text;
BEGIN
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(p_account,p_family,p_company,p_command,p_operation,p_input);
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(public.native_company_policy_codec_v2(p_input),p_input);
 PERFORM pg_advisory_xact_lock(hashtextextended('console.company.business-policy.admission/1',0));
 PERFORM pg_advisory_xact_lock(hashtextextended('console.company.business-policy.command/1:'||p_account::text||':'||p_command::text,0));
 SELECT i.* INTO accepted_input FROM public.native_company_policy_inputs_v1 i
  WHERE i.actor_account_id=p_account AND i.command_id=p_command;
 IF accepted_input.actor_account_id IS NOT NULL THEN
  IF accepted_input.org_id<>p_company OR accepted_input.operation<>p_operation OR accepted_input.input_bytes<>p_input THEN
   RAISE EXCEPTION 'native_company_policy.conflict';
  END IF;
  SELECT r.* INTO terminal FROM public.native_company_policy_receipts_v1 r WHERE r.actor_account_id=p_account AND r.command_id=p_command;
  inserted:=false;
 ELSE
  at_time:=clock_timestamp();
  failure:=public.native_company_policy_operation_check_v1(p_company,p_input,at_time);
  IF failure IS NOT NULL THEN RAISE EXCEPTION 'native_company_policy.%',failure; END IF;
  BEGIN
   INSERT INTO public.native_company_policy_inputs_v1(actor_account_id,command_id,org_id,operation,codec_version,
    input_bytes,input_digest,intake_receipt_id,accepted_at,execution_not_after,accepting_session_id,acceptance_xid,acceptance_backend_pid)
   VALUES(p_account,p_command,p_company,p_operation,d.codec_version,p_input,sha256(p_input),gen_random_uuid(),at_time,
    at_time+interval '168 hours',p_family,pg_current_xact_id(),pg_backend_pid()) RETURNING * INTO accepted_input;
  EXCEPTION WHEN unique_violation THEN
   GET STACKED DIAGNOSTICS error_schema=SCHEMA_NAME,error_table=TABLE_NAME,error_constraint=CONSTRAINT_NAME;
   IF error_schema='public' AND error_table='native_company_policy_inputs_v1'
    AND error_constraint='native_company_policy_inputs_v1_pkey' THEN
    RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='native_company_policy.conflict';
   END IF;
   RAISE;
  END;
  SELECT * INTO STRICT capacity FROM public.native_company_policy_capacity_v1(p_account,p_command,at_time);
  IF capacity.actor_live_count>=16 OR capacity.deployment_live_count>=256 THEN
   RAISE EXCEPTION 'native_company_policy.capacity_exceeded';
  END IF;
  inserted:=true;
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEXT;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_execute_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_operation smallint)
RETURNS TABLE(inserted boolean,terminal public.native_company_policy_receipts_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); material record; d record;
 accepted public.native_company_policy_inputs_v1; failure text; at_time timestamptz;
 business_role public.policy_roles; assignment public.user_role_assignments; previous public.policy_assignment_revisions;
BEGIN
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(p_account,p_family,p_company,p_command,p_operation,NULL);
 PERFORM set_config('app.current_org',p_company::text,true);
 PERFORM pg_advisory_xact_lock(hashtextextended('console.company.business-policy.command/1:'||p_account::text||':'||p_command::text,0));
 SELECT i.* INTO accepted FROM public.native_company_policy_inputs_v1 i WHERE i.actor_account_id=p_account AND i.command_id=p_command;
 IF accepted.actor_account_id IS NULL THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF accepted.org_id<>p_company OR accepted.operation<>p_operation THEN RAISE EXCEPTION 'native_company_policy.conflict'; END IF;
 SELECT r.* INTO terminal FROM public.native_company_policy_receipts_v1 r WHERE r.actor_account_id=p_account AND r.command_id=p_command;
 IF terminal.actor_account_id IS NOT NULL THEN
  inserted:=false;
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT; RETURN;
 END IF;
 IF accepted.acceptance_xid=pg_current_xact_id() THEN RAISE EXCEPTION 'native_company_policy.separate_commit_required'; END IF;
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(accepted.codec_version,accepted.input_bytes);
 at_time:=clock_timestamp();
 IF at_time>=accepted.execution_not_after THEN failure:='intake_expired';
 ELSE failure:=public.native_company_policy_operation_check_v1(p_company,accepted.input_bytes,at_time); END IF;
 terminal.actor_account_id:=p_account; terminal.command_id:=p_command; terminal.org_id:=p_company;
 terminal.operation:=p_operation; terminal.codec_version:=d.codec_version; terminal.intake_receipt_id:=accepted.intake_receipt_id;
 terminal.input_digest:=accepted.input_digest; terminal.receipt_id:=gen_random_uuid();
 terminal.outcome:=CASE WHEN failure IS NULL THEN 'COMMITTED' ELSE 'REJECTED' END;
 terminal.result_code:=coalesce(failure,(ARRAY['installed','granted','revoked'])[p_operation]);
 terminal.execution_session_id:=p_family; terminal.executed_at:=at_time;
 terminal.effect_xid:=pg_current_xact_id(); terminal.effect_backend_pid:=pg_backend_pid();
 terminal.epoch_before:=material.company_epoch;
 terminal.epoch_after:=CASE WHEN failure IS NULL THEN material.company_epoch+1 ELSE material.company_epoch END;
 terminal.catalog_version:=d.catalog_version; terminal.manifest_digest:=d.manifest_digest;
 terminal.predecessor_receipt_id:=material.current_policy_receipt_id;
 IF failure IS NULL THEN
  IF p_operation=1 THEN terminal.installed_object_type_id:=gen_random_uuid();
  ELSE
   SELECT r.* INTO business_role FROM public.policy_roles r WHERE r.org_id=p_company AND r.role_key=d.role_key;
   IF business_role.id IS NULL THEN
    terminal.role_id:=gen_random_uuid(); terminal.assignment_id:=gen_random_uuid();
    terminal.assignment_revision_after:=1;
   ELSE
    SELECT a.* INTO STRICT assignment FROM public.user_role_assignments a WHERE a.org_id=p_company
     AND a.role_id=business_role.id AND a.account_id=material.administrative_account_id;
    SELECT r.* INTO STRICT previous FROM public.policy_assignment_revisions r
     WHERE r.org_id=p_company AND r.assignment_id=assignment.id AND r.revision=assignment.native_current_revision;
    terminal.role_id:=business_role.id; terminal.assignment_id:=assignment.id;
    terminal.assignment_revision_before:=assignment.native_current_revision;
    terminal.assignment_revision_after:=assignment.native_current_revision+1;
   END IF;
   terminal.recipient_account_id:=material.administrative_account_id; terminal.role_revision:=1;
   terminal.assignment_state_after:=CASE WHEN p_operation=2 THEN 'ACTIVE' ELSE 'REVOKED' END;
   terminal.assignment_valid_from:=CASE WHEN p_operation=2 THEN at_time ELSE previous.valid_from END;
   terminal.assignment_valid_until:=CASE WHEN p_operation=2 THEN d.expires_at ELSE previous.valid_until END;
  END IF;
 END IF;
 INSERT INTO public.native_company_policy_receipts_v1(actor_account_id,command_id,org_id,operation,codec_version,
  intake_receipt_id,input_digest,receipt_id,outcome,result_code,execution_session_id,executed_at,effect_xid,effect_backend_pid,
  epoch_before,epoch_after,catalog_version,manifest_digest,predecessor_receipt_id,installed_object_type_id,
  recipient_account_id,role_id,role_revision,assignment_id,assignment_revision_before,assignment_revision_after,
  assignment_state_after,assignment_valid_from,assignment_valid_until)
 VALUES(terminal.actor_account_id,terminal.command_id,terminal.org_id,terminal.operation,terminal.codec_version,
  terminal.intake_receipt_id,terminal.input_digest,terminal.receipt_id,terminal.outcome,terminal.result_code,
  terminal.execution_session_id,terminal.executed_at,terminal.effect_xid,terminal.effect_backend_pid,
  terminal.epoch_before,terminal.epoch_after,terminal.catalog_version,terminal.manifest_digest,terminal.predecessor_receipt_id,
  terminal.installed_object_type_id,terminal.recipient_account_id,terminal.role_id,terminal.role_revision,
  terminal.assignment_id,terminal.assignment_revision_before,terminal.assignment_revision_after,
  terminal.assignment_state_after,terminal.assignment_valid_from,terminal.assignment_valid_until)
 RETURNING * INTO terminal;
 IF failure IS NULL THEN
  IF p_operation=1 THEN
   IF d.codec_version=1 THEN PERFORM ontology_api.install_native_company_payroll_catalog_v1(p_company,terminal.receipt_id);
   ELSE PERFORM ontology_api.install_native_company_people_catalog_v1(p_company,terminal.receipt_id); END IF;
  ELSE PERFORM public.native_company_policy_apply_assignment_v1(p_company,terminal.receipt_id); END IF;
  UPDATE public.company_authority_heads h SET epoch=terminal.epoch_after,current_policy_receipt_id=terminal.receipt_id
   WHERE h.org_id=p_company AND h.epoch=terminal.epoch_before
    AND h.current_policy_receipt_id IS NOT DISTINCT FROM terminal.predecessor_receipt_id;
  IF NOT FOUND THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 END IF;
 inserted:=true;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEXT;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_form_v2(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_operation smallint,p_codec smallint,p_action text)
RETURNS TABLE(installed_object_type_id uuid,role_id uuid,role_revision bigint,
 assignment_id uuid,assignment_revision bigint,assignment_state text,
 assignment_valid_from timestamptz,assignment_valid_until timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); material record; catalog text; selected_object_key text; selected_role text;
 role_row public.policy_roles; a public.user_role_assignments; ar public.policy_assignment_revisions;
BEGIN
 IF p_codec=1 AND p_action IS NOT DISTINCT FROM 'payroll.collection.read' THEN
  catalog:='native-payroll-collection-read-v1'; selected_object_key:='pay_run'; selected_role:='native_payroll_collection_read';
 ELSIF p_codec=2 AND ((p_operation=1 AND p_action IS NULL)
   OR (p_operation IN (2,3) AND p_action IN ('people.directory.read','people.directory.create'))) THEN
  catalog:='native-people-directory-v1'; selected_object_key:='person';
  selected_role:=CASE p_action WHEN 'people.directory.read' THEN 'native_people_directory_read'
   WHEN 'people.directory.create' THEN 'native_people_directory_create' END;
 ELSE RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(p_account,p_family,p_company,p_command,p_operation,NULL);
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.object_type_id INTO installed_object_type_id FROM public.native_company_object_refs o
  WHERE o.org_id=p_company AND o.catalog_version=catalog AND o.object_key=selected_object_key;
 SELECT r.* INTO role_row FROM public.policy_roles r WHERE r.org_id=p_company AND r.role_key=selected_role FOR SHARE;
 IF installed_object_type_id IS NULL AND role_row.id IS NOT NULL THEN RAISE EXCEPTION 'native_company_policy.form_custody_invalid'; END IF;
 IF role_row.id IS NOT NULL THEN
  SELECT x.* INTO STRICT a FROM public.user_role_assignments x WHERE x.org_id=p_company AND x.role_id=role_row.id
   AND x.account_id=material.administrative_account_id FOR SHARE;
  SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x WHERE x.org_id=p_company
   AND x.assignment_id=a.id AND x.revision=a.native_current_revision;
  PERFORM public.native_company_policy_assert_effects_v1(p_company,ar.policy_receipt_id);
  PERFORM public.native_company_policy_business_clauses_v1(p_company,role_row.id);
  IF ar.state NOT IN ('ACTIVE','REVOKED') OR ar.valid_until IS NULL OR NOT isfinite(ar.valid_until)
   OR ar.valid_from IS NULL OR NOT isfinite(ar.valid_from) OR ar.valid_until<=ar.valid_from THEN
   RAISE EXCEPTION 'native_company_policy.form_custody_invalid';
  END IF;
  role_id:=role_row.id; role_revision:=role_row.native_current_revision;
  assignment_id:=a.id; assignment_revision:=a.native_current_revision; assignment_state:=ar.state;
  assignment_valid_from:=ar.valid_from; assignment_valid_until:=ar.valid_until;
 ELSIF EXISTS(SELECT 1 FROM public.user_role_assignments x JOIN public.policy_roles r ON r.org_id=x.org_id AND r.id=x.role_id
   WHERE x.org_id=p_company AND r.role_key=selected_role AND x.policy_receipt_id IS NOT NULL) THEN
  RAISE EXCEPTION 'native_company_policy.form_custody_invalid';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEXT;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE EXCEPTION 'native_company_policy.form_custody_invalid';
WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_form_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_operation smallint)
RETURNS TABLE(installed_object_type_id uuid,role_id uuid,role_revision bigint,
 assignment_id uuid,assignment_revision bigint,assignment_state text,
 assignment_valid_from timestamptz,assignment_valid_until timestamptz)
LANGUAGE sql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$ SELECT * FROM public.native_company_policy_form_v2(
 p_account,p_family,p_company,p_command,p_operation,1::smallint,'payroll.collection.read') $body$;
