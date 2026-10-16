-- Generated UNINSTALLED Company-information manager Current source; not a custody finalizer.
-- No installed profile or serving readiness is asserted by this artifact.
-- source: ops/native-company-information/group-lock-v1.sql
-- UNINSTALLED additive private lock source. Exact successor custody is required.
-- This primitive retains only the selected Group head and Group row, before
-- Account/family. It neither locks the Company row nor establishes a permit.
CREATE FUNCTION public.identity_company_information_group_lock_v1(p_company uuid,p_group uuid)
RETURNS TABLE(group_row jsonb,group_head_row jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid;
 head record; selected_group public.groups;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_company IS NULL OR p_group IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 IF NOT FOUND OR planned_group IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 SELECT * INTO STRICT head FROM public.group_authority_lock_shared_v1(p_group);
 SELECT g.* INTO STRICT selected_group FROM public.groups g WHERE g.id=p_group FOR SHARE OF g;
 -- Do not acquire a Company lock in this Group-class primitive.
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 IF NOT FOUND OR planned_group IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 IF head.revision IS NULL OR head.revision<1 OR head.incarnation IS NULL
  OR head.incarnation='00000000-0000-0000-0000-000000000000'::uuid
  OR head.state NOT IN('ACTIVE','RETIRED') OR head.state IS NULL
  OR selected_group.id IS DISTINCT FROM p_group
  OR selected_group.status NOT IN('ACTIVE','SUSPENDED','ARCHIVED') OR selected_group.status IS NULL
  OR num_nonnulls(selected_group.origin_account_id,selected_group.origin_command_id,
      selected_group.origin_receipt_id) NOT IN(0,3)
  OR selected_group.created_at IS NULL OR selected_group.updated_at IS NULL
  OR NOT isfinite(selected_group.created_at) OR NOT isfinite(selected_group.updated_at)
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(selected_group.origin_account_id,
      selected_group.origin_command_id,selected_group.origin_receipt_id) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 RETURN QUERY SELECT to_jsonb(selected_group),jsonb_build_object('group_id',p_group,
  'revision',head.revision,'incarnation',head.incarnation,'state',head.state);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: ops/native-company-information/selected-lock-v1.sql
-- UNINSTALLED private selected-row guard. No sibling/founding Company scan.
-- The manager caller already retains selected Group head+row, then Account/family.
-- This topology-owned helper supplies only the next requested Company guard class.
CREATE FUNCTION public.identity_company_information_selected_lock_v1(p_company uuid,p_group uuid)
RETURNS TABLE(company_row jsonb,membership_row jsonb,membership_revision_row jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); selected_company record;
 member public.group_memberships; history public.group_membership_revisions;
 receipt public.platform_legacy_topology_receipts; decoded record; head public.group_authority_heads;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_company IS NULL OR p_group IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.id,o.group_id,o.slug,o.name,o.status,o.created_at,o.updated_at,
  o.origin_account_id,o.origin_command_id,o.origin_receipt_id INTO STRICT selected_company
  FROM public.organizations o WHERE o.id=p_company FOR SHARE OF o;
 IF selected_company.group_id IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 SELECT m.* INTO STRICT member FROM public.group_memberships m
  WHERE m.org_id=p_company AND m.group_id=p_group FOR SHARE OF m;
 SELECT h.* INTO STRICT history FROM public.group_membership_revisions h
  WHERE (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)=
   (member.group_id,member.org_id,member.membership_id,member.current_revision,member.incarnation)
  FOR SHARE OF h;
 IF selected_company.status IS NULL OR selected_company.status NOT IN('ACTIVE','SUSPENDED','ARCHIVED')
  OR selected_company.created_at IS NULL OR selected_company.updated_at IS NULL
  OR NOT isfinite(selected_company.created_at) OR NOT isfinite(selected_company.updated_at)
  OR selected_company.updated_at<selected_company.created_at
  OR num_nonnulls(selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id) NOT IN(0,3)
  OR member.current_revision IS NULL OR member.current_revision<1
  OR member.membership_id IS NULL OR member.incarnation IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(member.membership_id,member.incarnation,
   selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id)
  OR history.state IS DISTINCT FROM 'ACTIVE' OR history.to_time IS NOT NULL
  OR history.from_time IS DISTINCT FROM member.created_at OR NOT isfinite(history.from_time) OR history.from_time>clock_timestamp() THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 CASE history.provenance_kind
 WHEN 'COMPANY_ENROLLMENT_V1' THEN
  IF history.revision<>1 OR history.legacy_actor_user_id IS NOT NULL OR history.force_actor_user_id IS NOT NULL
   OR (history.native_account_id,history.command_id,history.command_receipt) IS DISTINCT FROM
    (selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id)
   OR num_nonnulls(history.native_account_id,history.command_id,history.command_receipt)<>3 THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 WHEN 'LEGACY_BACKFILL' THEN
  IF history.revision<>1 OR num_nonnulls(selected_company.origin_account_id,selected_company.origin_command_id,
    selected_company.origin_receipt_id,history.native_account_id,history.legacy_actor_user_id,
    history.force_actor_user_id,history.command_id,history.command_receipt)<>0 THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 WHEN 'LEGACY_TOPOLOGY_V1' THEN
  IF num_nonnulls(selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id,
    history.native_account_id,history.force_actor_user_id)<>0 THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  SELECT r.* INTO STRICT receipt FROM public.platform_legacy_topology_receipts r
   WHERE (r.actor_user_id,r.command_id,r.receipt_id)=
    (history.legacy_actor_user_id,history.command_id,history.command_receipt);
  IF jsonb_typeof(receipt.heads_after) IS DISTINCT FROM 'array' THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  IF jsonb_array_length(receipt.heads_after) NOT BETWEEN 1 AND 2 THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(receipt.input_bytes);
  SELECT h.* INTO STRICT head FROM public.group_authority_heads h WHERE h.group_id=p_group;
  IF receipt.outcome IS DISTINCT FROM 'APPLIED' OR receipt.result_code IS DISTINCT FROM 'applied'
   OR receipt.kind NOT IN(1,4,5) OR receipt.codec_version<>1
   OR receipt.input_digest IS DISTINCT FROM sha256(receipt.input_bytes)
   OR (decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest) IS DISTINCT FROM
    (receipt.actor_user_id,receipt.command_id,receipt.kind,receipt.input_digest)
   OR receipt.result_org_id IS DISTINCT FROM p_company OR receipt.result_group_id IS DISTINCT FROM p_group
   OR receipt.occurred_at IS DISTINCT FROM history.from_time OR history.revision<>1
   OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(receipt.heads_after) entry
    WHERE (entry->>'group_id')::uuid=p_group AND (entry->>'incarnation')::uuid=head.incarnation
     AND (entry->>'revision')::bigint BETWEEN 1 AND head.revision) THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 ELSE RAISE EXCEPTION 'company_information.material_unavailable';
 END CASE;
 RETURN QUERY SELECT to_jsonb(selected_company),to_jsonb(member),to_jsonb(history);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: ops/native-company-information/root-material-v1.sql
-- UNINSTALLED finite selected native birth/root validator, independent of caller.
-- Existing Group/Account/Company guards precede this function. No Account/family
-- impersonation, Group sibling scan or singleton/epoch1 current assumption.
CREATE FUNCTION public.identity_company_information_root_material_v1(p_company uuid,p_group uuid)
RETURNS TABLE(root_account_id uuid,company_epoch bigint,current_policy_receipt_id uuid,
 assignment_id uuid,role_id uuid,installed_object_type_id uuid,registered_clauses jsonb,root_material jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); organization record; selected_group record;
 birth public.company_enrollment_receipts; request_row public.company_enrollment_requests;
 effect public.company_enrollment_effect_bindings; actor_row public.company_actors;
 head public.company_authority_heads; assignment public.user_role_assignments; role public.policy_roles;
 ar public.policy_assignment_revisions; rr public.policy_role_revisions; workspace public.native_company_object_refs;
 installation public.native_company_catalog_installs; decoded record; event record; provenance jsonb; trace text;
 clauses jsonb:='[]'::jsonb; clause jsonb; fields jsonb; role_hash bytea; clause_row record;
 actions jsonb; properties jsonb; catalog jsonb; events jsonb; initial_sources jsonb; source_row jsonb;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_company IS NULL OR p_group IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.id,o.group_id,o.created_at,o.origin_account_id,o.origin_command_id,o.origin_receipt_id
  INTO STRICT organization FROM public.organizations o WHERE o.id=p_company;
 SELECT g.id,g.created_at,g.origin_account_id,g.origin_command_id,g.origin_receipt_id
  INTO STRICT selected_group FROM public.groups g WHERE g.id=p_group;
 IF organization.group_id IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 SELECT h.* INTO STRICT head FROM public.company_authority_heads h WHERE h.org_id=p_company FOR SHARE OF h;
 PERFORM public.native_company_policy_assert_current_head_v1(p_company);
 SELECT r.* INTO STRICT birth FROM public.company_enrollment_receipts r
  WHERE (r.org_id,r.account_id,r.command_id,r.receipt_id)=
   (head.org_id,head.origin_account_id,head.origin_command_id,head.origin_receipt_id);
 SELECT q.* INTO STRICT request_row FROM public.company_enrollment_requests q
  WHERE (q.account_id,q.command_id)=(birth.account_id,birth.command_id);
 SELECT e.* INTO STRICT effect FROM public.company_enrollment_effect_bindings e
  WHERE (e.account_id,e.command_id,e.receipt_id)=(birth.account_id,birth.command_id,birth.receipt_id);
 IF (head.origin_account_id,head.origin_command_id,head.origin_receipt_id) IS DISTINCT FROM
    (organization.origin_account_id,organization.origin_command_id,organization.origin_receipt_id)
  OR birth.group_id IS DISTINCT FROM p_group OR birth.root_revision<>1 OR birth.codec_version<>1
  OR birth.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
  OR birth.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex')
  OR request_row.state IS DISTINCT FROM 'COMMITTED' OR request_row.committed_receipt_id IS DISTINCT FROM birth.receipt_id
  OR request_row.terminal_at IS DISTINCT FROM birth.committed_at
  OR (request_row.codec_version,request_row.input_digest,request_row.designation_receipt_id) IS DISTINCT FROM
   (birth.codec_version,birth.input_digest,birth.designation_receipt_id)
  OR (effect.org_id,effect.group_id,effect.administrative_account_id,effect.codec_version,effect.designation_receipt_id,
      effect.input_digest,effect.started_at,effect.catalog_version,effect.manifest_digest,effect.session_id) IS DISTINCT FROM
   (birth.org_id,birth.group_id,birth.administrative_account_id,birth.codec_version,birth.designation_receipt_id,
      birth.input_digest,birth.committed_at,birth.catalog_version,birth.manifest_digest,birth.session_id)
  OR organization.created_at IS DISTINCT FROM birth.committed_at OR NOT isfinite(birth.committed_at)
  OR (selected_group.origin_account_id,selected_group.origin_command_id,selected_group.origin_receipt_id) IS DISTINCT FROM
   (birth.account_id,birth.command_id,birth.receipt_id) OR selected_group.created_at IS DISTINCT FROM birth.committed_at THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 -- Payload retention may lawfully remove bytes; never reinterpret/reseal its digest.
 IF request_row.input_bytes IS NOT NULL THEN
  SELECT * INTO STRICT decoded FROM public.company_enrollment_decode_input_v1(request_row.input_bytes);
  IF (decoded.codec_version,decoded.account_id,decoded.command_id,decoded.input_digest,decoded.administrative_account_id)
    IS DISTINCT FROM (birth.codec_version,birth.account_id,birth.command_id,birth.input_digest,birth.administrative_account_id)
   OR decoded.group_id IS NOT NULL OR sha256(request_row.input_bytes) IS DISTINCT FROM birth.input_digest THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 END IF;
 SELECT jsonb_agg(to_jsonb(e) ORDER BY e.event_revision) INTO events
  FROM public.company_enrollment_request_events e WHERE (e.account_id,e.command_id)=(birth.account_id,birth.command_id);
 IF events IS NULL OR jsonb_array_length(events)<>2 THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
 -- Preparation is immutable provenance; its session may differ or be expired now.
 SELECT e.* INTO STRICT event FROM public.company_enrollment_request_events e
  WHERE (e.account_id,e.command_id,e.event_revision)=(birth.account_id,birth.command_id,1);
 IF event.from_state IS NOT NULL OR event.to_state IS DISTINCT FROM 'PENDING'
  OR event.reason_code IS DISTINCT FROM 'PREPARED' OR event.occurred_at IS DISTINCT FROM request_row.created_at
  OR event.actor_account_id IS DISTINCT FROM request_row.account_id OR event.session_id IS NULL
  OR event.session_id='00000000-0000-0000-0000-000000000000'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT e.* INTO STRICT event FROM public.company_enrollment_request_events e
  WHERE (e.account_id,e.command_id,e.event_revision)=(birth.account_id,birth.command_id,2);
 IF event.from_state IS DISTINCT FROM 'PENDING' OR event.to_state IS DISTINCT FROM 'COMMITTED'
  OR event.reason_code IS DISTINCT FROM 'COMMITTED' OR event.occurred_at IS DISTINCT FROM birth.committed_at
  OR event.actor_account_id IS DISTINCT FROM birth.account_id OR event.session_id IS DISTINCT FROM birth.session_id THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT a.* INTO STRICT actor_row FROM public.company_actors a
  WHERE (a.org_id,a.account_id)=(p_company,birth.administrative_account_id);
 IF actor_row.admission_receipt_id IS DISTINCT FROM birth.receipt_id OR actor_row.created_at IS DISTINCT FROM birth.committed_at
  OR actor_row.entitlement_ref IS DISTINCT FROM jsonb_build_object('kind','COMPANY_ENROLLMENT_V1',
   'account_id',birth.account_id::text,'command_id',birth.command_id::text,'org_id',p_company::text,'receipt_id',birth.receipt_id::text) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT a.* INTO STRICT assignment FROM public.user_role_assignments a
  WHERE (a.org_id,a.id)=(p_company,birth.root_assignment_id) FOR SHARE OF a;
 SELECT r.* INTO STRICT role FROM public.policy_roles r WHERE (r.org_id,r.id)=(p_company,assignment.role_id) FOR SHARE OF r;
 SELECT a.* INTO STRICT ar FROM public.policy_assignment_revisions a
  WHERE (a.org_id,a.assignment_id,a.revision)=(p_company,assignment.id,1);
 SELECT r.* INTO STRICT rr FROM public.policy_role_revisions r WHERE (r.org_id,r.role_id,r.revision)=(p_company,role.id,1);
 IF assignment.subject_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR assignment.account_id IS DISTINCT FROM birth.administrative_account_id
  OR assignment.native_current_revision<>1 OR assignment.policy_receipt_id IS NOT NULL
  OR role.subject_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR role.native_current_revision<>1 OR role.policy_receipt_id IS NOT NULL
  OR role.role_key IS DISTINCT FROM 'native_company_administration' OR role.display_name IS DISTINCT FROM '회사 초기 관리자'
  OR role.description IS NOT NULL OR role.is_system IS DISTINCT FROM true OR role.status NOT IN('DRAFT','ACTIVE','RETIRED')
  OR ar.subject_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR rr.subject_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
  OR (ar.account_id,ar.role_id,ar.role_revision) IS DISTINCT FROM (birth.administrative_account_id,role.id,1::bigint)
  OR ar.ceiling_digest IS DISTINCT FROM rr.clause_digest OR ar.policy_receipt_id IS NOT NULL OR rr.policy_receipt_id IS NOT NULL
  OR ar.state NOT IN('ACTIVE','REVOKED') OR rr.state NOT IN('ACTIVE','RETIRED')
  OR ar.valid_from IS DISTINCT FROM birth.committed_at OR rr.valid_from IS DISTINCT FROM birth.committed_at
  OR ar.valid_until IS NOT NULL OR rr.valid_until IS NOT NULL
  OR rr.catalog_version IS DISTINCT FROM birth.catalog_version OR rr.manifest_digest IS DISTINCT FROM birth.manifest_digest THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 -- Validate original attribution even when a later Company head/catalog exists.
 FOR provenance IN SELECT to_jsonb(assignment) UNION ALL SELECT to_jsonb(role)
  UNION ALL SELECT to_jsonb(ar) UNION ALL SELECT to_jsonb(rr) LOOP
  IF (provenance->>'origin_account_id')::uuid IS DISTINCT FROM birth.account_id
   OR (provenance->>'origin_command_id')::uuid IS DISTINCT FROM birth.command_id
   OR (provenance->>'origin_receipt_id')::uuid IS DISTINCT FROM birth.receipt_id THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  FOREACH trace IN ARRAY ARRAY['created_by','updated_by','assigned_by','user_id'] LOOP
   IF provenance?trace AND provenance->trace IS DISTINCT FROM 'null'::jsonb THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  END LOOP;
  FOREACH trace IN ARRAY ARRAY['created_by_account_id','updated_by_account_id','assigned_by_account_id','actor_account_id'] LOOP
   IF provenance?trace AND (provenance->>trace)::uuid IS DISTINCT FROM birth.account_id THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  END LOOP;
  FOREACH trace IN ARRAY ARRAY['created_at','updated_at'] LOOP
   IF provenance?trace AND (provenance->>trace)::timestamptz IS DISTINCT FROM birth.committed_at THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  END LOOP;
  IF provenance?'session_id' AND (provenance->>'session_id')::uuid IS DISTINCT FROM birth.session_id THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
 END LOOP;
 -- Catalog validation is caller-independent: it must run before O/B/inactive denial.
 PERFORM ontology_api.lock_native_company_catalog_current_v2(p_company);
 SELECT n.* INTO STRICT installation FROM public.native_company_catalog_installs n
  WHERE (n.org_id,n.catalog_version)=(p_company,birth.catalog_version);
 IF (installation.origin_account_id,installation.origin_command_id,installation.origin_receipt_id,installation.installed_at)
   IS DISTINCT FROM (birth.account_id,birth.command_id,birth.receipt_id,birth.committed_at)
  OR installation.manifest_digest IS DISTINCT FROM birth.manifest_digest OR installation.policy_receipt_id IS NOT NULL THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 -- Actual initial source attribution is still immutable after later optional installs.
 SELECT jsonb_agg(src.entry ORDER BY src.entry->>'kind' COLLATE "C",src.entry->>'id' COLLATE "C") INTO initial_sources FROM (
  SELECT jsonb_build_object('kind','installation','id',n.catalog_version,'row',to_jsonb(n)) entry
   FROM public.ont_builtin_catalog_installs n WHERE (n.org_id,n.catalog_version)=(p_company,birth.catalog_version)
  UNION ALL SELECT jsonb_build_object('kind','object','id',o.id::text,'row',jsonb_build_object(
   'attribution_protocol',o.attribution_protocol,'created_by',o.created_by,'created_by_account_id',o.created_by_account_id,
   'created_at',o.created_at,'updated_at',o.updated_at,'origin_account_id',o.origin_account_id,
   'origin_command_id',o.origin_command_id,'origin_receipt_id',o.origin_receipt_id,'policy_receipt_id',o.policy_receipt_id))
   FROM public.ont_object_types o JOIN public.native_company_object_refs r ON r.org_id=o.org_id AND r.object_type_id=o.id
    WHERE (r.org_id,r.catalog_version)=(p_company,birth.catalog_version)) src;
 IF initial_sources IS NULL OR jsonb_array_length(initial_sources)<>3 THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
 FOR source_row IN SELECT entry->'row' FROM jsonb_array_elements(initial_sources) entry LOOP
  IF source_row->>'attribution_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT'
   OR (source_row->>'origin_account_id')::uuid IS DISTINCT FROM birth.account_id
   OR (source_row->>'origin_command_id')::uuid IS DISTINCT FROM birth.command_id
   OR (source_row->>'origin_receipt_id')::uuid IS DISTINCT FROM birth.receipt_id
   OR (source_row?'policy_receipt_id' AND source_row->'policy_receipt_id' IS DISTINCT FROM 'null'::jsonb) THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  FOREACH trace IN ARRAY ARRAY['created_by','installed_by'] LOOP
   IF source_row?trace AND source_row->trace IS DISTINCT FROM 'null'::jsonb THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  END LOOP;
  FOREACH trace IN ARRAY ARRAY['created_by_account_id','installed_by_account_id'] LOOP
   IF source_row?trace AND (source_row->>trace)::uuid IS DISTINCT FROM birth.account_id THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  END LOOP;
  FOREACH trace IN ARRAY ARRAY['created_at','updated_at','installed_at'] LOOP
   IF source_row?trace AND (source_row->>trace)::timestamptz IS DISTINCT FROM birth.committed_at THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  END LOOP;
 END LOOP;
 IF EXISTS(SELECT 1 FROM public.ont_property_defs p JOIN public.native_company_property_refs r
   ON r.org_id=p.org_id AND r.object_type_id=p.object_type_id AND r.property_id=p.id
   WHERE (r.org_id,r.catalog_version)=(p_company,birth.catalog_version) AND p.created_at IS DISTINCT FROM birth.committed_at)
  OR EXISTS(SELECT 1 FROM public.ont_action_types a JOIN public.native_company_action_refs r
   ON r.org_id=a.org_id AND r.object_type_id=a.object_type_id AND r.action_type_id=a.id
   WHERE (r.org_id,r.catalog_version)=(p_company,birth.catalog_version) AND a.created_at IS DISTINCT FROM birth.committed_at)
  OR EXISTS(SELECT 1 FROM public.ont_object_type_key_revisions k JOIN public.native_company_object_refs r
   ON r.org_id=k.org_id AND r.object_key=k.stable_key
   WHERE (r.org_id,r.catalog_version)=(p_company,birth.catalog_version)
    AND (k.created_at IS DISTINCT FROM birth.committed_at OR k.updated_at IS DISTINCT FROM birth.committed_at)) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT w.* INTO STRICT workspace FROM public.native_company_object_refs w
  WHERE (w.org_id,w.catalog_version,w.object_key)=(p_company,birth.catalog_version,'company_workspace');
 SELECT jsonb_agg(jsonb_build_object('org_id',a.org_id::text,'object_type_id',a.object_type_id::text,
   'action_type_id',a.action_type_id::text,'registration_revision',a.registration_revision::text,'manifest_digest',encode(a.manifest_digest,'hex'))
   ORDER BY a.org_id,a.object_type_id,a.action_type_id,a.registration_revision,a.manifest_digest) INTO actions
  FROM public.native_company_action_refs a WHERE (a.org_id,a.catalog_version)=(p_company,birth.catalog_version);
 SELECT jsonb_agg(jsonb_build_object('org_id',p.org_id::text,'object_type_id',p.object_type_id::text,
   'property_id',p.property_id::text,'schema_revision',p.schema_revision::text)
   ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision) INTO properties
  FROM public.native_company_property_refs p WHERE (p.org_id,p.catalog_version)=(p_company,birth.catalog_version);
 IF actions IS DISTINCT FROM birth.action_refs OR properties IS DISTINCT FROM birth.property_refs
  OR workspace.schema_revision<>1 OR workspace.manifest_digest IS DISTINCT FROM birth.manifest_digest THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 -- Reconstruct from authoritative field membership, not the cached digest.
 clauses:='[]'::jsonb;
 IF (SELECT count(*) FROM public.policy_capability_clauses c WHERE c.org_id=p_company AND c.role_id=role.id AND c.role_revision=1)<>7
  OR (SELECT count(*) FROM public.policy_capability_clause_fields f WHERE f.org_id=p_company AND f.role_id=role.id AND f.role_revision=1)<>16 THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 -- Check all omitted field digests against their exact clause and catalog tuples.
 -- The validated sixteen-row shape is fixed; existing FKs alone do not read data.
 IF EXISTS(SELECT 1 FROM public.policy_capability_clause_fields f
  LEFT JOIN public.policy_capability_clauses c
   ON (c.org_id,c.role_id,c.role_revision,c.clause_index,c.action_object_type_id,c.manifest_digest)=
    (f.org_id,f.role_id,f.role_revision,f.clause_index,f.object_type_id,f.manifest_digest)
  LEFT JOIN public.native_company_property_refs p
   ON (p.org_id,p.object_type_id,p.property_id,p.schema_revision,p.manifest_digest,p.content_digest)=
    (f.org_id,f.object_type_id,f.property_id,f.schema_revision,f.manifest_digest,f.content_digest)
  WHERE f.org_id=p_company AND f.role_id=role.id AND f.role_revision=1
   AND (c.role_id IS NULL OR p.property_id IS NULL)) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 FOR clause_row IN SELECT c.*,a.action_key FROM public.policy_capability_clauses c
  JOIN public.native_company_action_refs a ON a.org_id=c.org_id AND a.object_type_id=c.action_object_type_id
   AND a.action_type_id=c.action_type_id AND a.registration_revision=c.registration_revision AND a.manifest_digest=c.manifest_digest
  WHERE c.org_id=p_company AND c.role_id=role.id AND c.role_revision=1 ORDER BY c.clause_index
 LOOP
  IF clause_row.clause_index IS DISTINCT FROM jsonb_array_length(clauses)+1
   OR clause_row.action_key IS DISTINCT FROM (ARRAY['context.discover','company.identity.read','company.policy.read',
    'company.policy.assign','company.policy.revoke','context.discover','company.identity.read'])[clause_row.clause_index]
   OR clause_row.effect IS DISTINCT FROM 'ALLOW' OR clause_row.resource_org_id IS DISTINCT FROM p_company
   OR clause_row.delegable IS DISTINCT FROM (clause_row.clause_index>5)
   OR clause_row.valid_from IS DISTINCT FROM birth.committed_at OR clause_row.valid_until IS NOT NULL THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  SELECT coalesce(jsonb_agg(jsonb_build_object('org_id',f.org_id::text,'object_type_id',f.object_type_id::text,
    'property_id',f.property_id::text,'schema_revision',f.schema_revision::text)
    ORDER BY f.org_id,f.object_type_id,f.property_id,f.schema_revision),'[]'::jsonb) INTO fields
   FROM public.policy_capability_clause_fields f
   WHERE f.org_id=p_company AND f.role_id=role.id AND f.role_revision=1 AND f.clause_index=clause_row.clause_index;
  IF jsonb_array_length(fields) IS DISTINCT FROM (ARRAY[2,2,8,0,0,2,2])[clause_row.clause_index] THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  clause:=jsonb_build_object('kind','COMPANY_CAPABILITY_CLAUSE_V1',
   'action',jsonb_build_object('org_id',p_company::text,'object_type_id',clause_row.action_object_type_id::text,
    'action_type_id',clause_row.action_type_id::text,'registration_revision',clause_row.registration_revision::text,
    'manifest_digest',encode(clause_row.manifest_digest,'hex')),
   'resource',jsonb_build_object('kind','COMPANY','org_id',p_company::text),'fields',fields,
   'valid_from',to_char(clause_row.valid_from AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
   'valid_until',NULL,'delegable',clause_row.delegable);
  IF clause_row.clause_digest IS DISTINCT FROM sha256(convert_to(clause::text,'UTF8')) THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  clauses:=clauses||jsonb_build_array(clause);
 END LOOP;
 IF jsonb_array_length(clauses)<>7 THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
 SELECT sha256(convert_to(jsonb_agg(jsonb_build_object('clause_index',n,
   'clause_digest',encode(sha256(convert_to(c::text,'UTF8')),'hex')) ORDER BY n)::text,'UTF8'))
  INTO role_hash FROM jsonb_array_elements(clauses) WITH ORDINALITY AS x(c,n);

 IF role_hash IS DISTINCT FROM rr.clause_digest THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_build_object('initial_source_attribution',initial_sources,
  'installs',(SELECT jsonb_agg(to_jsonb(c) ORDER BY c.catalog_version COLLATE "C") FROM public.native_company_catalog_installs c WHERE c.org_id=p_company),
  'objects',(SELECT jsonb_agg(to_jsonb(o) ORDER BY o.object_key COLLATE "C") FROM public.native_company_object_refs o WHERE o.org_id=p_company),
  'actions',(SELECT jsonb_agg(to_jsonb(a) ORDER BY a.action_key COLLATE "C") FROM public.native_company_action_refs a WHERE a.org_id=p_company),
  'properties',(SELECT jsonb_agg(to_jsonb(p) ORDER BY p.property_key COLLATE "C") FROM public.native_company_property_refs p WHERE p.org_id=p_company)) INTO catalog;
 RETURN QUERY SELECT birth.administrative_account_id,head.epoch,head.current_policy_receipt_id,assignment.id,role.id,
  workspace.object_type_id,clauses,jsonb_build_object('company_head',to_jsonb(head),'birth_receipt',to_jsonb(birth),
   'birth_request',to_jsonb(request_row)-'input_bytes','birth_effect',to_jsonb(effect),'birth_events',events,
   'company_actor',to_jsonb(actor_row),'assignment',to_jsonb(assignment),'role',to_jsonb(role),
   'assignment_revision',to_jsonb(ar),'role_revision',to_jsonb(rr),'registered_clauses',clauses,
   'workspace',to_jsonb(workspace),'catalog',catalog);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: ops/native-company-information/manager-current-v1.sql
-- UNINSTALLED initial manager Current-Grant material. No command is accepted.
-- The adapter must first verify only the signed namespace, then call this owner
-- before acquiring Account/family. Final rereads pass the retained Group ID;
-- NULL is permitted only on the first source call in a fresh retained scope.
CREATE FUNCTION public.identity_company_information_manager_current_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_expected_group uuid)
RETURNS TABLE(actor_account_id uuid,session_id uuid,org_id uuid,command_id uuid,
 current_group_id uuid,company_epoch bigint,current_policy_receipt_id uuid,
 context_generation bigint,assignment_id uuid,assignment_revision bigint,
 role_id uuid,role_revision bigint,registered_clauses jsonb,company_name text,
 company_slug text,installed_object_type_id uuid,observed_at timestamptz,
 source_xid xid8,source_backend_pid integer,source_material jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid;
 actual_group uuid; planned_present boolean; actual_present boolean;
 control record; family record; retained_group record; presence record;
 selected record; native_root record; observed timestamptz; material jsonb; account_row jsonb;
 registration jsonb; consent jsonb;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR num_nonnulls(p_account,p_family,p_company,p_command)<>4
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_account,p_family,p_company,p_command,p_expected_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 -- The only initial planning read. No labels or source leave this scope.
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 planned_present:=FOUND;
 IF p_expected_group IS NOT NULL AND planned_group IS DISTINCT FROM p_expected_group THEN
  -- In particular, never discover/lock a second Group during finalization.
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 IF planned_group IS NOT NULL THEN
  SELECT * INTO STRICT retained_group
   FROM public.identity_company_information_group_lock_v1(p_company,planned_group);
 END IF;
 -- Actual Group head+row precede the original Account and family rows.
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation
  OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' OR family.auth_time IS NULL
  OR family.created_at IS NULL OR NOT isfinite(family.auth_time) OR NOT isfinite(family.created_at)
  OR family.auth_time>family.created_at OR family.created_at>clock_timestamp() THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 IF control.security_generation IS NULL OR control.security_generation<1
  OR control.revision IS NULL OR control.revision<1
  OR control.context_generation IS NULL OR control.context_generation NOT BETWEEN 1 AND 257 THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_agg(to_jsonb(r) ORDER BY r.terms_kind COLLATE "C") INTO registration
  FROM public.native_company_policy_registration_custody_v1(p_account) r;
 IF registration IS NULL OR jsonb_array_length(registration) NOT BETWEEN 1 AND 8 THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_agg(to_jsonb(c) ORDER BY c.terms_kind COLLATE "C") INTO consent
  FROM public.account_login_consent_v1(p_account) c;
 IF consent IS DISTINCT FROM registration THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
 SELECT * INTO STRICT presence FROM public.account_context_presence_v1(p_account);
 IF presence.context_generation IS DISTINCT FROM control.context_generation THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT o.group_id INTO actual_group FROM public.organizations o WHERE o.id=p_company;
 actual_present:=FOUND;
 IF actual_present IS DISTINCT FROM planned_present OR actual_group IS DISTINCT FROM planned_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 IF planned_group IS NULL THEN
  -- A present row with no Group violates the finalized Company substrate.
  IF planned_present THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  -- Authenticated healthy absent Company.
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 -- Only requested Company/membership/history. No whole-Group provenance helper.
 SELECT * INTO STRICT selected FROM public.identity_company_information_selected_lock_v1(p_company,planned_group);
 IF num_nonnulls((selected.company_row->>'origin_account_id')::uuid,
    (selected.company_row->>'origin_command_id')::uuid,(selected.company_row->>'origin_receipt_id')::uuid)=0 THEN
  -- This is finite selected nonnative source validation, not Group-wide health.
  IF EXISTS(SELECT 1 FROM public.company_enrollment_receipts r WHERE r.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.company_enrollment_effect_bindings e WHERE e.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.native_company_catalog_installs c WHERE c.org_id=p_company) THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 -- This validates actual birth/root/catalog without authenticating as the manager.
 -- Every native corruption check precedes every healthy inactive/nonmanager return.
 SELECT * INTO STRICT native_root FROM public.identity_company_information_root_material_v1(p_company,planned_group);
 IF (selected.membership_revision_row->>'native_account_id',selected.membership_revision_row->>'command_id',
     selected.membership_revision_row->>'command_receipt') IS DISTINCT FROM
    (native_root.root_material->'birth_receipt'->>'account_id',native_root.root_material->'birth_receipt'->>'command_id',
     native_root.root_material->'birth_receipt'->>'receipt_id')
  OR (selected.membership_revision_row->>'from_time')::timestamptz IS DISTINCT FROM
     (native_root.root_material->'birth_receipt'->>'committed_at')::timestamptz THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 observed:=clock_timestamp();
 IF (selected.membership_revision_row->>'from_time')::timestamptz>observed THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 IF retained_group.group_head_row->>'state' IS DISTINCT FROM 'ACTIVE'
  OR retained_group.group_row->>'status' IS DISTINCT FROM 'ACTIVE'
  OR selected.company_row->>'status' IS DISTINCT FROM 'ACTIVE'
  OR native_root.root_material->'role'->>'status' IS DISTINCT FROM 'ACTIVE'
  OR native_root.root_material->'assignment_revision'->>'state' IS DISTINCT FROM 'ACTIVE'
  OR native_root.root_material->'role_revision'->>'state' IS DISTINCT FROM 'ACTIVE'
  OR (native_root.root_material->'assignment_revision'->>'valid_from')::timestamptz>observed
  OR (native_root.root_material->'role_revision'->>'valid_from')::timestamptz>observed
  OR native_root.root_account_id IS DISTINCT FROM p_account THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF EXISTS(SELECT 1 FROM public.native_company_policy_inputs_v1 i
   WHERE i.org_id=p_company AND i.actor_account_id=p_account AND i.command_id=p_command)
  OR EXISTS(SELECT 1 FROM public.native_company_policy_receipts_v1 r
   WHERE r.org_id=p_company AND r.actor_account_id=p_account AND r.command_id=p_command) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_build_object('id',a.id,'created_at',a.created_at,'security',to_jsonb(s)) INTO STRICT account_row
  FROM public.accounts a JOIN public.account_security s ON s.account_id=a.id WHERE a.id=p_account;
 material:=native_root.root_material||jsonb_build_object('kind','COMPANY_INFORMATION_MANAGER_CURRENT_SOURCE_V1',
  'request',jsonb_build_object('codec_version',4,'operation','Grant','account_id',p_account,
    'session_id',p_family,'org_id',p_company,'command_id',p_command),
  'account',account_row,'family',jsonb_build_object('id',p_family,'material',to_jsonb(family)),
  'registration',registration,'consent',consent,'context_presence',to_jsonb(presence),
  'group',retained_group.group_row,'group_head',retained_group.group_head_row,'company',selected.company_row,
  'membership',selected.membership_row,'membership_revision',selected.membership_revision_row);
 observed:=clock_timestamp();
 RETURN QUERY SELECT p_account,p_family,p_company,p_command,planned_group,native_root.company_epoch,
  native_root.current_policy_receipt_id,control.context_generation,native_root.assignment_id,
  1::bigint,native_root.role_id,1::bigint,native_root.registered_clauses,
  selected.company_row->>'name',selected.company_row->>'slug',native_root.installed_object_type_id,
  observed,pg_current_xact_id(),pg_backend_pid(),material;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: ops/native-company-information/acl-v1.sql
-- UNINSTALLED exact routine ACL source; no role/table/schema privilege change.
ALTER FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid) OWNER TO console_app;
ALTER FUNCTION public.identity_company_information_selected_lock_v1(uuid,uuid) OWNER TO console_app;
ALTER FUNCTION public.identity_company_information_root_material_v1(uuid,uuid) OWNER TO console_account_owner;
ALTER FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid) OWNER TO console_account_owner;
DO $acl$
DECLARE routine regprocedure; grantee_name text;
BEGIN
 FOREACH routine IN ARRAY ARRAY[
  'public.identity_company_information_group_lock_v1(uuid,uuid)'::regprocedure,
  'public.identity_company_information_selected_lock_v1(uuid,uuid)'::regprocedure,
  'public.identity_company_information_root_material_v1(uuid,uuid)'::regprocedure,
  'public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)'::regprocedure] LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',routine);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p
   CROSS JOIN LATERAL aclexplode(coalesce(p.proacl,acldefault('f',p.proowner))) a
   JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid=routine LOOP
   EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',routine,grantee_name);
  END LOOP;
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_information_group_lock_v1(uuid,uuid),
 public.identity_company_information_selected_lock_v1(uuid,uuid),
 public.identity_company_information_root_material_v1(uuid,uuid) TO console_account_owner;
GRANT EXECUTE ON FUNCTION public.identity_company_information_manager_current_v1(uuid,uuid,uuid,uuid,uuid)
 TO console_account_owner,console_rt;
