CREATE FUNCTION public.native_company_policy_clause_v1(p_org uuid,p_from timestamptz) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE action_ref record; fields jsonb;
BEGIN
 SELECT a.* INTO STRICT action_ref FROM public.native_company_action_refs a
  WHERE a.org_id=p_org AND a.catalog_version='native-payroll-collection-read-v1' AND a.action_key='payroll.collection.read';
 SELECT jsonb_agg(jsonb_build_object('org_id',p.org_id::text,'object_type_id',p.object_type_id::text,
  'property_id',p.property_id::text,'schema_revision',p.schema_revision::text)
  ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision) INTO fields
 FROM public.native_company_property_refs p WHERE p.org_id=p_org
  AND p.catalog_version='native-payroll-collection-read-v1' AND p.object_type_id=action_ref.object_type_id;
 IF jsonb_array_length(fields) IS DISTINCT FROM 18 OR p_from IS NULL OR NOT isfinite(p_from) THEN
  RAISE EXCEPTION 'native_company_policy.clause_custody_invalid';
 END IF;
 RETURN jsonb_build_object('kind','COMPANY_CAPABILITY_CLAUSE_V1',
  'action',jsonb_build_object('org_id',p_org::text,'object_type_id',action_ref.object_type_id::text,
   'action_type_id',action_ref.action_type_id::text,'registration_revision',action_ref.registration_revision::text,
   'manifest_digest',encode(action_ref.manifest_digest,'hex')),
  'resource',jsonb_build_object('kind','COMPANY','org_id',p_org::text),'fields',fields,
  'valid_from',to_char(p_from AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
  'valid_until',NULL,'delegable',false);
END
$body$;

CREATE FUNCTION public.native_company_policy_apply_assignment_v1(p_company uuid,p_receipt uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.native_company_policy_receipts_v1; origin public.company_authority_heads;
 clause jsonb; clause_hash bytea; role_hash bytea;
BEGIN
 r:=public.native_company_policy_effect_frame_v1(p_company,p_receipt);
 IF r.operation NOT IN (2,3) THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 SELECT h.* INTO STRICT origin FROM public.company_authority_heads h WHERE h.org_id=p_company;
 IF r.assignment_revision_before IS NULL THEN
  clause:=public.native_company_policy_clause_v1(p_company,r.executed_at);
  clause_hash:=sha256(convert_to(clause::text,'UTF8'));
  role_hash:=sha256(convert_to(jsonb_build_array(jsonb_build_object('clause_index',1,'clause_digest',encode(clause_hash,'hex')))::text,'UTF8'));
  INSERT INTO public.policy_roles(id,org_id,role_key,display_name,description,status,is_system,
   created_at,updated_at,subject_protocol,native_current_revision,created_by_account_id,updated_by_account_id,
   origin_account_id,origin_command_id,origin_receipt_id,policy_receipt_id)
  VALUES(r.role_id,p_company,'native_payroll_collection_read','급여 목록 열람',NULL,'ACTIVE',true,
   r.executed_at,r.executed_at,'NATIVE_ACCOUNT',1,r.actor_account_id,r.actor_account_id,
   origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id,r.receipt_id);
  INSERT INTO public.policy_role_revisions(org_id,role_id,revision,subject_protocol,state,valid_from,valid_until,
   catalog_version,manifest_digest,clause_digest,origin_account_id,origin_command_id,origin_receipt_id,
   actor_account_id,session_id,created_at,policy_receipt_id)
  VALUES(p_company,r.role_id,1,'NATIVE_ACCOUNT','ACTIVE',r.executed_at,NULL,r.catalog_version,r.manifest_digest,
   role_hash,origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id,
   r.actor_account_id,r.execution_session_id,r.executed_at,r.receipt_id);
  INSERT INTO public.policy_capability_clauses(org_id,role_id,role_revision,clause_index,effect,
   action_object_type_id,action_type_id,registration_revision,manifest_digest,resource_org_id,
   valid_from,valid_until,delegable,clause_digest)
  VALUES(p_company,r.role_id,1,1,'ALLOW',(clause->'action'->>'object_type_id')::uuid,
   (clause->'action'->>'action_type_id')::uuid,1,r.manifest_digest,p_company,r.executed_at,NULL,false,clause_hash);
  INSERT INTO public.policy_capability_clause_fields(org_id,role_id,role_revision,clause_index,
   object_type_id,property_id,schema_revision,manifest_digest,content_digest)
  SELECT p_company,r.role_id,1,1,p.object_type_id,p.property_id,p.schema_revision,p.manifest_digest,p.content_digest
   FROM public.native_company_property_refs p WHERE p.org_id=p_company AND p.catalog_version=r.catalog_version;
  INSERT INTO public.user_role_assignments(id,org_id,role_id,created_at,subject_protocol,account_id,
   native_current_revision,assigned_by_account_id,origin_account_id,origin_command_id,origin_receipt_id,policy_receipt_id)
  VALUES(r.assignment_id,p_company,r.role_id,r.executed_at,'NATIVE_ACCOUNT',r.recipient_account_id,1,r.actor_account_id,
   origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id,r.receipt_id);
 ELSE
  SELECT v.clause_digest INTO STRICT role_hash FROM public.policy_role_revisions v
   WHERE v.org_id=p_company AND v.role_id=r.role_id AND v.revision=1;
 END IF;
 INSERT INTO public.policy_assignment_revisions(org_id,assignment_id,revision,subject_protocol,account_id,
  role_id,role_revision,state,valid_from,valid_until,ceiling_digest,origin_account_id,origin_command_id,
  origin_receipt_id,actor_account_id,session_id,created_at,policy_receipt_id)
 VALUES(p_company,r.assignment_id,r.assignment_revision_after,'NATIVE_ACCOUNT',r.recipient_account_id,r.role_id,1,
  r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until,role_hash,
  origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id,r.actor_account_id,
  r.execution_session_id,r.executed_at,r.receipt_id);
 IF r.assignment_revision_before IS NOT NULL THEN
  UPDATE public.user_role_assignments a SET native_current_revision=r.assignment_revision_after
   WHERE a.org_id=p_company AND a.id=r.assignment_id AND a.native_current_revision=r.assignment_revision_before;
  IF NOT FOUND THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 END IF;
END
$body$;

CREATE FUNCTION public.native_company_policy_business_clauses_v1(p_org uuid,p_role uuid) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE role_row public.policy_roles; revision public.policy_role_revisions; clause_row public.policy_capability_clauses;
 creator public.native_company_policy_receipts_v1; clause jsonb; clause_hash bytea; role_hash bytea;
BEGIN
 SELECT r.* INTO STRICT role_row FROM public.policy_roles r WHERE r.org_id=p_org AND r.id=p_role;
 SELECT r.* INTO STRICT revision FROM public.policy_role_revisions r WHERE r.org_id=p_org AND r.role_id=p_role AND r.revision=1;
 SELECT r.* INTO STRICT creator FROM public.native_company_policy_receipts_v1 r WHERE r.org_id=p_org AND r.receipt_id=role_row.policy_receipt_id;
 SELECT c.* INTO STRICT clause_row FROM public.policy_capability_clauses c WHERE c.org_id=p_org AND c.role_id=p_role AND c.role_revision=1;
 clause:=public.native_company_policy_clause_v1(p_org,revision.valid_from);
 clause_hash:=sha256(convert_to(clause::text,'UTF8'));
 role_hash:=sha256(convert_to(jsonb_build_array(jsonb_build_object('clause_index',1,'clause_digest',encode(clause_hash,'hex')))::text,'UTF8'));
 IF creator.outcome<>'COMMITTED' OR creator.operation<>2 OR creator.assignment_revision_before IS NOT NULL
  OR creator.role_id IS DISTINCT FROM p_role OR role_row.role_key<>'native_payroll_collection_read'
  OR role_row.subject_protocol<>'NATIVE_ACCOUNT' OR role_row.native_current_revision<>1 OR role_row.status<>'ACTIVE'
  OR role_row.created_at IS DISTINCT FROM creator.executed_at OR role_row.updated_at IS DISTINCT FROM creator.executed_at
  OR role_row.created_by_account_id IS DISTINCT FROM creator.actor_account_id
  OR role_row.updated_by_account_id IS DISTINCT FROM creator.actor_account_id
  OR revision.policy_receipt_id IS DISTINCT FROM creator.receipt_id OR revision.state<>'ACTIVE'
  OR revision.valid_from IS DISTINCT FROM creator.executed_at OR revision.valid_until IS NOT NULL
  OR revision.actor_account_id IS DISTINCT FROM creator.actor_account_id OR revision.session_id IS DISTINCT FROM creator.execution_session_id
  OR revision.created_at IS DISTINCT FROM creator.executed_at OR revision.clause_digest IS DISTINCT FROM role_hash
  OR revision.catalog_version<>creator.catalog_version OR revision.manifest_digest IS DISTINCT FROM creator.manifest_digest
  OR clause_row.clause_index<>1 OR clause_row.effect<>'ALLOW' OR clause_row.delegable
  OR clause_row.resource_org_id<>p_org OR clause_row.valid_from IS DISTINCT FROM creator.executed_at
  OR clause_row.valid_until IS NOT NULL OR clause_row.clause_digest IS DISTINCT FROM clause_hash
  OR clause_row.action_object_type_id IS DISTINCT FROM (clause->'action'->>'object_type_id')::uuid
  OR clause_row.action_type_id IS DISTINCT FROM (clause->'action'->>'action_type_id')::uuid
  OR clause_row.registration_revision<>1 OR clause_row.manifest_digest IS DISTINCT FROM creator.manifest_digest
  OR (SELECT count(*) FROM public.policy_role_revisions v WHERE v.org_id=p_org AND v.role_id=p_role)<>1
  OR (SELECT count(*) FROM public.policy_capability_clause_fields f WHERE f.org_id=p_org AND f.role_id=p_role)<>18
  OR EXISTS(SELECT 1 FROM public.native_company_property_refs p WHERE p.org_id=p_org AND p.catalog_version=creator.catalog_version
   AND NOT EXISTS(SELECT 1 FROM public.policy_capability_clause_fields f WHERE f.org_id=p_org AND f.role_id=p_role
    AND f.role_revision=1 AND f.clause_index=1 AND f.object_type_id=p.object_type_id AND f.property_id=p.property_id
    AND f.schema_revision=p.schema_revision AND f.manifest_digest=p.manifest_digest AND f.content_digest=p.content_digest)) THEN
  RAISE EXCEPTION 'native_company_policy.clause_custody_invalid';
 END IF;
 RETURN jsonb_build_array(clause);
END
$body$;
