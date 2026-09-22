-- Fixed manifest is part of the command protocol; never accept caller JSON.
CREATE FUNCTION public.native_company_policy_manifest_v1() RETURNS jsonb
LANGUAGE sql IMMUTABLE SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT $manifest${"catalog_version":"native-payroll-collection-read-v1","object_types":[{"stable_key":"pay_run","title":"급여 실행","title_property_key":"source_label","backing_kind":"projected","backing_table":"payroll_draft_runs","primary_key_property":"id","properties":[{"key":"id","title":"식별자","field_type":"reference","config":{},"backing_column":"id","required":true,"in_property_policy":true},{"key":"period_start","title":"기간 시작","field_type":"date","config":{},"backing_column":"period_start","required":true,"in_property_policy":true},{"key":"period_end","title":"기간 종료","field_type":"date","config":{},"backing_column":"period_end","required":true,"in_property_policy":true},{"key":"source_label","title":"자료 이름","field_type":"text","config":{},"backing_column":"source_label","required":true,"in_property_policy":true},{"key":"status","title":"상태","field_type":"text","config":{},"backing_column":"status","required":true,"in_property_policy":true},{"key":"calculation_enabled","title":"계산 허용","field_type":"boolean","config":{},"backing_column":"calculation_enabled","required":true,"in_property_policy":true},{"key":"created_by","title":"작성자","field_type":"reference","config":{"nullable":true},"backing_column":"created_by","required":false,"in_property_policy":true},{"key":"approved_by","title":"승인자","field_type":"reference","config":{"nullable":true},"backing_column":"approved_by","required":false,"in_property_policy":true},{"key":"approved_at","title":"승인 시각","field_type":"timestamp","config":{"nullable":true,"precision":"microsecond","timezone":"UTC"},"backing_column":"approved_at","required":false,"in_property_policy":true},{"key":"close_receipt","title":"근태 마감 증빙 전체","field_type":"json","config":{"nullable":true},"backing_column":"close_receipt","required":false,"in_property_policy":true},{"key":"submitted_by","title":"제출자","field_type":"reference","config":{"nullable":true},"backing_column":"submitted_by","required":false,"in_property_policy":true},{"key":"submitted_at","title":"제출 시각","field_type":"timestamp","config":{"nullable":true,"precision":"microsecond","timezone":"UTC"},"backing_column":"submitted_at","required":false,"in_property_policy":true},{"key":"decided_by","title":"결정자","field_type":"reference","config":{"nullable":true},"backing_column":"decided_by","required":false,"in_property_policy":true},{"key":"decided_at","title":"결정 시각","field_type":"timestamp","config":{"nullable":true,"precision":"microsecond","timezone":"UTC"},"backing_column":"decided_at","required":false,"in_property_policy":true},{"key":"decision_reason","title":"결정 사유","field_type":"text","config":{"nullable":true},"backing_column":"decision_reason","required":false,"in_property_policy":true},{"key":"approval_ref","title":"승인 참조","field_type":"reference","config":{"nullable":true},"backing_column":"approval_ref","required":false,"in_property_policy":true},{"key":"created_at","title":"생성 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC"},"backing_column":"created_at","required":true,"in_property_policy":true},{"key":"updated_at","title":"수정 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC"},"backing_column":"updated_at","required":true,"in_property_policy":true}],"links":[],"actions":[{"stable_key":"collection_read","title":"회사 급여 목록 전체 항목 보기","params_schema":{"type":"object","additionalProperties":false,"properties":{"limit":{"type":"integer","minimum":-9223372036854775808,"maximum":9223372036854775807},"offset":{"type":"integer","minimum":-9223372036854775808,"maximum":9223372036854775807}},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"payroll.collection.read","control_points":["authority"]}],"analytics":[]}]}$manifest$::jsonb $body$;
ALTER FUNCTION public.native_company_policy_manifest_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_manifest_v1() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.native_company_policy_manifest_v1() TO console_ontology_writer;

CREATE FUNCTION public.native_company_policy_assert_payroll_catalog_v1(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE install public.native_company_catalog_installs; object_row public.ont_object_types;
 receipt public.native_company_policy_receipts_v1; wanted jsonb:=public.native_company_policy_manifest_v1()->'object_types'->0;
 properties jsonb; actions jsonb;
BEGIN
 SELECT i.* INTO STRICT install FROM public.native_company_catalog_installs i WHERE i.org_id=p_org
  AND i.catalog_version='native-payroll-collection-read-v1';
 SELECT r.* INTO STRICT receipt FROM public.native_company_policy_receipts_v1 r
  WHERE r.org_id=p_org AND r.receipt_id=install.policy_receipt_id;
 SELECT o.* INTO STRICT object_row FROM public.ont_object_types o WHERE o.org_id=p_org AND o.id=receipt.installed_object_type_id;
 IF receipt.operation<>1 OR receipt.outcome<>'COMMITTED' OR receipt.result_code<>'installed'
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
  OR (SELECT count(*) FROM public.native_company_action_refs r WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version)<>1
  OR (SELECT count(*) FROM public.native_company_property_refs r WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version)<>18
  OR EXISTS(SELECT 1 FROM public.native_company_property_refs r JOIN public.ont_property_defs p
    ON p.org_id=r.org_id AND p.id=r.property_id WHERE r.org_id=p_org AND r.catalog_version=install.catalog_version
     AND r.property_key IS DISTINCT FROM 'pay_run.'||p.key) THEN
  RAISE EXCEPTION 'native_company_policy.catalog_custody_invalid';
 END IF;
END
$body$;

CREATE FUNCTION ontology_api.install_native_company_payroll_catalog_v1(p_company uuid,p_receipt uuid)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); r public.native_company_policy_receipts_v1;
 origin record; manifest jsonb:=public.native_company_policy_manifest_v1(); snapshot jsonb;
BEGIN
 PERFORM set_config('app.current_org',p_company::text,true);
 r:=public.native_company_policy_effect_frame_v1(p_company,p_receipt);
 IF r.operation<>1 THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 SELECT i.origin_account_id,i.origin_command_id,i.origin_receipt_id INTO STRICT origin
  FROM public.native_company_catalog_installs i WHERE i.org_id=p_company
   AND i.catalog_version='native-company-identity-2026-09-19.1';
 IF sha256(convert_to(manifest::text,'UTF8')) IS DISTINCT FROM r.manifest_digest THEN
  RAISE EXCEPTION 'native_company_policy.manifest_invalid';
 END IF;
 snapshot:=manifest->'object_types'->0;
 PERFORM pg_advisory_xact_lock(hashtextextended('ontology-bootstrap:'||p_company::text,0));
 INSERT INTO public.ont_object_type_key_revisions(org_id,stable_key,created_at,updated_at)
 VALUES(p_company,'pay_run',r.executed_at,r.executed_at);
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
 SELECT p_company,r.catalog_version,'pay_run.'||p.key,p.object_type_id,p.id,1,r.manifest_digest,
  sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,'config',p.config,
   'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8'))
 FROM public.ont_property_defs p WHERE p.org_id=p_company AND p.object_type_id=r.installed_object_type_id;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN r.installed_object_type_id;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
