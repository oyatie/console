-- Versioned current readers; the birth-only v1 functions remain byte-identical.
-- Payroll appends the two actual assignment interval columns from the reviewed
-- eleven-column successor; permanent role/clause dates never replace them.
CREATE FUNCTION ontology_api.lock_native_company_catalog_current_v2(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); found_count integer; installation record; extra integer;
BEGIN
 PERFORM set_config('app.current_org',p_org::text,true);
 SELECT n.* INTO STRICT installation FROM public.native_company_catalog_installs n WHERE n.org_id=p_org AND n.catalog_version='native-company-identity-2026-09-19.1';
 IF installation.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
  OR installation.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex') THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 SELECT count(*)::integer INTO extra FROM public.native_company_catalog_installs n WHERE n.org_id=p_org
  AND n.catalog_version='native-payroll-collection-read-v1';
 IF extra NOT IN (0,1) OR (SELECT count(*) FROM public.native_company_catalog_installs n WHERE n.org_id=p_org)<>1+extra THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM k.org_id FROM public.ont_object_type_key_revisions k
  JOIN public.native_company_object_refs r ON r.org_id=k.org_id AND r.object_key=k.stable_key
  WHERE k.org_id=p_org ORDER BY k.org_id,k.stable_key COLLATE "C" FOR SHARE OF k;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>2+extra THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM o.id FROM public.ont_object_types o JOIN public.native_company_object_refs r ON r.org_id=o.org_id AND r.object_type_id=o.id
  WHERE o.org_id=p_org ORDER BY o.org_id,o.id FOR SHARE OF o;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>2+extra THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM p.id FROM public.ont_property_defs p JOIN public.native_company_property_refs r ON r.org_id=p.org_id AND r.property_id=p.id
  WHERE p.org_id=p_org ORDER BY p.org_id,p.object_type_id,p.id FOR SHARE OF p;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>10+18*extra THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM a.id FROM public.ont_action_types a JOIN public.native_company_action_refs r ON r.org_id=a.org_id AND r.action_type_id=a.id
  WHERE a.org_id=p_org ORDER BY a.org_id,a.object_type_id,a.id FOR SHARE OF a;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>5+extra THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 IF (SELECT count(*) FROM public.native_company_object_refs r WHERE r.org_id=p_org)<>2+extra
  OR (SELECT count(*) FROM public.native_company_action_refs r WHERE r.org_id=p_org)<>5+extra
  OR (SELECT count(*) FROM public.native_company_property_refs r WHERE r.org_id=p_org)<>10+18*extra
  OR EXISTS(SELECT 1 FROM public.native_company_object_refs r
   LEFT JOIN public.ont_object_types o ON o.org_id=r.org_id AND o.id=r.object_type_id
   LEFT JOIN public.ont_object_type_key_revisions k ON k.org_id=o.org_id AND k.stable_key=o.stable_key
   WHERE r.org_id=p_org AND (o.id IS NULL OR k.revision IS DISTINCT FROM 1
    OR o.schema_version IS DISTINCT FROM r.schema_revision OR o.lifecycle_state IS DISTINCT FROM 'published'
    OR o.stable_key IS DISTINCT FROM r.object_key OR o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
    OR o.origin_account_id IS DISTINCT FROM installation.origin_account_id
    OR o.origin_command_id IS DISTINCT FROM installation.origin_command_id
    OR o.origin_receipt_id IS DISTINCT FROM installation.origin_receipt_id
    OR r.content_digest IS DISTINCT FROM sha256(convert_to(jsonb_build_object('stable_key',o.stable_key,'title',o.title,
     'title_property_key',o.title_property_key,'backing_kind',o.backing_kind,'backing_table',o.backing_table,
     'primary_key_property',o.primary_key_property,'schema_version',o.schema_version,'lifecycle_state',o.lifecycle_state)::text,'UTF8'))
    OR EXISTS(SELECT 1 FROM public.ont_object_types other WHERE other.org_id=o.org_id AND other.stable_key=o.stable_key
     AND other.id<>o.id AND other.lifecycle_state='published')))
  OR EXISTS(SELECT 1 FROM public.native_company_action_refs r
   LEFT JOIN public.ont_action_types a ON a.org_id=r.org_id AND a.object_type_id=r.object_type_id AND a.id=r.action_type_id
   WHERE r.org_id=p_org AND (a.id IS NULL OR a.dispatch_target IS DISTINCT FROM r.action_key
    OR r.content_digest IS DISTINCT FROM sha256(convert_to(jsonb_build_object('stable_key',a.stable_key,'title',a.title,
     'params_schema',a.params_schema,'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,
     'dispatch',a.dispatch,'dispatch_target',a.dispatch_target,'control_points',a.control_points)::text,'UTF8'))))
  OR EXISTS(SELECT 1 FROM public.native_company_property_refs r
   LEFT JOIN public.ont_property_defs p ON p.org_id=r.org_id AND p.object_type_id=r.object_type_id AND p.id=r.property_id
   WHERE r.org_id=p_org AND (p.id IS NULL
    OR r.content_digest IS DISTINCT FROM sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,
     'config',p.config,'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8')))) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 IF (SELECT count(*) FROM public.ont_object_types o WHERE o.org_id=p_org)<>2+extra
  OR (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=p_org)<>10+18*extra
  OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=p_org)<>5+extra
  OR EXISTS(SELECT 1 FROM public.ont_link_types l WHERE l.org_id=p_org)
  OR EXISTS(SELECT 1 FROM public.ont_analytics a WHERE a.org_id=p_org) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 IF extra=1 THEN PERFORM public.native_company_policy_assert_payroll_catalog_v1(p_org); END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.identity_company_projection_v2(p_account uuid,p_family uuid,p_company uuid)
RETURNS TABLE(company_epoch bigint,context_generation bigint,assignment_id uuid,assignment_revision bigint,
 role_id uuid,role_revision bigint,registered_clauses jsonb,company_name text,company_slug text,current_policy_receipt_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid; actual_group uuid;
 company uuid:=p_company; role_id_value uuid; birth_time timestamptz;
 control record; family record; organization record; assignment record; assignment_revision_row record;
 role record; role_revision_row record; clause_row record; head record; group_head record;
 clauses jsonb; clause jsonb; fields jsonb; role_hash bytea;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_account IS NULL OR p_family IS NULL OR p_company IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN (p_account,p_family,p_company)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 IF planned_group IS NOT NULL THEN
  SELECT * INTO STRICT group_head FROM public.group_authority_lock_shared_v1(planned_group);
 END IF;
 SELECT o.group_id INTO actual_group FROM public.organizations o WHERE o.id=p_company;
 IF actual_group IS DISTINCT FROM planned_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
 END IF;
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 PERFORM 1 FROM public.account_context_presence_v1(p_account);
 IF planned_group IS NULL THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT o.* INTO STRICT organization FROM public.organizations o WHERE o.id=p_company;
 IF organization.origin_account_id IS NULL THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF group_head.state<>'ACTIVE' OR organization.status<>'ACTIVE'
  OR NOT EXISTS(SELECT 1 FROM public.groups g WHERE g.id=planned_group AND g.status='ACTIVE') THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF NOT EXISTS(SELECT 1 FROM public.group_memberships m JOIN public.group_membership_revisions r
    ON r.group_id=m.group_id AND r.org_id=m.org_id AND r.membership_id=m.membership_id
     AND r.revision=m.current_revision AND r.incarnation=m.incarnation
   WHERE m.org_id=p_company AND m.group_id=planned_group AND r.state='ACTIVE' AND r.to_time IS NULL) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 SELECT h.* INTO STRICT head FROM public.company_authority_heads h WHERE h.org_id=p_company FOR SHARE;
 PERFORM public.native_company_policy_assert_current_head_v1(p_company);
 IF NOT EXISTS(SELECT 1 FROM public.user_role_assignments a WHERE a.org_id=p_company
   AND a.subject_protocol='NATIVE_ACCOUNT' AND a.account_id=p_account
   AND a.id=(SELECT e.root_assignment_id FROM public.company_enrollment_receipts e WHERE e.org_id=p_company)) THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT a.* INTO STRICT assignment FROM public.user_role_assignments a WHERE a.org_id=p_company
  AND a.subject_protocol='NATIVE_ACCOUNT' AND a.account_id=p_account
   AND a.id=(SELECT e.root_assignment_id FROM public.company_enrollment_receipts e WHERE e.org_id=p_company) FOR SHARE;
 SELECT r.* INTO STRICT role FROM public.policy_roles r WHERE r.org_id=p_company AND r.id=assignment.role_id FOR SHARE;
 SELECT r.* INTO STRICT assignment_revision_row FROM public.policy_assignment_revisions r
  WHERE r.org_id=p_company AND r.assignment_id=assignment.id AND r.revision=assignment.native_current_revision;
 SELECT r.* INTO STRICT role_revision_row FROM public.policy_role_revisions r
  WHERE r.org_id=p_company AND r.role_id=role.id AND r.revision=role.native_current_revision;
 IF assignment.native_current_revision<>1 OR role.native_current_revision<>1
  OR role.subject_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
  OR assignment_revision_row.role_id IS DISTINCT FROM role.id OR assignment_revision_row.role_revision<>1
  OR assignment_revision_row.account_id IS DISTINCT FROM p_account
  OR assignment_revision_row.ceiling_digest IS DISTINCT FROM role_revision_row.clause_digest
  OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_receipts r JOIN public.company_enrollment_requests q
    ON q.account_id=r.account_id AND q.command_id=r.command_id AND q.committed_receipt_id=r.receipt_id
    WHERE r.org_id=p_company AND r.administrative_account_id=p_account AND r.root_assignment_id=assignment.id
     AND r.root_revision=1 AND q.state='COMMITTED' AND r.account_id=assignment.origin_account_id
     AND r.command_id=assignment.origin_command_id AND r.receipt_id=assignment.origin_receipt_id)
  OR NOT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company AND a.account_id=p_account
    AND a.admission_receipt_id=assignment.origin_receipt_id) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 IF role.status<>'ACTIVE' OR role_revision_row.state<>'ACTIVE' OR assignment_revision_row.state<>'ACTIVE'
  OR role_revision_row.valid_from>clock_timestamp() OR assignment_revision_row.valid_from>clock_timestamp()
  OR (role_revision_row.valid_until IS NOT NULL AND role_revision_row.valid_until<=clock_timestamp())
  OR (assignment_revision_row.valid_until IS NOT NULL AND assignment_revision_row.valid_until<=clock_timestamp()) THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF role_revision_row.valid_until IS NOT NULL OR assignment_revision_row.valid_until IS NOT NULL
  OR role_revision_row.valid_from IS DISTINCT FROM assignment_revision_row.valid_from THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM ontology_api.lock_native_company_catalog_current_v2(p_company);
 role_id_value:=role.id; birth_time:=role_revision_row.valid_from;
 -- Reconstruct from authoritative field membership, not the cached digest.
 clauses:='[]'::jsonb;
 IF (SELECT count(*) FROM public.policy_capability_clauses c WHERE c.org_id=company AND c.role_id=role_id_value AND c.role_revision=1)<>7
  OR (SELECT count(*) FROM public.policy_capability_clause_fields f WHERE f.org_id=company AND f.role_id=role_id_value AND f.role_revision=1)<>16 THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 FOR clause_row IN SELECT c.*,a.action_key FROM public.policy_capability_clauses c
  JOIN public.native_company_action_refs a ON a.org_id=c.org_id AND a.object_type_id=c.action_object_type_id
   AND a.action_type_id=c.action_type_id AND a.registration_revision=c.registration_revision AND a.manifest_digest=c.manifest_digest
  WHERE c.org_id=company AND c.role_id=role_id_value AND c.role_revision=1 ORDER BY c.clause_index
 LOOP
  IF clause_row.clause_index IS DISTINCT FROM jsonb_array_length(clauses)+1
   OR clause_row.action_key IS DISTINCT FROM (ARRAY['context.discover','company.identity.read','company.policy.read',
    'company.policy.assign','company.policy.revoke','context.discover','company.identity.read'])[clause_row.clause_index]
   OR clause_row.effect IS DISTINCT FROM 'ALLOW' OR clause_row.resource_org_id IS DISTINCT FROM company
   OR clause_row.delegable IS DISTINCT FROM (clause_row.clause_index>5)
   OR clause_row.valid_from IS DISTINCT FROM birth_time OR clause_row.valid_until IS NOT NULL THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  SELECT coalesce(jsonb_agg(jsonb_build_object('org_id',f.org_id::text,'object_type_id',f.object_type_id::text,
    'property_id',f.property_id::text,'schema_revision',f.schema_revision::text)
    ORDER BY f.org_id,f.object_type_id,f.property_id,f.schema_revision),'[]'::jsonb) INTO fields
   FROM public.policy_capability_clause_fields f
   WHERE f.org_id=company AND f.role_id=role_id_value AND f.role_revision=1 AND f.clause_index=clause_row.clause_index;
  IF jsonb_array_length(fields) IS DISTINCT FROM (ARRAY[2,2,8,0,0,2,2])[clause_row.clause_index] THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  clause:=jsonb_build_object('kind','COMPANY_CAPABILITY_CLAUSE_V1',
   'action',jsonb_build_object('org_id',company::text,'object_type_id',clause_row.action_object_type_id::text,
    'action_type_id',clause_row.action_type_id::text,'registration_revision',clause_row.registration_revision::text,
    'manifest_digest',encode(clause_row.manifest_digest,'hex')),
   'resource',jsonb_build_object('kind','COMPANY','org_id',company::text),'fields',fields,
   'valid_from',to_char(clause_row.valid_from AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
   'valid_until',NULL,'delegable',clause_row.delegable);
  IF clause_row.clause_digest IS DISTINCT FROM sha256(convert_to(clause::text,'UTF8')) THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  clauses:=clauses||jsonb_build_array(clause);
 END LOOP;
 IF jsonb_array_length(clauses)<>7 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 SELECT sha256(convert_to(jsonb_agg(jsonb_build_object('clause_index',n,
   'clause_digest',encode(sha256(convert_to(c::text,'UTF8')),'hex')) ORDER BY n)::text,'UTF8'))
  INTO role_hash FROM jsonb_array_elements(clauses) WITH ORDINALITY AS x(c,n);

 IF role_hash IS DISTINCT FROM role_revision_row.clause_digest THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 RETURN QUERY SELECT head.epoch,control.context_generation,assignment.id,assignment.native_current_revision,
  role.id,role.native_current_revision,clauses,organization.name,organization.slug,head.current_policy_receipt_id;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.identity_company_payroll_projection_v1(p_account uuid,p_family uuid,p_company uuid)
RETURNS TABLE(company_epoch bigint,current_policy_receipt_id uuid,assignment_id uuid,assignment_revision bigint,
 role_id uuid,role_revision bigint,registered_clauses jsonb,company_name text,company_slug text,
 assignment_valid_from timestamptz,assignment_valid_until timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); identity_row record;
 role_row public.policy_roles; a public.user_role_assignments; ar public.policy_assignment_revisions;
 r public.native_company_policy_receipts_v1; clauses jsonb; at_time timestamptz;
BEGIN
 SELECT * INTO identity_row FROM public.identity_company_projection_v2(p_account,p_family,p_company);
 IF NOT FOUND THEN RETURN; END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT x.* INTO role_row FROM public.policy_roles x WHERE x.org_id=p_company AND x.role_key='native_payroll_collection_read' FOR SHARE;
 IF NOT FOUND THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT x.* INTO STRICT a FROM public.user_role_assignments x WHERE x.org_id=p_company AND x.role_id=role_row.id
  AND x.account_id=p_account FOR SHARE;
 SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x WHERE x.org_id=p_company
  AND x.assignment_id=a.id AND x.revision=a.native_current_revision;
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_company AND x.receipt_id=ar.policy_receipt_id;
 IF r.outcome<>'COMMITTED' OR r.assignment_id IS DISTINCT FROM a.id OR r.assignment_revision_after IS DISTINCT FROM ar.revision
  OR a.subject_protocol<>'NATIVE_ACCOUNT' OR a.native_current_revision<1 OR role_row.subject_protocol<>'NATIVE_ACCOUNT'
  OR (r.recipient_account_id,r.role_id,r.role_revision,r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until)
   IS DISTINCT FROM (p_account,role_row.id,1::bigint,ar.state,ar.valid_from,ar.valid_until) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM public.native_company_policy_assert_effects_v1(p_company,r.receipt_id);
 clauses:=public.native_company_policy_business_clauses_v1(p_company,role_row.id);
 at_time:=clock_timestamp();
 IF ar.state<>'ACTIVE' OR role_row.status<>'ACTIVE' OR ar.valid_from>at_time OR ar.valid_until<=at_time THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 RETURN QUERY SELECT identity_row.company_epoch,identity_row.current_policy_receipt_id,a.id,a.native_current_revision,
  role_row.id,1::bigint,clauses,identity_row.company_name,identity_row.company_slug,ar.valid_from,ar.valid_until;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
