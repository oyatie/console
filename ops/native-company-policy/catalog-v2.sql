-- Additive dual-codec successor; installed only after exact predecessor custody verification.

CREATE FUNCTION public.native_company_people_manifest_v1() RETURNS jsonb
LANGUAGE sql IMMUTABLE SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT $manifest${"catalog_version":"native-people-directory-v1","object_types":[{"stable_key":"person","title":"사람 목록","title_property_key":"legal_name","backing_kind":"projected","backing_table":"employees","primary_key_property":"id","properties":[{"key":"employee_id","title":"사람 목록 식별자","field_type":"reference","config":{},"backing_column":"id","required":true,"in_property_policy":true},{"key":"person_id","title":"사람 식별자","field_type":"reference","config":{},"backing_column":null,"required":true,"in_property_policy":true},{"key":"legal_name","title":"이름","field_type":"text","config":{"maxLength":200},"backing_column":null,"required":true,"in_property_policy":true},{"key":"employee_number","title":"사번","field_type":"text","config":{"maxLength":64},"backing_column":"employee_number","required":true,"in_property_policy":true},{"key":"person_version","title":"사람 기록 버전","field_type":"integer","config":{"minimum":1},"backing_column":null,"required":true,"in_property_policy":true},{"key":"directory_registered_at","title":"목록 등록 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC"},"backing_column":"created_at","required":true,"in_property_policy":true}],"links":[],"actions":[{"stable_key":"directory_create","title":"사람 목록에 등록","params_schema":{"type":"object","additionalProperties":false,"properties":{"legal_name":{"type":"string","minLength":1,"maxLength":200},"employee_number":{"type":"string","minLength":1,"maxLength":64}},"required":["legal_name","employee_number"]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"people.directory.create","control_points":["authority"]},{"stable_key":"directory_read","title":"사람 목록 열람","params_schema":{"type":"object","additionalProperties":false,"properties":{"limit":{"type":"integer","minimum":1,"maximum":100},"after_employee_id":{"type":"string","format":"uuid"}},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"people.directory.read","control_points":["authority"]}],"analytics":[]}]}$manifest$::jsonb $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_assert_people_catalog_v1(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE install public.native_company_catalog_installs; object_row public.ont_object_types;
 receipt public.native_company_policy_receipts_v1; wanted jsonb:=public.native_company_people_manifest_v1()->'object_types'->0;
 properties jsonb; actions jsonb;
BEGIN
 SELECT i.* INTO STRICT install FROM public.native_company_catalog_installs i WHERE i.org_id=p_org
  AND i.catalog_version='native-people-directory-v1';
 SELECT r.* INTO STRICT receipt FROM public.native_company_policy_receipts_v1 r
  WHERE r.org_id=p_org AND r.receipt_id=install.policy_receipt_id;
 SELECT o.* INTO STRICT object_row FROM public.ont_object_types o WHERE o.org_id=p_org AND o.id=receipt.installed_object_type_id;
 IF receipt.codec_version<>2 OR receipt.catalog_version<>'native-people-directory-v1'
  OR receipt.manifest_digest<>decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')
  OR receipt.operation<>1 OR receipt.outcome<>'COMMITTED' OR receipt.result_code<>'installed'
  OR object_row.policy_receipt_id IS DISTINCT FROM receipt.receipt_id
  OR object_row.created_by_account_id IS DISTINCT FROM receipt.actor_account_id
  OR object_row.created_at IS DISTINCT FROM receipt.executed_at OR object_row.updated_at IS DISTINCT FROM receipt.executed_at
  OR object_row.schema_version<>1 OR object_row.lifecycle_state<>'published'
  OR (object_row.stable_key,object_row.title,object_row.title_property_key,object_row.backing_kind,
    object_row.backing_table,object_row.primary_key_property) IS DISTINCT FROM
    (wanted->>'stable_key',wanted->>'title',wanted->>'title_property_key',wanted->>'backing_kind',
     wanted->>'backing_table',wanted->>'primary_key_property')
  OR install.manifest_digest IS DISTINCT FROM receipt.manifest_digest OR install.installed_at IS DISTINCT FROM receipt.executed_at
  OR NOT EXISTS(SELECT 1 FROM public.ont_builtin_catalog_installs i WHERE i.org_id=p_org
    AND i.catalog_version=install.catalog_version AND i.manifest_digest=install.manifest_digest
    AND i.policy_receipt_id=receipt.receipt_id AND i.installed_by_account_id=receipt.actor_account_id
    AND i.installed_at=receipt.executed_at AND i.attribution_protocol='NATIVE_ACCOUNT'
    AND (i.origin_account_id,i.origin_command_id,i.origin_receipt_id)=
     (install.origin_account_id,install.origin_command_id,install.origin_receipt_id)) THEN
  RAISE EXCEPTION 'native_company_policy.catalog_custody_invalid';
 END IF;
 SELECT jsonb_agg(jsonb_build_object('key',p.key,'title',p.title,'field_type',p.type,'config',p.config,
  'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy) ORDER BY p.key)
  INTO properties FROM public.ont_property_defs p WHERE p.org_id=p_org AND p.object_type_id=object_row.id;
 SELECT jsonb_agg(jsonb_build_object('stable_key',a.stable_key,'title',a.title,'params_schema',a.params_schema,
  'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,'dispatch',a.dispatch,
  'dispatch_target',a.dispatch_target,'control_points',a.control_points) ORDER BY a.stable_key)
  INTO actions FROM public.ont_action_types a WHERE a.org_id=p_org AND a.object_type_id=object_row.id;
 IF properties IS DISTINCT FROM (SELECT jsonb_agg(v ORDER BY v->>'key') FROM jsonb_array_elements(wanted->'properties') v)
  OR actions IS DISTINCT FROM wanted->'actions'
  OR (SELECT count(*) FROM public.native_company_object_refs r WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version)<>1
  OR (SELECT count(*) FROM public.native_company_action_refs r WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version)<>2
  OR (SELECT count(*) FROM public.native_company_property_refs r WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version)<>6
  OR EXISTS(SELECT 1 FROM public.native_company_property_refs r JOIN public.ont_property_defs p
    ON p.org_id=r.org_id AND p.id=r.property_id WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version
     AND r.property_key IS DISTINCT FROM 'person.'||p.key) THEN
  RAISE EXCEPTION 'native_company_policy.catalog_custody_invalid';
 END IF;
END
$body$;

CREATE OR REPLACE FUNCTION ontology_api.install_native_company_people_catalog_v1(p_company uuid,p_receipt uuid)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); r public.native_company_policy_receipts_v1;
 origin record; manifest jsonb:=public.native_company_people_manifest_v1(); snapshot jsonb;
BEGIN
 PERFORM set_config('app.current_org',p_company::text,true);
 r:=public.native_company_policy_effect_frame_v1(p_company,p_receipt);
 IF r.codec_version<>2 OR r.catalog_version<>'native-people-directory-v1' OR r.operation<>1 THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 SELECT i.origin_account_id,i.origin_command_id,i.origin_receipt_id INTO STRICT origin
  FROM public.native_company_catalog_installs i WHERE i.org_id=p_company
   AND i.catalog_version='native-company-identity-2026-09-19.1';
 IF sha256(convert_to(manifest::text,'UTF8')) IS DISTINCT FROM r.manifest_digest THEN
  RAISE EXCEPTION 'native_company_policy.manifest_invalid';
 END IF;
 snapshot:=manifest->'object_types'->0;
 PERFORM pg_advisory_xact_lock(hashtextextended('ontology-bootstrap:'||p_company::text,0));
 INSERT INTO public.ont_object_type_key_revisions(org_id,stable_key,created_at,updated_at)
 VALUES(p_company,'person',r.executed_at,r.executed_at);
 INSERT INTO public.ont_object_types(id,org_id,stable_key,title,title_property_key,backing_kind,
  backing_table,primary_key_property,schema_version,lifecycle_state,created_by,created_at,updated_at,
  attribution_protocol,created_by_account_id,origin_account_id,origin_command_id,origin_receipt_id,policy_receipt_id)
 VALUES(r.installed_object_type_id,p_company,snapshot->>'stable_key',snapshot->>'title',snapshot->>'title_property_key',
  snapshot->>'backing_kind',snapshot->>'backing_table',snapshot->>'primary_key_property',1,'published',NULL,
  r.executed_at,r.executed_at,'NATIVE_ACCOUNT',r.actor_account_id,
  origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id,r.receipt_id);
 INSERT INTO public.ont_property_defs(id,org_id,object_type_id,key,title,type,config,backing_column,
  required,in_property_policy,created_at)
 SELECT gen_random_uuid(),p_company,r.installed_object_type_id,p->>'key',p->>'title',p->>'field_type',
  p->'config',p->>'backing_column',(p->>'required')::boolean,(p->>'in_property_policy')::boolean,r.executed_at
 FROM jsonb_array_elements(snapshot->'properties') p;
 INSERT INTO public.ont_action_types(id,org_id,object_type_id,stable_key,title,params_schema,edits,
  submission_criteria,side_effects,dispatch,dispatch_target,control_points,created_at)
 SELECT gen_random_uuid(),p_company,r.installed_object_type_id,a->>'stable_key',a->>'title',a->'params_schema',
  a->'edits',a->'submission_criteria',a->'side_effects',a->>'dispatch',a->>'dispatch_target',a->'control_points',r.executed_at
 FROM jsonb_array_elements(snapshot->'actions') a;
 INSERT INTO public.ont_builtin_catalog_installs(org_id,catalog_version,manifest_digest,installed_by,installed_at,
  attribution_protocol,installed_by_account_id,origin_account_id,origin_command_id,origin_receipt_id,policy_receipt_id)
 VALUES(p_company,r.catalog_version,r.manifest_digest,NULL,r.executed_at,'NATIVE_ACCOUNT',r.actor_account_id,
  origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id,r.receipt_id);
 INSERT INTO public.native_company_catalog_installs(org_id,catalog_version,manifest_digest,
  origin_account_id,origin_command_id,origin_receipt_id,installed_at,policy_receipt_id)
 VALUES(p_company,r.catalog_version,r.manifest_digest,origin.origin_account_id,origin.origin_command_id,
  origin.origin_receipt_id,r.executed_at,r.receipt_id);
 INSERT INTO public.native_company_object_refs(org_id,catalog_version,manifest_digest,object_key,object_type_id,schema_revision,content_digest)
 SELECT p_company,r.catalog_version,r.manifest_digest,o.stable_key,o.id,1,
  sha256(convert_to(jsonb_build_object('stable_key',o.stable_key,'title',o.title,'title_property_key',o.title_property_key,
   'backing_kind',o.backing_kind,'backing_table',o.backing_table,'primary_key_property',o.primary_key_property,
   'schema_version',o.schema_version,'lifecycle_state',o.lifecycle_state)::text,'UTF8'))
 FROM public.ont_object_types o WHERE o.org_id=p_company AND o.id=r.installed_object_type_id;
 INSERT INTO public.native_company_action_refs(org_id,catalog_version,action_key,object_type_id,action_type_id,
  registration_revision,manifest_digest,content_digest)
 SELECT p_company,r.catalog_version,a.dispatch_target,a.object_type_id,a.id,1,r.manifest_digest,
  sha256(convert_to(jsonb_build_object('stable_key',a.stable_key,'title',a.title,'params_schema',a.params_schema,
   'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,'dispatch',a.dispatch,
   'dispatch_target',a.dispatch_target,'control_points',a.control_points)::text,'UTF8'))
 FROM public.ont_action_types a WHERE a.org_id=p_company AND a.object_type_id=r.installed_object_type_id;
 INSERT INTO public.native_company_property_refs(org_id,catalog_version,property_key,object_type_id,property_id,
  schema_revision,manifest_digest,content_digest)
 SELECT p_company,r.catalog_version,'person.'||p.key,p.object_type_id,p.id,1,r.manifest_digest,
  sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,'config',p.config,
   'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8'))
 FROM public.ont_property_defs p WHERE p.org_id=p_company AND p.object_type_id=r.installed_object_type_id;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN r.installed_object_type_id;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
