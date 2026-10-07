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
