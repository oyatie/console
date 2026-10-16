-- Additive read ABI. Install only with the complete reviewed owner/ACL/custody
-- successor. A returned selector is never authentication or authorization.
CREATE FUNCTION public.identity_native_group_process_incarnation_selector_v1(
 p_actor uuid,p_family uuid,p_group uuid,p_command uuid) RETURNS uuid
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE incarnation uuid; original jsonb; origin_columns integer;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR current_setting('server_encoding') IS DISTINCT FROM 'UTF8'
  OR num_nonnulls(p_actor,p_family,p_group)<>3
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_family,p_group,p_command) THEN
  RAISE EXCEPTION 'native_group_process.material_unavailable';
 END IF;
 PERFORM public.native_group_process_group_guard_v1(p_group,false);
 IF p_command IS NULL THEN
  -- Native birth retains this same historical guard. Take any live topology
  -- row before Account/family; a later scope rechecks the selected incarnation.
  PERFORM 1 FROM public.group_authority_lock_shared_v1(p_group);
 END IF;
 PERFORM public.native_group_process_account_material_v1(p_actor,p_family);
 IF p_command IS NULL THEN
  SELECT num_nonnulls(g.origin_account_id,g.origin_command_id,g.origin_receipt_id) INTO origin_columns
   FROM public.groups g WHERE g.id=p_group;
  IF NOT FOUND OR origin_columns=0 THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
  END IF;
  IF origin_columns<>3 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  SELECT h.incarnation INTO incarnation FROM public.group_authority_heads h WHERE h.group_id=p_group;
  IF NOT FOUND THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  PERFORM public.native_group_process_current_context_v1(p_actor,p_family,p_group,incarnation,false);
  PERFORM public.native_group_process_assert_current_v1(p_group,incarnation);
 ELSE
  -- Historical receipt selection has no live-head/designation prerequisite.
  -- The immutable input supplies the incarnation; do not reinterpret bytes.
  PERFORM public.native_group_process_command_guard_v1(p_actor,p_command,false);
  SELECT i.group_incarnation INTO incarnation FROM public.native_group_process_inputs_v1 i
   WHERE i.actor_account_id=p_actor AND i.command_id=p_command AND i.group_id=p_group;
  IF NOT FOUND THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
  END IF;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  original:=public.native_group_process_original_material_v1(p_actor,p_group,incarnation,p_command);
  IF original IS NULL THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 END IF;
 RETURN incarnation;
END
$body$;

CREATE FUNCTION public.identity_native_group_process_navigation_candidates_v1(
 p_actor uuid,p_family uuid,p_original bytea) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE original jsonb; snapshot_text text; captured jsonb; snapshot_bytes bytea;
 old_groups uuid[]:=ARRAY[]::uuid[]; current_groups uuid[]; guarded_groups uuid[]; after_groups uuid[];
 selected_group uuid; candidate jsonb; previous_group uuid; group_value jsonb; history_value jsonb;
 candidates jsonb:='[]'::jsonb; groups_value jsonb:='[]'::jsonb; consent_value jsonb:='[]'::jsonb;
 material jsonb; account_value jsonb; designation_value jsonb; item record; head_revision bigint;
 budget integer:=1048576; additional integer; first_group boolean:=true; first_history boolean;
 root public.accounts; security public.account_security; enrollment public.account_security_events;
 family record; terms record; designation public.deployment_operator_head; designation_receipt public.deployment_operator_receipts;
 rows_to_validate jsonb:='[]'::jsonb; checked_row jsonb; expected_keys text[];
 -- Exact schema types/nullability; no field-name inference. Native closure
 -- tightens nullable database columns only where that row purpose requires it.
 row_shapes jsonb:=$shapes${"account_security":{"account_id":["uuid",false],"context_generation":["integer",false],"revision":["integer",false],"security_generation":["integer",false],"security_state":["string",false],"updated_at":["timestamp",false]},"account_security_events":{"account_id":["uuid",false],"actor_account_id":["uuid",false],"evidence_ref":["object",false],"id":["uuid",false],"kind":["string",false],"occurred_at":["timestamp",false],"payload":["object",false],"session_id":["uuid",false]},"accounts":{"created_at":["timestamp",false],"id":["uuid",false]},"company_enrollment_receipts":{"account_id":["uuid",false],"action_refs":["array",false],"administrative_account_id":["uuid",false],"catalog_version":["string",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_at":["timestamp",false],"designation_receipt_id":["uuid",false],"group_id":["uuid",false],"input_digest":["bytea",false],"manifest_digest":["bytea",false],"org_id":["uuid",false],"property_refs":["array",false],"receipt_id":["uuid",false],"root_assignment_id":["uuid",false],"root_revision":["integer",false],"session_id":["uuid",false]},"company_enrollment_requests":{"account_id":["uuid",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_receipt_id":["uuid",false],"created_at":["timestamp",false],"designation_receipt_id":["uuid",false],"expires_at":["timestamp",false],"input_bytes":["bytea",true],"input_digest":["bytea",false],"state":["string",false],"terminal_at":["timestamp",false]},"consent":{"accepted_at":["timestamp",false],"content_sha256":["bytea",false],"manifest_sha256":["bytea",false],"terms_kind":["string",false]},"deployment_operator_head":{"account_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"receipt_id":["uuid",false],"revision":["integer",false],"singleton":["integer",false],"system_identifier":["string",false]},"deployment_operator_receipts":{"account_id":["uuid",false],"command_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"expected_revision":["integer",false],"expected_security_generation":["integer",true],"kind":["string",false],"reason":["string",true],"receipt_id":["uuid",false],"recorded_at":["timestamp",false],"revision":["integer",false],"system_identifier":["string",false]},"family":{"account_security_generation":["integer",false],"assurance":["string",false],"auth_time":["timestamp",false],"created_at":["timestamp",false],"org_id":["uuid",true],"protocol":["string",false],"revoked_at":["timestamp",true],"user_id":["uuid",false]},"group_authority_heads":{"group_id":["uuid",false],"incarnation":["uuid",false],"revision":["integer",false],"state":["string",false]},"groups":{"created_at":["timestamp",false],"id":["uuid",false],"name":["string",false],"origin_account_id":["uuid",false],"origin_command_id":["uuid",false],"origin_receipt_id":["uuid",false],"slug":["string",false],"status":["string",false],"updated_at":["timestamp",false]},"native_group_identity_policy_heads_v1":{"activated_at":["timestamp",false],"activation_receipt_id":["uuid",false],"codec_contract_digest":["bytea",false],"first_actor_account_id":["uuid",false],"first_command_id":["uuid",false],"first_input_digest":["bytea",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"revision":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"native_group_process_head_revisions_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_heads_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_versions_v1":{"account_possession_procedure":["string",false],"actor_account_id":["uuid",false],"admitted_at":["timestamp",false],"adopt_command_id":["uuid",false],"content_digest":["bytea",false],"designation_receipt_id":["uuid",false],"designation_revision":["integer",false],"duplicate_contradictory_claim_procedure":["string",false],"escalation_adjudication_procedure":["string",false],"evidence_minimization_retention_description":["string",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"input_digest":["bytea",false],"intended_claimant_matching_procedure":["string",false],"method":["string",false],"operator_responsibility":["integer",false],"physical_human_evidence_procedure":["string",false],"policy_head_digest":["bytea",false],"policy_revision":["integer",false],"process_id":["uuid",false],"qualification_criteria_instruction":["string",false],"recipient_responsibility":["string",false],"schema_id":["string",false],"title":["string",false],"version":["integer",false]},"source":{"cedar_language_version":["string",false],"cedar_sdk_version":["string",false],"codec_contract_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"terms_head":{"manifest_sha256":["bytea",false],"release_receipt_id":["uuid",false],"revision":["integer",false]}}$shapes$::jsonb;
 document jsonb; required text; incarnation uuid; row_key text; row_value jsonb;
 entry jsonb; revision_value text; field_shape jsonb; value_text text; parsed_time timestamptz;
 birth_request public.company_enrollment_requests; birth_receipt public.company_enrollment_receipts;
 birth_effect public.company_enrollment_effect_bindings; decoded_birth record;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR current_setting('server_encoding') IS DISTINCT FROM 'UTF8'
  OR num_nonnulls(p_actor,p_family)<>2
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_family) THEN
  RAISE EXCEPTION 'native_group_process.material_unavailable';
 END IF;
 IF p_original IS NOT NULL THEN
  IF octet_length(p_original) NOT BETWEEN 1 AND 1048576 THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  BEGIN
   snapshot_text:=convert_from(p_original,'UTF8'); original:=snapshot_text::jsonb;
  EXCEPTION WHEN invalid_text_representation OR character_not_in_repertoire THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END;
  -- Equality rejects duplicate members and alternate serialization before any
  -- source selector is used. This is an internal observation, not permission.
  IF convert_to(original::text,'UTF8') IS DISTINCT FROM p_original
   OR jsonb_typeof(original) IS DISTINCT FROM 'object'
   OR original->>'protocol' IS DISTINCT FROM 'GROUP_PROCESS_NAVIGATION_SOURCE_V1'
   OR ARRAY(SELECT key FROM jsonb_object_keys(original) key ORDER BY key COLLATE "C")
      IS DISTINCT FROM ARRAY['account','designation','groups','protocol']
   OR original#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR original#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR jsonb_typeof(original->'groups') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(original->'groups')>256 THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(original->'groups') LOOP
   IF jsonb_typeof(candidate) IS DISTINCT FROM 'object'
    OR jsonb_typeof(candidate->'group_id') IS DISTINCT FROM 'string'
    OR candidate->>'group_id' !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
    OR jsonb_typeof(candidate->'group_incarnation') IS DISTINCT FROM 'string'
    OR candidate->>'group_incarnation' !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   selected_group:=(candidate->>'group_id')::uuid;
   incarnation:=(candidate->>'group_incarnation')::uuid;
   IF '00000000-0000-0000-0000-000000000000'::uuid IN(selected_group,incarnation)
    OR (previous_group IS NOT NULL AND selected_group<=previous_group) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   old_groups:=array_append(old_groups,selected_group); previous_group:=selected_group;
  END LOOP;
  document:=original;
  IF jsonb_typeof(document->'account') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF ARRAY(SELECT key FROM jsonb_object_keys(document->'account') key ORDER BY key COLLATE "C")
   IS DISTINCT FROM ARRAY['actor_account_id','consent','enrollment','family','root','security','session_id','terms_head']
   OR document#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR document#>>'{account,root,id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,security,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,family,user_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,kind}' IS DISTINCT FROM 'ENROLLED'
   OR jsonb_typeof(document#>'{account,consent}') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(document#>'{account,consent}') NOT BETWEEN 1 AND 8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  rows_to_validate:=jsonb_build_array(jsonb_build_array('accounts',document#>'{account,root}'),
   jsonb_build_array('account_security',document#>'{account,security}'),
   jsonb_build_array('family',document#>'{account,family}'),
   jsonb_build_array('account_security_events',document#>'{account,enrollment}'),
   jsonb_build_array('terms_head',document#>'{account,terms_head}'));
  required:=NULL;
  FOR entry IN SELECT value FROM jsonb_array_elements(document#>'{account,consent}') LOOP
   IF jsonb_typeof(entry->'terms_kind') IS DISTINCT FROM 'string'
    OR entry->>'terms_kind' !~ '^[a-z][a-z0-9_.-]{0,63}$'
    OR (required IS NOT NULL AND (entry->>'terms_kind') COLLATE "C"<=required COLLATE "C") THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   required:=entry->>'terms_kind'; rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('consent',entry));
  END LOOP;
  IF document->'designation' IS DISTINCT FROM 'null'::jsonb THEN
   IF jsonb_typeof(document->'designation') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   IF ARRAY(SELECT key FROM jsonb_object_keys(document->'designation') key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','receipt'] THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('deployment_operator_head',document#>'{designation,head}'),
    jsonb_build_array('deployment_operator_receipts',document#>'{designation,receipt}'));
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(document->'groups') LOOP
   IF ARRAY(SELECT key FROM jsonb_object_keys(candidate) key ORDER BY key COLLATE "C")
    IS DISTINCT FROM ARRAY['birth_designation_receipt','birth_receipt','birth_request','group','group_id','group_incarnation','head','history','policy','source','topology','version']
    OR candidate#>>'{group,id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
    OR candidate#>>'{birth_receipt,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR jsonb_typeof(candidate->'history') IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   IF (candidate->'policy'='null'::jsonb) IS DISTINCT FROM (candidate->'head'='null'::jsonb)
    OR (candidate->'head'='null'::jsonb) IS DISTINCT FROM (candidate->'version'='null'::jsonb)
    OR ((candidate->'head'='null'::jsonb) IS DISTINCT FROM (jsonb_array_length(candidate->'history')=0)) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('groups',candidate->'group'),jsonb_build_array('group_authority_heads',candidate->'topology'),
    jsonb_build_array('company_enrollment_requests',candidate->'birth_request'),
    jsonb_build_array('company_enrollment_receipts',candidate->'birth_receipt'),
    jsonb_build_array('deployment_operator_receipts',candidate->'birth_designation_receipt'),
    jsonb_build_array('source',candidate->'source'));
   IF candidate->'policy' IS DISTINCT FROM 'null'::jsonb THEN
    rows_to_validate:=rows_to_validate||jsonb_build_array(
     jsonb_build_array('native_group_identity_policy_heads_v1',candidate->'policy'),
     jsonb_build_array('native_group_process_heads_v1',candidate->'head'),
     jsonb_build_array('native_group_process_versions_v1',candidate->'version'));
   END IF;
   head_revision:=0;
   FOR entry IN SELECT value FROM jsonb_array_elements(candidate->'history') LOOP
    IF jsonb_typeof(entry) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    IF ARRAY(SELECT key FROM jsonb_object_keys(entry) key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','reason']
     OR jsonb_typeof(entry->'reason') NOT IN ('string','null')
     OR entry#>>'{head,group_id}' IS DISTINCT FROM candidate->>'group_id'
     OR entry#>>'{head,group_incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
     OR entry#>>'{head,process_id}' IS DISTINCT FROM candidate#>>'{head,process_id}' THEN
     RAISE EXCEPTION 'native_group_process.material_unavailable';
    END IF;
    revision_value:=entry#>>'{head,head_revision}';
    IF revision_value IS NULL OR revision_value !~ '^[1-9][0-9]{0,18}$' OR revision_value::numeric>9223372036854775807
     OR revision_value::numeric<>head_revision::numeric+1 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    head_revision:=revision_value::bigint;
    rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('native_group_process_head_revisions_v1',entry->'head'));
   END LOOP;
   IF head_revision>0 AND (candidate->'history'->-1->'head') IS DISTINCT FROM candidate->'head' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END LOOP;
  FOR checked_row IN SELECT value FROM jsonb_array_elements(rows_to_validate) LOOP
   IF jsonb_typeof(checked_row->1) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   expected_keys:=ARRAY(SELECT key FROM jsonb_object_keys(row_shapes->(checked_row->>0)) key ORDER BY key COLLATE "C");
   IF cardinality(expected_keys)=0 OR ARRAY(SELECT key FROM jsonb_object_keys(checked_row->1) key ORDER BY key COLLATE "C") IS DISTINCT FROM expected_keys THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   FOR row_key,row_value IN SELECT key,value FROM jsonb_each(checked_row->1) LOOP
    field_shape:=row_shapes->(checked_row->>0)->row_key;
    IF row_value='null'::jsonb THEN
     IF field_shape->1 IS DISTINCT FROM 'true'::jsonb THEN
      RAISE EXCEPTION 'native_group_process.material_unavailable';
     END IF;
     CONTINUE;
    END IF;
    value_text:=row_value#>>'{}';
    CASE field_shape->>0
     WHEN 'uuid' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string'
       OR value_text !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
       OR value_text='00000000-0000-0000-0000-000000000000' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'integer' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'number' OR value_text !~ '^(0|[1-9][0-9]{0,18})$' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
      IF value_text::numeric>9223372036854775807 OR (value_text='0' AND row_key<>'expected_revision') THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'timestamp' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
      parsed_time:=value_text::timestamptz;
      IF NOT isfinite(parsed_time) OR to_jsonb(parsed_time) IS DISTINCT FROM row_value THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'bytea' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' OR left(value_text,2) IS DISTINCT FROM '\x'
       OR substring(value_text FROM 3) !~ '^([0-9a-f]{2})*$'
       OR (right(row_key,6)='digest' AND length(value_text)<>66)
       OR (right(row_key,6)='sha256' AND length(value_text)<>66) THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     ELSE
      IF jsonb_typeof(row_value) IS DISTINCT FROM field_shape->>0 THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
    END CASE;
   END LOOP;
  END LOOP;
 END IF;

 SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO current_groups
  FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
 IF cardinality(current_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 IF p_original IS NOT NULL THEN
  SELECT coalesce(array_agg(id ORDER BY id),ARRAY[]::uuid[]) INTO guarded_groups
   FROM (SELECT DISTINCT unnest(old_groups||current_groups) id) required_guards;
  FOREACH selected_group IN ARRAY guarded_groups LOOP
   PERFORM public.native_group_process_group_guard_v1(selected_group,false);
  END LOOP;
  FOREACH selected_group IN ARRAY guarded_groups LOOP
   PERFORM 1 FROM public.group_authority_lock_shared_v1(selected_group);
  END LOOP;
  SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO after_groups
   FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
  IF cardinality(after_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF NOT after_groups<@guarded_groups THEN
   RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
    'session_id',p_family,'status','CONFLICT','candidates','[]'::jsonb,'snapshot',NULL);
  END IF;
  current_groups:=after_groups;
 END IF;

 -- Enumeration takes no Group locks. Account guards end with this transaction
 -- before per-candidate Landing7 scopes. Final recheck has taken all Group
 -- guards already; neither branch acquires a new Group guard after Account.
 material:=public.native_group_process_account_material_v1(p_actor,p_family);
 SELECT a.* INTO STRICT root FROM public.accounts a WHERE a.id=p_actor;
 SELECT s.* INTO STRICT security FROM public.account_security s WHERE s.account_id=p_actor;
 SELECT e.* INTO STRICT enrollment FROM public.account_security_events e WHERE e.account_id=p_actor AND e.kind='ENROLLED';
 SELECT f.* INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_actor,p_family) f;
 SELECT h.* INTO STRICT terms FROM public.account_terms_registration_head_v1() h;
 FOR item IN SELECT c.* FROM public.account_login_consent_v1(p_actor) c ORDER BY c.terms_kind COLLATE "C" LIMIT 9 LOOP
  IF jsonb_array_length(consent_value)>=8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  additional:=octet_length(convert_to(to_jsonb(item)::text,'UTF8'))+2;
  IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  consent_value:=consent_value||jsonb_build_array(to_jsonb(item)); budget:=budget-additional;
 END LOOP;
 IF jsonb_array_length(consent_value)=0 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 account_value:=jsonb_build_object('actor_account_id',p_actor,'session_id',p_family,
  'root',to_jsonb(root),'security',to_jsonb(security),'family',to_jsonb(family),
  'enrollment',to_jsonb(enrollment),'consent',consent_value,'terms_head',to_jsonb(terms));
 PERFORM public.account_company_setup_eligibility_v1(p_actor);
 SELECT h.* INTO designation FROM public.deployment_operator_head h WHERE h.singleton=1 FOR SHARE OF h;
 IF FOUND THEN
  SELECT r.* INTO STRICT designation_receipt FROM public.deployment_operator_receipts r WHERE r.receipt_id=designation.receipt_id;
  IF (designation.receipt_id,designation.revision,designation.account_id,designation.system_identifier,designation.database_name,designation.database_oid)
   IS DISTINCT FROM (designation_receipt.receipt_id,designation_receipt.revision,designation_receipt.account_id,
    designation_receipt.system_identifier,designation_receipt.database_name,designation_receipt.database_oid)
   OR designation_receipt.expected_revision IS DISTINCT FROM designation_receipt.revision-1
   OR ((designation_receipt.kind='DESIGNATE' AND designation_receipt.revision=1
     AND designation_receipt.expected_security_generation>0 AND designation_receipt.reason IS NULL)
    OR (designation_receipt.kind='REVOKE' AND designation_receipt.revision>1
     AND designation_receipt.expected_security_generation IS NULL AND designation_receipt.reason IS NOT NULL
     AND octet_length(designation_receipt.reason) BETWEEN 1 AND 512
     AND length(btrim(designation_receipt.reason,E' \t\n\r\f\013'))>0)) IS DISTINCT FROM true
   OR designation.system_identifier IS DISTINCT FROM (SELECT system_identifier::text FROM pg_control_system())
   OR designation.database_name IS DISTINCT FROM current_database()::text
   OR designation.database_oid IS DISTINCT FROM (SELECT oid::bigint FROM pg_database WHERE datname=current_database()) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  designation_value:=jsonb_build_object('head',to_jsonb(designation),'receipt',to_jsonb(designation_receipt));
 END IF;
 captured:=jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_SOURCE_V1',
  'account',account_value,'designation',designation_value,'groups',groups_value);
 -- The base already contains consent. Reset to its exact canonical byte size;
 -- subsequent accounting replaces the empty array with bounded complete rows.
 budget:=1048576-octet_length(convert_to(captured::text,'UTF8'));
 IF budget<0 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;

 FOREACH selected_group IN ARRAY current_groups LOOP
  -- One SPI snapshot binds policy/head/content. History is the immutable prefix
  -- ending at that captured head, even when enumeration races a later commit.
  SELECT jsonb_build_object('group_id',g.id,'group_incarnation',t.incarnation,
    'group',to_jsonb(g),'topology',to_jsonb(t),'birth_request',to_jsonb(q),'birth_receipt',to_jsonb(r),
    'birth_designation_receipt',to_jsonb(d),'source',public.native_group_process_source_v1(g.id,t.incarnation),
    'policy',to_jsonb(p),'head',to_jsonb(h),'version',to_jsonb(v),'history','[]'::jsonb)
   INTO STRICT group_value FROM public.groups g
   LEFT JOIN public.group_authority_heads t ON t.group_id=g.id
   LEFT JOIN public.company_enrollment_receipts r ON
    (r.account_id,r.command_id,r.receipt_id,r.group_id)=(g.origin_account_id,g.origin_command_id,g.origin_receipt_id,g.id)
   LEFT JOIN public.company_enrollment_requests q ON
    (q.account_id,q.command_id,q.committed_receipt_id,q.state)=(r.account_id,r.command_id,r.receipt_id,'COMMITTED')
   LEFT JOIN public.deployment_operator_receipts d ON d.receipt_id=r.designation_receipt_id
   LEFT JOIN public.native_group_identity_policy_heads_v1 p ON (p.group_id,p.group_incarnation)=(g.id,t.incarnation)
   LEFT JOIN public.native_group_process_heads_v1 h ON (h.group_id,h.group_incarnation)=(g.id,t.incarnation)
   LEFT JOIN public.native_group_process_versions_v1 v ON
    (v.group_id,v.group_incarnation,v.process_id,v.version)=(h.group_id,h.group_incarnation,h.process_id,h.content_version)
   WHERE g.id=selected_group;
  incarnation:=(group_value->>'group_incarnation')::uuid;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  FOREACH required IN ARRAY ARRAY['group','topology','birth_request','birth_receipt','birth_designation_receipt','source'] LOOP
   IF jsonb_typeof(group_value->required) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  END LOOP;
  IF group_value#>>'{birth_designation_receipt,kind}' IS DISTINCT FROM 'DESIGNATE'
   OR group_value#>>'{birth_designation_receipt,system_identifier}' IS DISTINCT FROM (SELECT system_identifier::text FROM pg_control_system())
   OR group_value#>>'{birth_designation_receipt,database_name}' IS DISTINCT FROM current_database()::text
   OR group_value#>>'{birth_designation_receipt,database_oid}' IS DISTINCT FROM (SELECT oid::bigint::text FROM pg_database WHERE datname=current_database()) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  SELECT q.* INTO STRICT birth_request FROM public.company_enrollment_requests q WHERE
   (q.account_id,q.command_id)=((group_value#>>'{group,origin_account_id}')::uuid,(group_value#>>'{group,origin_command_id}')::uuid);
  SELECT r.* INTO STRICT birth_receipt FROM public.company_enrollment_receipts r WHERE
   (r.account_id,r.command_id)=(birth_request.account_id,birth_request.command_id);
  SELECT b.* INTO STRICT birth_effect FROM public.company_enrollment_effect_bindings b WHERE
   (b.account_id,b.command_id)=(birth_request.account_id,birth_request.command_id);
  IF to_jsonb(birth_request) IS DISTINCT FROM group_value->'birth_request'
   OR to_jsonb(birth_receipt) IS DISTINCT FROM group_value->'birth_receipt'
   OR birth_request.state IS DISTINCT FROM 'COMMITTED'
   OR (birth_request.codec_version,birth_request.input_digest,birth_request.designation_receipt_id,
       birth_request.committed_receipt_id,birth_request.terminal_at)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.input_digest,birth_receipt.designation_receipt_id,
       birth_receipt.receipt_id,birth_receipt.committed_at)
   OR (birth_effect.codec_version,birth_effect.input_digest,birth_effect.designation_receipt_id,
       birth_effect.org_id,birth_effect.group_id,birth_effect.receipt_id,birth_effect.administrative_account_id,
       birth_effect.catalog_version,birth_effect.manifest_digest,birth_effect.session_id,birth_effect.started_at)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.input_digest,birth_receipt.designation_receipt_id,
       birth_receipt.org_id,birth_receipt.group_id,birth_receipt.receipt_id,birth_receipt.administrative_account_id,
       birth_receipt.catalog_version,birth_receipt.manifest_digest,birth_receipt.session_id,birth_receipt.committed_at)
   OR birth_receipt.codec_version<>1 OR birth_receipt.root_revision<>1
   OR birth_receipt.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
   OR birth_receipt.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex')
   OR birth_receipt.committed_at<birth_request.created_at OR birth_receipt.committed_at>=birth_request.expires_at
   OR public.native_group_process_micros_v1(birth_receipt.committed_at) IS DISTINCT FROM
      public.native_group_process_micros_v1((group_value#>>'{group,created_at}')::timestamptz)
   OR group_value#>>'{group,created_at}' IS DISTINCT FROM group_value#>>'{group,updated_at}'
   OR birth_receipt.account_id::text IS DISTINCT FROM group_value#>>'{birth_designation_receipt,account_id}'
   OR group_value#>>'{birth_designation_receipt,revision}' IS DISTINCT FROM '1'
   OR group_value#>>'{birth_designation_receipt,expected_revision}' IS DISTINCT FROM '0'
   OR group_value#>>'{birth_designation_receipt,expected_security_generation}' IS NULL
   OR (group_value#>>'{birth_designation_receipt,expected_security_generation}')::bigint<1
   OR group_value#>'{birth_designation_receipt,reason}' IS DISTINCT FROM 'null'::jsonb
   OR (SELECT count(*) FROM public.company_enrollment_request_events e
      WHERE (e.account_id,e.command_id)=(birth_request.account_id,birth_request.command_id))<>2
   OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE (e.account_id,e.command_id,e.event_revision)=(birth_request.account_id,birth_request.command_id,1)
      AND e.from_state IS NULL AND e.to_state='PENDING' AND e.reason_code='PREPARED'
      AND e.occurred_at=birth_request.created_at AND e.actor_account_id=birth_request.account_id)
   OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE (e.account_id,e.command_id,e.event_revision)=(birth_request.account_id,birth_request.command_id,2)
      AND e.from_state='PENDING' AND e.to_state='COMMITTED' AND e.reason_code='COMMITTED'
      AND e.occurred_at=birth_receipt.committed_at AND e.actor_account_id=birth_receipt.account_id
      AND e.session_id=birth_receipt.session_id) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  -- Retention may remove terminal payload; decode only retained historical bytes.
  IF birth_request.input_bytes IS NOT NULL THEN
   SELECT * INTO STRICT decoded_birth FROM public.company_enrollment_decode_input_v1(birth_request.input_bytes);
   IF (decoded_birth.codec_version,decoded_birth.account_id,decoded_birth.command_id,decoded_birth.input_digest,
       decoded_birth.administrative_account_id,decoded_birth.company_name)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.account_id,birth_receipt.command_id,birth_receipt.input_digest,
       birth_receipt.administrative_account_id,group_value#>>'{group,name}')
    OR decoded_birth.group_id IS NOT NULL THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END IF;
  additional:=octet_length(convert_to(group_value::text,'UTF8'))+CASE WHEN first_group THEN 0 ELSE 2 END;
  IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  budget:=budget-additional;
  history_value:='[]'::jsonb; first_history:=true;
  head_revision:=(group_value#>>'{head,head_revision}')::bigint;
  FOR item IN SELECT h.*,i.operation,i.input_bytes FROM public.native_group_process_head_revisions_v1 h
   LEFT JOIN public.native_group_process_inputs_v1 i ON (i.actor_account_id,i.command_id)=(h.last_actor_account_id,h.last_command_id)
   WHERE h.group_id=selected_group AND h.group_incarnation=incarnation AND h.head_revision<=head_revision ORDER BY h.head_revision
  LOOP
   IF item.operation IS NULL OR item.input_bytes IS NULL THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   candidate:=jsonb_build_object('head',to_jsonb(item)-ARRAY['operation','input_bytes'],
    'reason',CASE WHEN item.operation=6 THEN public.native_group_process_decode_v1(item.input_bytes)->>'reason' END);
   additional:=octet_length(convert_to(candidate::text,'UTF8'))+CASE WHEN first_history THEN 0 ELSE 2 END;
   IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   budget:=budget-additional;
   history_value:=history_value||jsonb_build_array(candidate); first_history:=false;
  END LOOP;
  -- Full owner closure work starts only after the complete history fits.
  PERFORM public.native_group_process_assert_current_v1(selected_group,incarnation);
  group_value:=jsonb_set(group_value,'{history}',history_value);
  groups_value:=groups_value||jsonb_build_array(group_value); first_group:=false;
  candidates:=candidates||jsonb_build_array(jsonb_build_object('group_id',selected_group,'group_incarnation',incarnation));
 END LOOP;
 captured:=jsonb_set(captured,'{groups}',groups_value);
 snapshot_bytes:=convert_to(captured::text,'UTF8');
 IF octet_length(snapshot_bytes) NOT BETWEEN 1 AND 1048576 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;

 -- Recheck the actual row shapes too: adding a column cannot silently broaden
 -- this ephemeral internal projection. The same frozen parser governs both.
 document:=captured;
  IF jsonb_typeof(document->'account') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF ARRAY(SELECT key FROM jsonb_object_keys(document->'account') key ORDER BY key COLLATE "C")
   IS DISTINCT FROM ARRAY['actor_account_id','consent','enrollment','family','root','security','session_id','terms_head']
   OR document#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR document#>>'{account,root,id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,security,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,family,user_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,kind}' IS DISTINCT FROM 'ENROLLED'
   OR jsonb_typeof(document#>'{account,consent}') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(document#>'{account,consent}') NOT BETWEEN 1 AND 8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  rows_to_validate:=jsonb_build_array(jsonb_build_array('accounts',document#>'{account,root}'),
   jsonb_build_array('account_security',document#>'{account,security}'),
   jsonb_build_array('family',document#>'{account,family}'),
   jsonb_build_array('account_security_events',document#>'{account,enrollment}'),
   jsonb_build_array('terms_head',document#>'{account,terms_head}'));
  required:=NULL;
  FOR entry IN SELECT value FROM jsonb_array_elements(document#>'{account,consent}') LOOP
   IF jsonb_typeof(entry->'terms_kind') IS DISTINCT FROM 'string'
    OR entry->>'terms_kind' !~ '^[a-z][a-z0-9_.-]{0,63}$'
    OR (required IS NOT NULL AND (entry->>'terms_kind') COLLATE "C"<=required COLLATE "C") THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   required:=entry->>'terms_kind'; rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('consent',entry));
  END LOOP;
  IF document->'designation' IS DISTINCT FROM 'null'::jsonb THEN
   IF jsonb_typeof(document->'designation') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   IF ARRAY(SELECT key FROM jsonb_object_keys(document->'designation') key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','receipt'] THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('deployment_operator_head',document#>'{designation,head}'),
    jsonb_build_array('deployment_operator_receipts',document#>'{designation,receipt}'));
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(document->'groups') LOOP
   IF ARRAY(SELECT key FROM jsonb_object_keys(candidate) key ORDER BY key COLLATE "C")
    IS DISTINCT FROM ARRAY['birth_designation_receipt','birth_receipt','birth_request','group','group_id','group_incarnation','head','history','policy','source','topology','version']
    OR candidate#>>'{group,id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
    OR candidate#>>'{birth_receipt,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR jsonb_typeof(candidate->'history') IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   IF (candidate->'policy'='null'::jsonb) IS DISTINCT FROM (candidate->'head'='null'::jsonb)
    OR (candidate->'head'='null'::jsonb) IS DISTINCT FROM (candidate->'version'='null'::jsonb)
    OR ((candidate->'head'='null'::jsonb) IS DISTINCT FROM (jsonb_array_length(candidate->'history')=0)) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('groups',candidate->'group'),jsonb_build_array('group_authority_heads',candidate->'topology'),
    jsonb_build_array('company_enrollment_requests',candidate->'birth_request'),
    jsonb_build_array('company_enrollment_receipts',candidate->'birth_receipt'),
    jsonb_build_array('deployment_operator_receipts',candidate->'birth_designation_receipt'),
    jsonb_build_array('source',candidate->'source'));
   IF candidate->'policy' IS DISTINCT FROM 'null'::jsonb THEN
    rows_to_validate:=rows_to_validate||jsonb_build_array(
     jsonb_build_array('native_group_identity_policy_heads_v1',candidate->'policy'),
     jsonb_build_array('native_group_process_heads_v1',candidate->'head'),
     jsonb_build_array('native_group_process_versions_v1',candidate->'version'));
   END IF;
   head_revision:=0;
   FOR entry IN SELECT value FROM jsonb_array_elements(candidate->'history') LOOP
    IF jsonb_typeof(entry) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    IF ARRAY(SELECT key FROM jsonb_object_keys(entry) key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','reason']
     OR jsonb_typeof(entry->'reason') NOT IN ('string','null')
     OR entry#>>'{head,group_id}' IS DISTINCT FROM candidate->>'group_id'
     OR entry#>>'{head,group_incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
     OR entry#>>'{head,process_id}' IS DISTINCT FROM candidate#>>'{head,process_id}' THEN
     RAISE EXCEPTION 'native_group_process.material_unavailable';
    END IF;
    revision_value:=entry#>>'{head,head_revision}';
    IF revision_value IS NULL OR revision_value !~ '^[1-9][0-9]{0,18}$' OR revision_value::numeric>9223372036854775807
     OR revision_value::numeric<>head_revision::numeric+1 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    head_revision:=revision_value::bigint;
    rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('native_group_process_head_revisions_v1',entry->'head'));
   END LOOP;
   IF head_revision>0 AND (candidate->'history'->-1->'head') IS DISTINCT FROM candidate->'head' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END LOOP;
  FOR checked_row IN SELECT value FROM jsonb_array_elements(rows_to_validate) LOOP
   IF jsonb_typeof(checked_row->1) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   expected_keys:=ARRAY(SELECT key FROM jsonb_object_keys(row_shapes->(checked_row->>0)) key ORDER BY key COLLATE "C");
   IF cardinality(expected_keys)=0 OR ARRAY(SELECT key FROM jsonb_object_keys(checked_row->1) key ORDER BY key COLLATE "C") IS DISTINCT FROM expected_keys THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   FOR row_key,row_value IN SELECT key,value FROM jsonb_each(checked_row->1) LOOP
    field_shape:=row_shapes->(checked_row->>0)->row_key;
    IF row_value='null'::jsonb THEN
     IF field_shape->1 IS DISTINCT FROM 'true'::jsonb THEN
      RAISE EXCEPTION 'native_group_process.material_unavailable';
     END IF;
     CONTINUE;
    END IF;
    value_text:=row_value#>>'{}';
    CASE field_shape->>0
     WHEN 'uuid' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string'
       OR value_text !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
       OR value_text='00000000-0000-0000-0000-000000000000' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'integer' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'number' OR value_text !~ '^(0|[1-9][0-9]{0,18})$' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
      IF value_text::numeric>9223372036854775807 OR (value_text='0' AND row_key<>'expected_revision') THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'timestamp' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
      parsed_time:=value_text::timestamptz;
      IF NOT isfinite(parsed_time) OR to_jsonb(parsed_time) IS DISTINCT FROM row_value THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'bytea' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' OR left(value_text,2) IS DISTINCT FROM '\x'
       OR substring(value_text FROM 3) !~ '^([0-9a-f]{2})*$'
       OR (right(row_key,6)='digest' AND length(value_text)<>66)
       OR (right(row_key,6)='sha256' AND length(value_text)<>66) THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     ELSE
      IF jsonb_typeof(row_value) IS DISTINCT FROM field_shape->>0 THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
    END CASE;
   END LOOP;
  END LOOP;
 SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO after_groups
  FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
 IF cardinality(after_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 IF p_original IS NOT NULL AND (after_groups IS DISTINCT FROM current_groups OR snapshot_bytes IS DISTINCT FROM p_original) THEN
  RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
   'session_id',p_family,'status','CONFLICT','candidates','[]'::jsonb,'snapshot',NULL);
 END IF;
 RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
  'session_id',p_family,'status','MATCH','candidates',candidates,'snapshot',snapshot_bytes);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;
