-- Uninstalled exact Group ABI/ACL roster. No new owner role or membership.
-- Complete custody still verifies effective privileges/defaults and all source.
DO $acl$
DECLARE routine record; grantee_name text; relation text; column_name text;
BEGIN
 FOR routine IN SELECT * FROM (VALUES
  ('public.native_group_process_uuid_v1(bytea,integer)',NULL::text),
  ('public.native_group_process_i64_v1(bytea,integer)',NULL::text),
  ('public.native_group_process_text_v1(text,integer)',NULL::text),
  ('public.native_group_process_micros_v1(timestamptz)',NULL::text),
  ('public.native_group_process_decode_v1(bytea)',NULL::text),
  ('public.native_group_process_policy_ref_v1(smallint,bigint,bytea)',NULL::text),
  ('public.native_group_process_head_ref_v1(uuid,bigint,bigint,bytea,bytea,text,timestamptz)',NULL::text),
  ('public.native_group_process_action_roster_v1()',NULL::text),
  ('public.native_group_process_registration_bytes_v1(uuid,uuid,text,bytea,bytea,bytea,jsonb)',NULL::text),
  ('public.native_group_process_policy_head_bytes_v1(public.native_group_identity_policy_heads_v1)',NULL::text),
  ('public.native_group_process_version_bytes_v1(public.native_group_process_versions_v1)',NULL::text),
  ('public.native_group_process_head_bytes_v1(public.native_group_process_head_revisions_v1)',NULL::text),
  ('public.native_group_process_result_bytes_v2(public.native_group_process_results_v1)',NULL::text),
  ('public.native_group_process_registered_actions_v1()',NULL::text),
  ('public.native_group_process_source_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_group_guard_v1(uuid,boolean)',NULL::text),
  ('public.native_group_process_command_guard_v1(uuid,uuid,boolean)',NULL::text),
  ('public.native_group_process_topology_fence_v1()',NULL::text),
  ('public.native_group_process_account_material_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_original_material_v1(uuid,uuid,uuid,uuid)',NULL::text),
  ('public.identity_native_group_process_material_v1(uuid,uuid,uuid,uuid,uuid,smallint,bytea)','console_rt'),
  ('public.identity_native_group_process_incarnation_selector_v1(uuid,uuid,uuid,uuid)','console_rt'),
  ('public.identity_native_group_process_navigation_candidates_v1(uuid,uuid,bytea)','console_rt'),
  ('public.native_group_process_accept_snapshot_v1(public.native_group_process_inputs_v1)',NULL::text),
  ('public.native_group_process_complete_snapshot_v1(public.native_group_process_results_v1)',NULL::text),
  ('public.native_group_process_assert_input_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_assert_result_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_assert_current_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_audit_guard_v1()',NULL::text),
  ('public.native_group_process_audit_truncate_guard_v1()',NULL::text),
  ('public.native_group_process_current_context_v1(uuid,uuid,uuid,uuid,boolean)',NULL::text),
  ('public.native_group_process_classify_v1(public.native_group_process_effects_v1,jsonb)',NULL::text),
  ('public.native_group_process_validate_frame_v1(public.native_group_process_effects_v1)',NULL::text),
  ('public.native_group_process_input_guard_v1()',NULL::text),
  ('public.native_group_process_effect_guard_v1()',NULL::text),
  ('public.native_group_process_participant_guard_v1()',NULL::text),
  ('public.native_group_process_deferred_guard_v1()',NULL::text),
  ('public.native_group_process_immutable_statement_v1()',NULL::text),
  ('public.native_group_process_head_statement_v1()',NULL::text),
  ('public.identity_native_group_process_prepare_v1(uuid,uuid,uuid,bytea)','console_rt'),
  ('public.identity_native_group_process_execute_v1(uuid,uuid,uuid)','console_rt')
 ) required(signature,executor_name) LOOP
  EXECUTE format('ALTER FUNCTION %s OWNER TO console_account_owner',routine.signature);
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',routine.signature);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p
   CROSS JOIN LATERAL aclexplode(p.proacl) a JOIN pg_roles r ON r.oid=a.grantee
   WHERE p.oid=routine.signature::regprocedure AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',routine.signature,grantee_name); END LOOP;
  IF routine.executor_name IS NOT NULL THEN
   EXECUTE format('GRANT EXECUTE ON FUNCTION %s TO %I',routine.signature,routine.executor_name);
  END IF;
 END LOOP;
 FOREACH relation IN ARRAY ARRAY['native_group_process_inputs_v1','native_group_process_effects_v1','native_group_process_results_v1',
  'native_group_identity_policy_heads_v1','native_group_process_versions_v1','native_group_process_head_revisions_v1','native_group_process_heads_v1'] LOOP
  EXECUTE format('REVOKE ALL ON TABLE public.%I FROM PUBLIC',relation);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c
   CROSS JOIN LATERAL aclexplode(c.relacl) a JOIN pg_roles r ON r.oid=a.grantee
   WHERE c.oid=format('public.%I',relation)::regclass AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL ON TABLE public.%I FROM %I',relation,grantee_name); END LOOP;
  FOR column_name IN SELECT a.attname FROM pg_attribute a
   WHERE a.attrelid=format('public.%I',relation)::regclass AND a.attnum>0 AND NOT a.attisdropped
  LOOP EXECUTE format('REVOKE ALL (%I) ON TABLE public.%I FROM PUBLIC',column_name,relation); END LOOP;
  FOR column_name,grantee_name IN SELECT DISTINCT a.attname,r.rolname FROM pg_attribute a
   CROSS JOIN LATERAL aclexplode(a.attacl) grant_entry JOIN pg_roles r ON r.oid=grant_entry.grantee
   WHERE a.attrelid=format('public.%I',relation)::regclass AND a.attnum>0 AND NOT a.attisdropped
    AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL (%I) ON TABLE public.%I FROM %I',column_name,relation,grantee_name); END LOOP;
 END LOOP;
END
$acl$;
