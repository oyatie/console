-- Generated atomic native Company policy dual-codec custody upgrade.
-- Requires compatible readers before this separately operated database change.
DO $native_policy_v2_custody$
DECLARE observed text; rights_valid boolean; predecessor text; prior_rights_valid boolean;
 expected_final text; relation_name text;
BEGIN
 IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
  OR session_user IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
   'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
   'console_platform_force_cmd','console_account_owner','console_terms_owner','console_credential_owner',
   'console_durability_observer') THEN RAISE EXCEPTION 'native_company_policy_v2.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
  RAISE EXCEPTION 'native_company_policy_v2.unsupported_isolation'; END IF;
 PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
 PERFORM pg_catalog.set_config('lock_timeout','1s',true);
 FOR relation_name IN SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname IN ('account_context_candidates','account_security','account_security_events','account_terms_acceptances','account_terms_head','account_terms_release_receipts','accounts','audit_events','auth_bootstrap_credentials','auth_device_login_handoffs','auth_refresh_token_families','auth_refresh_tokens','auth_webauthn_ceremonies','auth_webauthn_ceremony_bindings','auth_webauthn_credentials','cedar_policy_catalog_entries','company_actors','company_authority_heads','company_enrollment_effect_bindings','company_enrollment_receipts','company_enrollment_request_events','company_enrollment_requests','deployment_operator_head','deployment_operator_receipts','group_authority_heads','group_membership_revisions','group_memberships','group_role_grants','groups','native_company_action_refs','native_company_catalog_installs','native_company_object_refs','native_company_policy_inputs_v1','native_company_policy_receipts_v1','native_company_property_refs','ont_action_types','ont_analytics','ont_builtin_catalog_allowlist','ont_builtin_catalog_installs','ont_link_types','ont_object_policies','ont_object_type_key_revisions','ont_object_types','ont_property_defs','organizations','platform_force_removal_effect_bindings','platform_force_removal_receipts','platform_legacy_catalog_effect_bindings','platform_legacy_membership_effect_bindings','platform_legacy_topology_effect_bindings','platform_legacy_topology_receipts','platform_legacy_user_birth_witnesses','policy_assignment_revisions','policy_capability_clause_fields','policy_capability_clauses','policy_role_conditions','policy_role_permissions','policy_role_revisions','policy_roles','user_role_assignments','users') ORDER BY c.relname COLLATE "C"
 LOOP EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name); END LOOP;
 SELECT snapshot_sha256,native_policy_startup_rights_valid INTO observed,rights_valid FROM (
-- READ-ONLY disposable capture. Freeze only after independent declared-source comparison.
WITH wanted(name) AS (VALUES
 ('accounts'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('company_actors'),
 ('account_context_candidates'),
 ('deployment_operator_receipts'),
 ('deployment_operator_head'),
 ('audit_events'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('cedar_policy_catalog_entries'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_property_refs'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('organizations'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND (a.attnum>0 OR (a.attnum<0 AND a.attacl IS NOT NULL)) AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), deployment_observer_role AS (
 SELECT * FROM pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_proc p
 CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
   OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a
    JOIN deployment_observer_role r ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('schema',n.nspname,'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE (n.nspname,p.proname) IN (VALUES ('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app')
), related_fks AS (
 SELECT k.* FROM pg_constraint k WHERE k.contype='f'
   AND (k.conrelid IN (SELECT oid FROM relations) OR k.confrelid IN (SELECT oid FROM relations))
), foreign_key_records AS (
 SELECT jsonb_build_object('schema',ns.nspname,'name',k.conname,
   'source',k.conrelid::regclass::text,'target',k.confrelid::regclass::text,
   'namespace',kn.nspname,'type',k.contypid::regtype::text,
   'parent',CASE WHEN k.conparentid=0 THEN NULL ELSE parent.conname END,
   'validated',k.convalidated,'enforced',k.conenforced,'period',k.conperiod,
   'local',k.conislocal,'inherited',k.coninhcount,'noinherit',k.connoinherit,
   'deferrable',k.condeferrable,'deferred',k.condeferred,
   'keys',k.conkey,'foreign_keys',k.confkey,'update',k.confupdtype,
   'delete',k.confdeltype,'match',k.confmatchtype,'delete_columns',k.confdelsetcols,
   'binary_expression',pg_get_expr(k.conbin,k.conrelid),
   'primary_foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conpfeqop) WITH ORDINALITY a(op,ordinal)),
   'primary_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conppeqop) WITH ORDINALITY a(op,ordinal)),
   'foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conffeqop) WITH ORDINALITY a(op,ordinal)),
   'exclusion_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conexclop) WITH ORDINALITY a(op,ordinal)),
   'supporting_index',k.conindid::regclass::text,
   'supporting_index_definition',pg_get_indexdef(k.conindid),
   'index_flags',jsonb_build_array(i.indrelid=k.confrelid,i.indisunique,i.indisvalid,i.indisready,i.indislive,
       i.indimmediate,i.indisexclusion,i.indnkeyatts,i.indnatts,i.indkey::text,
       pg_get_expr(i.indexprs,i.indrelid),pg_get_expr(i.indpred,i.indrelid)),
   'ri',(SELECT jsonb_agg(jsonb_build_object(
       'on',t.tgrelid::regclass::text,'other',t.tgconstrrelid::regclass::text,
       'index',t.tgconstrindid::regclass::text,'function',t.tgfoid::regprocedure::text,
       'name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$',
       'internal',t.tgisinternal,'enabled',t.tgenabled,'type',t.tgtype,
       'arguments',t.tgnargs,'args',encode(t.tgargs,'hex'),'attributes',t.tgattr::text,
       'deferrable',t.tgdeferrable,'deferred',t.tginitdeferred,
       'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,'new_table',t.tgnewtable,
       'condition',pg_get_expr(t.tgqual,t.tgrelid))
       ORDER BY t.tgrelid::regclass::text,t.tgfoid::regprocedure::text)
     FROM pg_trigger t WHERE t.tgconstraint=k.oid)) AS record
 FROM related_fks k JOIN pg_class c ON c.oid=k.conrelid
 JOIN pg_namespace ns ON ns.oid=c.relnamespace
 JOIN pg_namespace kn ON kn.oid=k.connamespace
 LEFT JOIN pg_constraint parent ON parent.oid=k.conparentid
 LEFT JOIN pg_index i ON i.indexrelid=k.conindid
), legacy_root_boundary AS (
-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT jsonb_build_object('root_profile',(SELECT valid FROM root_profile),'projection',(SELECT valid FROM projection),'deactivation_guard',(SELECT valid FROM deactivation_guard),'deactivation_users_acl',(SELECT valid FROM deactivation_users_acl),'root_insert_acl',(SELECT valid FROM root_insert_acl),'receipt_guard',(SELECT valid FROM receipt_guard),'terms_current',(SELECT valid FROM terms_current),'auth7_root_profile',(SELECT valid FROM auth7_root_profile)) AS boundary
), deployment_startup AS (
 SELECT role.* FROM (VALUES ('console_auth_startup')) required(name)
 LEFT JOIN pg_roles role ON role.rolname=required.name
), deployment_rights_relations AS (
 SELECT name,oid,relkind FROM relations
 UNION ALL
 SELECT required.name,c.oid,c.relkind
 FROM (VALUES ('users'),('organizations'),('groups'),('employees'),('persons'),
   ('person_revisions'),('employee_person_bindings'),('group_memberships'),('group_role_grants')) required(name)
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
), deployment_startup_database_settings AS (
 SELECT setting FROM pg_db_role_setting d
 JOIN deployment_startup startup ON startup.oid=d.setrole
 CROSS JOIN LATERAL unnest(d.setconfig) setting
 WHERE d.setdatabase<>0
), deployment_startup_database_overrides AS (
 SELECT wanted.key,count(actual.setting) AS override_count
 FROM (VALUES ('statement_timeout'),('idle_in_transaction_session_timeout'),('transaction_timeout')) wanted(key)
 LEFT JOIN deployment_startup_database_settings actual ON split_part(actual.setting,'=',1)=wanted.key
 GROUP BY wanted.key
), deployment_table_rights AS (
 SELECT r.name, privilege.name AS privilege,
   has_table_privilege(s.oid,r.oid,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) privilege(name)
), deployment_column_rights AS (
 SELECT r.name, a.attname, privilege.name AS privilege,
   has_column_privilege(s.oid,r.oid,a.attnum,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 JOIN pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) privilege(name)
), deployment_function_rights AS (
 SELECT p.oid,n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) AS identity_arguments,
   has_function_privilege(s.oid,p.oid,'EXECUTE') AS allowed,
   has_function_privilege(s.oid,p.oid,'EXECUTE WITH GRANT OPTION') AS grantable,
   p.oid IN (
     to_regprocedure('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)'),
     to_regprocedure('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)')) AS expected_execute
 FROM deployment_startup s CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE p.proowner IN (SELECT oid FROM owner_roles)
    OR (n.nspname='public' AND p.prosecdef)
), deployment_mandatory_functions AS (
 SELECT required.identity,p.oid IS NOT NULL AS present,
   p.prokind='f' AND p.prosecdef AND p.prorettype NOT IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) AS callable_definer,
   checked.oid IS NOT NULL AS included, checked.allowed, checked.grantable
 FROM (VALUES
   ('public.account_session_shared_material_v1(uuid,uuid)'),
   ('public.auth_account_session_shared_material_v1(uuid,uuid)'),
   ('public.account_company_setup_eligibility_v1(uuid)'),
   ('public.auth_legacy_platform_source_material_v1(uuid,uuid)'),
   ('public.account_context_presence_v1(uuid)'),
   ('public.account_login_consent_v1(uuid)'),
   ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])'),
   ('public.account_registration_begin_v1()'),
   ('public.account_security_lock_exclusive_v1(uuid)'),
   ('public.account_security_lock_shared_v1(uuid)'),
   ('public.account_session_logout_v1(uuid,uuid,bigint,interval)'),
   ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.account_terms_current_v1()'),
   ('public.account_terms_registration_head_v1()'),
   ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)'),
   ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)'),
   ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)'),
   ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_cold_start_admin_v1()'),
   ('public.auth_legacy_company_lock_v1(uuid)'),
   ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)'),
   ('public.auth_legacy_group_passkey_flag_v1(uuid)'),
   ('public.auth_legacy_purge_company_v1(uuid)'),
   ('public.auth_legacy_purge_subjects_v1(uuid)'),
   ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)'),
   ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkeys_v1(uuid,uuid)'),
   ('public.auth_legacy_session_context_v1(uuid,uuid)'),
   ('public.auth_legacy_user_active_v1(uuid,uuid)'),
   ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)'),
   ('public.group_member_org_ids(uuid,uuid)'),
   ('public.group_role_grants_for_user(uuid)'),
   ('public.platform_assign_org_to_group(uuid,uuid)'),
   ('public.platform_attach_group_of_one(uuid)'),
   ('public.platform_attach_membership(uuid,uuid)'),
   ('public.platform_create_group(text,text)'),
   ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)'),
   ('public.platform_create_organization(text,text)'),
   ('public.platform_force_remove_direct_org_children(uuid)'),
   ('public.platform_force_remove_organization(uuid)'),
   ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)'),
   ('public.platform_get_group(uuid)'),
   ('public.platform_get_organization(uuid)'),
   ('public.platform_list_group_accounts(uuid)'),
   ('public.platform_list_groups()'),
   ('public.platform_list_organizations()'),
   ('public.platform_mint_group_row(uuid,text,text)'),
   ('public.platform_mint_missing_group_of_one(uuid)'),
   ('public.platform_remove_org_from_group(uuid,uuid)'),
   ('public.platform_remove_organization(uuid)'),
   ('public.platform_resolve_bootstrap_org(bytea)'),
   ('public.platform_resolve_credential_org(text)'),
   ('public.platform_resolve_token_org(bytea)'),
   ('public.platform_revoke_group_role(uuid,uuid,text)'),
   ('public.platform_set_organization_status(uuid,text)'),
   ('public.platform_update_group(uuid,text,text,text)')) required(identity)
 LEFT JOIN pg_proc p ON p.oid=to_regprocedure(required.identity)
 LEFT JOIN deployment_function_rights checked ON checked.oid=p.oid
), deployment_builtin AS (
 SELECT p.oid, p.proowner, owner.rolsuper AS owner_superuser,
   jsonb_build_object(
     'identity',p.oid::regprocedure::text,'owner',jsonb_build_array('builtin_owner'),
     'owner_superuser',owner.rolsuper,'language',language.lanname,
     'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,
     'returns_set',p.proretset,'leakproof',p.proleakproof,
     'volatility',p.provolatile,'parallel',p.proparallel,
     'result',pg_get_function_result(p.oid),'config',p.proconfig,
     'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
     'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
         CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       ORDER BY CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb),
     'account_owner_execute',has_function_privilege((SELECT oid FROM pg_roles WHERE rolname='console_account_owner'),p.oid,'EXECUTE'),
     'startup_execute',has_function_privilege((SELECT oid FROM deployment_startup),p.oid,'EXECUTE')
   ) AS record
 FROM pg_proc p JOIN pg_roles owner ON owner.oid=p.proowner
 JOIN pg_language language ON language.oid=p.prolang
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_memberships AS (
 SELECT jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
   CASE WHEN m.roleid=(SELECT oid FROM pg_roles WHERE rolname='pg_read_all_stats')
         AND m.member=o.oid AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option
         AND grantor.rolsuper AND grantor.rolcanlogin
         AND grantor.rolname NOT IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer')
     THEN jsonb_build_array('trusted_maintenance_grantor')
     ELSE jsonb_build_array('role',pg_get_userbyid(m.grantor)) END,
   m.admin_option,m.inherit_option,m.set_option) AS record
 FROM deployment_observer_active o JOIN pg_auth_members m ON m.roleid=o.oid OR m.member=o.oid
 JOIN pg_roles grantor ON grantor.oid=m.grantor
), deployment_observer_settings AS (
 SELECT CASE WHEN d.setdatabase=0 THEN 'global'
          WHEN d.setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) THEN 'current'
          ELSE 'other' END AS scope,
        d.setconfig IS NULL AS config_is_null,
        (SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
           FROM unnest(d.setconfig) setting) AS config_hashes
 FROM deployment_observer_active o JOIN pg_db_role_setting d ON d.setrole=o.oid
), deployment_observer_external_routine_grants AS (
 SELECT jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
    CASE WHEN p.oid=to_regprocedure('pg_catalog.pg_control_system()')
          AND a.grantor=p.proowner AND owner.rolsuper
      THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
    CASE WHEN a.grantee=0 THEN jsonb_build_array('public') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
    a.privilege_type,a.is_grantable) AS record
 FROM deployment_observer_active o CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_roles owner ON owner.oid=p.proowner
 CROSS JOIN LATERAL aclexplode(p.proacl) a
 WHERE p.proowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)
), deployment_observer_boundary AS (
 SELECT CASE WHEN NOT (SELECT present FROM deployment_observer_local) THEN jsonb_build_object('present',false)
 ELSE jsonb_build_object(
   'present',true,
   'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,rolcanlogin,rolinherit,
       rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconnlimit,rolvaliduntil,
       rolconfig IS NULL,(SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
         FROM unnest(rolconfig) setting)) ORDER BY rolname) FROM deployment_observer_active),
   'memberships',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_memberships),
   'settings',(SELECT jsonb_agg(jsonb_build_array(scope,config_is_null,config_hashes)
       ORDER BY scope,config_is_null,config_hashes::text COLLATE "C") FROM deployment_observer_settings),
   'external_routine_grants',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_external_routine_grants),
   'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind)
       ORDER BY n.nspname,c.relname) FROM deployment_observer_active o JOIN pg_class c ON c.relowner=o.oid
       JOIN pg_namespace n ON n.oid=c.relnamespace),
   'owner_schemas',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o JOIN pg_namespace n ON n.nspowner=o.oid),
   'schema_create',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o CROSS JOIN pg_namespace n
       WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(o.oid,n.oid,'CREATE')),
   'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_default_acl d
       LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace CROSS JOIN LATERAL aclexplode(d.defaclacl) a
       WHERE d.defaclrole=o.oid OR a.grantee=o.oid OR a.grantor=o.oid),
   'external_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
       CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)),
   'external_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_attribute col JOIN pg_class c ON c.oid=col.attrelid
       JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(col.attacl) a
       WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid))
 ) END AS record
), deployment_boundary AS (
 SELECT jsonb_build_object(
   'builtin',(SELECT record FROM deployment_builtin),
   'observer',(SELECT record FROM deployment_observer_boundary),
   'startup_table_rights',(SELECT jsonb_agg(jsonb_build_array(name,privilege,allowed) ORDER BY name,privilege) FROM deployment_table_rights),
   'startup_column_rights',(SELECT jsonb_agg(jsonb_build_array(name,attname,privilege,allowed) ORDER BY name,attname,privilege) FROM deployment_column_rights),
   'startup_function_rights',(SELECT jsonb_agg(jsonb_build_array(nspname,proname,identity_arguments,allowed,grantable,expected_execute) ORDER BY nspname,proname,identity_arguments) FROM deployment_function_rights),
   'startup_mandatory_function_rights',(SELECT jsonb_agg(jsonb_build_array(identity,present,callable_definer,included,allowed,grantable) ORDER BY identity) FROM deployment_mandatory_functions),
   'startup_managed_database_overrides',(SELECT jsonb_agg(jsonb_build_array(key,override_count) ORDER BY key) FROM deployment_startup_database_overrides),
   'startup_final_rights_valid',
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=18 AND count(oid)=18 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=27 AND count(oid)=27 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=216 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=27 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin)
 ) AS record
), company_startup_rights AS (
 SELECT
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=61 AND count(oid)=61 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=70 AND count(oid)=70 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=560 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=65 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS (
 SELECT jsonb_build_object(
  'required_schemas',(SELECT jsonb_agg(jsonb_build_object(
    'name',required.name,'present',n.oid IS NOT NULL,
    'owner',CASE WHEN required.name='pg_catalog' AND n.nspowner=(SELECT proowner FROM deployment_builtin)
       AND owner.rolsuper THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',owner.rolname) END,
    'owner_superuser',owner.rolsuper,'acl_is_null',n.nspacl IS NULL,
    'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
       CASE WHEN required.name='pg_catalog' AND a.grantor=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
       CASE WHEN a.grantee=0 THEN jsonb_build_array('public')
         WHEN required.name='pg_catalog' AND a.grantee=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
       a.privilege_type,a.is_grantable) ORDER BY
       CASE WHEN a.grantor=n.nspowner THEN '' ELSE pg_get_userbyid(a.grantor) END,
       CASE WHEN a.grantee=n.nspowner THEN '' WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable)
      FROM aclexplode(COALESCE(n.nspacl,acldefault('n',n.nspowner))) a),'[]'::jsonb),
    'effective_rights',(SELECT jsonb_agg(jsonb_build_array(r.rolname,
       has_schema_privilege(r.oid,n.oid,'USAGE'),has_schema_privilege(r.oid,n.oid,'CREATE'),
       has_schema_privilege(r.oid,n.oid,'USAGE WITH GRANT OPTION'),
       has_schema_privilege(r.oid,n.oid,'CREATE WITH GRANT OPTION')) ORDER BY r.rolname)
      FROM (SELECT oid,rolname FROM protected_roles UNION SELECT oid,rolname FROM deployment_observer_active) r)
    ) ORDER BY required.name)
    FROM (VALUES ('public'),('ontology_api'),('ont_policy_api'),('leave_api'),('pg_catalog')) required(name)
    LEFT JOIN pg_namespace n ON n.nspname=required.name LEFT JOIN pg_roles owner ON owner.oid=n.nspowner),
  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),
  'legacy_root_boundary',(SELECT boundary FROM legacy_root_boundary),
  'tables',(SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records),
  'routines',(SELECT jsonb_agg(jsonb_build_object('metadata',record,'extra_valid',extra_valid)
      ORDER BY record->>'schema',record->>'name',record->>'identity_arguments') FROM routine_records),
  'foreign_keys',(SELECT jsonb_agg(record ORDER BY record->>'schema',record->>'source',record->>'name') FROM foreign_key_records),
  'constraint_flags',(SELECT jsonb_agg(jsonb_build_array(k.conrelid::regclass::text,k.conname,
      k.conenforced,k.conperiod,k.contypid=0,k.conparentid=0,k.coninhcount,k.conislocal)
      ORDER BY k.conrelid::regclass::text,k.conname)
    FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)),
  'trigger_links',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM (
    SELECT jsonb_build_array(t.tgrelid::regclass::text,
      CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgfoid::regprocedure::text,
      t.tgparentid=0,t.tgattr::text,t.tgoldtable,t.tgnewtable,
      k.conname,CASE WHEN k.conrelid IS NULL THEN NULL ELSE k.conrelid::regclass::text END,
      CASE WHEN t.tgconstrindid=0 THEN NULL ELSE t.tgconstrindid::regclass::text END) AS record
    FROM pg_trigger t LEFT JOIN pg_constraint k ON k.oid=t.tgconstraint
    WHERE t.tgrelid IN (SELECT oid FROM relations)) triggers),
  'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,
      CASE WHEN rolname='console_auth_rt' THEN NULL ELSE rolcanlogin END,
      rolinherit,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconfig) ORDER BY rolname) FROM protected_roles),
  'memberships',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
      pg_get_userbyid(m.grantor),m.admin_option,m.inherit_option,m.set_option)
      ORDER BY pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),pg_get_userbyid(m.grantor))
    FROM pg_auth_members m WHERE m.roleid IN (SELECT oid FROM protected_roles) OR m.member IN (SELECT oid FROM protected_roles)),
  'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner))
      ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
    WHERE c.relowner IN (SELECT oid FROM owner_roles) AND c.relkind IN ('r','p','v','m','S','f')),
  'owner_schemas',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname)
    FROM pg_namespace n WHERE n.nspowner IN (SELECT oid FROM owner_roles)),
  'schema_create',(SELECT jsonb_agg(jsonb_build_array(r.rolname,n.nspname) ORDER BY r.rolname,n.nspname)
    FROM owner_roles r CROSS JOIN pg_namespace n
    -- The current temporary namespace derives CREATE from database TEMP, not
    -- persistent schema authority. Keep every other namespace/ACL check exact.
    WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(r.oid,n.oid,'CREATE')),
  'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
      pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_default_acl d LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace
    CROSS JOIN LATERAL aclexplode(d.defaclacl) a
    WHERE d.defaclrole IN (SELECT oid FROM protected_roles) OR a.grantee IN (SELECT oid FROM owner_roles)),
  'external_owner_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL)),
  'external_owner_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_attribute col JOIN pg_class c ON c.oid=col.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace
    CROSS JOIN LATERAL aclexplode(col.attacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL))
 ) AS snapshot
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_policy_startup_rights_valid FROM snapshots
) captured;
 IF observed IN ('e94251d48fdee392d7632709b19d80cc04d53ea6742e0f2d6538d68391ab8bf2','f132054a56641dcb0cc5875d6485846df2091e631d0d902a18356d2202343fd9') AND rights_valid IS TRUE THEN
-- Immutable People reference; no grants or business records are created.
DO $people_reference$
BEGIN
 INSERT INTO public.ont_builtin_catalog_allowlist(catalog_version,manifest_digest)
 VALUES('native-people-directory-v1',decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex'))
 ON CONFLICT(catalog_version) DO NOTHING;
 IF (SELECT manifest_digest FROM public.ont_builtin_catalog_allowlist
     WHERE catalog_version='native-people-directory-v1') IS DISTINCT FROM
     decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex') THEN
  RAISE EXCEPTION 'native_company_policy.catalog_reference_mismatch';
 END IF;
END
$people_reference$;

  RETURN;
 END IF;
 IF (SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 AND (n.nspname,p.proname) NOT IN (VALUES ('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1')))
 OR EXISTS(SELECT 1 FROM pg_constraint c WHERE c.conrelid IN
   (to_regclass('public.native_company_policy_inputs_v1'),to_regclass('public.native_company_policy_receipts_v1'),
    to_regclass('public.native_company_catalog_installs'),to_regclass('public.native_company_object_refs'),
    to_regclass('public.native_company_action_refs'),to_regclass('public.native_company_property_refs'))
  AND c.conname IN ('native_company_policy_inputs_v1_codec_shape_v2',
   'native_company_policy_receipts_v1_codec_shape_v2','native_company_catalog_installs_catalog_policy_shape_v2',
   'native_company_object_refs_object_key_v3','native_company_action_refs_action_key_v3',
   'native_company_property_refs_property_key_v3')) AS present) THEN
  RAISE EXCEPTION 'native_company_policy_v2.profile_mismatch'; END IF;
 SELECT snapshot_sha256,native_policy_startup_rights_valid INTO predecessor,prior_rights_valid FROM (
-- READ-ONLY disposable capture. Freeze only after independent declared-source comparison.
WITH wanted(name) AS (VALUES
 ('accounts'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('company_actors'),
 ('account_context_candidates'),
 ('deployment_operator_receipts'),
 ('deployment_operator_head'),
 ('audit_events'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('cedar_policy_catalog_entries'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_property_refs'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('organizations'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND (a.attnum>0 OR (a.attnum<0 AND a.attacl IS NOT NULL)) AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), deployment_observer_role AS (
 SELECT * FROM pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_proc p
 CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
   OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a
    JOIN deployment_observer_role r ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('schema',n.nspname,'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE (n.nspname,p.proname) IN (VALUES ('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app')
), related_fks AS (
 SELECT k.* FROM pg_constraint k WHERE k.contype='f'
   AND (k.conrelid IN (SELECT oid FROM relations) OR k.confrelid IN (SELECT oid FROM relations))
), foreign_key_records AS (
 SELECT jsonb_build_object('schema',ns.nspname,'name',k.conname,
   'source',k.conrelid::regclass::text,'target',k.confrelid::regclass::text,
   'namespace',kn.nspname,'type',k.contypid::regtype::text,
   'parent',CASE WHEN k.conparentid=0 THEN NULL ELSE parent.conname END,
   'validated',k.convalidated,'enforced',k.conenforced,'period',k.conperiod,
   'local',k.conislocal,'inherited',k.coninhcount,'noinherit',k.connoinherit,
   'deferrable',k.condeferrable,'deferred',k.condeferred,
   'keys',k.conkey,'foreign_keys',k.confkey,'update',k.confupdtype,
   'delete',k.confdeltype,'match',k.confmatchtype,'delete_columns',k.confdelsetcols,
   'binary_expression',pg_get_expr(k.conbin,k.conrelid),
   'primary_foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conpfeqop) WITH ORDINALITY a(op,ordinal)),
   'primary_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conppeqop) WITH ORDINALITY a(op,ordinal)),
   'foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conffeqop) WITH ORDINALITY a(op,ordinal)),
   'exclusion_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conexclop) WITH ORDINALITY a(op,ordinal)),
   'supporting_index',k.conindid::regclass::text,
   'supporting_index_definition',pg_get_indexdef(k.conindid),
   'index_flags',jsonb_build_array(i.indrelid=k.confrelid,i.indisunique,i.indisvalid,i.indisready,i.indislive,
       i.indimmediate,i.indisexclusion,i.indnkeyatts,i.indnatts,i.indkey::text,
       pg_get_expr(i.indexprs,i.indrelid),pg_get_expr(i.indpred,i.indrelid)),
   'ri',(SELECT jsonb_agg(jsonb_build_object(
       'on',t.tgrelid::regclass::text,'other',t.tgconstrrelid::regclass::text,
       'index',t.tgconstrindid::regclass::text,'function',t.tgfoid::regprocedure::text,
       'name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$',
       'internal',t.tgisinternal,'enabled',t.tgenabled,'type',t.tgtype,
       'arguments',t.tgnargs,'args',encode(t.tgargs,'hex'),'attributes',t.tgattr::text,
       'deferrable',t.tgdeferrable,'deferred',t.tginitdeferred,
       'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,'new_table',t.tgnewtable,
       'condition',pg_get_expr(t.tgqual,t.tgrelid))
       ORDER BY t.tgrelid::regclass::text,t.tgfoid::regprocedure::text)
     FROM pg_trigger t WHERE t.tgconstraint=k.oid)) AS record
 FROM related_fks k JOIN pg_class c ON c.oid=k.conrelid
 JOIN pg_namespace ns ON ns.oid=c.relnamespace
 JOIN pg_namespace kn ON kn.oid=k.connamespace
 LEFT JOIN pg_constraint parent ON parent.oid=k.conparentid
 LEFT JOIN pg_index i ON i.indexrelid=k.conindid
), legacy_root_boundary AS (
-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT jsonb_build_object('root_profile',(SELECT valid FROM root_profile),'projection',(SELECT valid FROM projection),'deactivation_guard',(SELECT valid FROM deactivation_guard),'deactivation_users_acl',(SELECT valid FROM deactivation_users_acl),'root_insert_acl',(SELECT valid FROM root_insert_acl),'receipt_guard',(SELECT valid FROM receipt_guard),'terms_current',(SELECT valid FROM terms_current),'auth7_root_profile',(SELECT valid FROM auth7_root_profile)) AS boundary
), deployment_startup AS (
 SELECT role.* FROM (VALUES ('console_auth_startup')) required(name)
 LEFT JOIN pg_roles role ON role.rolname=required.name
), deployment_rights_relations AS (
 SELECT name,oid,relkind FROM relations
 UNION ALL
 SELECT required.name,c.oid,c.relkind
 FROM (VALUES ('users'),('organizations'),('groups'),('employees'),('persons'),
   ('person_revisions'),('employee_person_bindings'),('group_memberships'),('group_role_grants')) required(name)
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
), deployment_startup_database_settings AS (
 SELECT setting FROM pg_db_role_setting d
 JOIN deployment_startup startup ON startup.oid=d.setrole
 CROSS JOIN LATERAL unnest(d.setconfig) setting
 WHERE d.setdatabase<>0
), deployment_startup_database_overrides AS (
 SELECT wanted.key,count(actual.setting) AS override_count
 FROM (VALUES ('statement_timeout'),('idle_in_transaction_session_timeout'),('transaction_timeout')) wanted(key)
 LEFT JOIN deployment_startup_database_settings actual ON split_part(actual.setting,'=',1)=wanted.key
 GROUP BY wanted.key
), deployment_table_rights AS (
 SELECT r.name, privilege.name AS privilege,
   has_table_privilege(s.oid,r.oid,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) privilege(name)
), deployment_column_rights AS (
 SELECT r.name, a.attname, privilege.name AS privilege,
   has_column_privilege(s.oid,r.oid,a.attnum,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 JOIN pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) privilege(name)
), deployment_function_rights AS (
 SELECT p.oid,n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) AS identity_arguments,
   has_function_privilege(s.oid,p.oid,'EXECUTE') AS allowed,
   has_function_privilege(s.oid,p.oid,'EXECUTE WITH GRANT OPTION') AS grantable,
   p.oid IN (
     to_regprocedure('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)'),
     to_regprocedure('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)')) AS expected_execute
 FROM deployment_startup s CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE p.proowner IN (SELECT oid FROM owner_roles)
    OR (n.nspname='public' AND p.prosecdef)
), deployment_mandatory_functions AS (
 SELECT required.identity,p.oid IS NOT NULL AS present,
   p.prokind='f' AND p.prosecdef AND p.prorettype NOT IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) AS callable_definer,
   checked.oid IS NOT NULL AS included, checked.allowed, checked.grantable
 FROM (VALUES
   ('public.account_session_shared_material_v1(uuid,uuid)'),
   ('public.auth_account_session_shared_material_v1(uuid,uuid)'),
   ('public.account_company_setup_eligibility_v1(uuid)'),
   ('public.auth_legacy_platform_source_material_v1(uuid,uuid)'),
   ('public.account_context_presence_v1(uuid)'),
   ('public.account_login_consent_v1(uuid)'),
   ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])'),
   ('public.account_registration_begin_v1()'),
   ('public.account_security_lock_exclusive_v1(uuid)'),
   ('public.account_security_lock_shared_v1(uuid)'),
   ('public.account_session_logout_v1(uuid,uuid,bigint,interval)'),
   ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.account_terms_current_v1()'),
   ('public.account_terms_registration_head_v1()'),
   ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)'),
   ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)'),
   ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)'),
   ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_cold_start_admin_v1()'),
   ('public.auth_legacy_company_lock_v1(uuid)'),
   ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)'),
   ('public.auth_legacy_group_passkey_flag_v1(uuid)'),
   ('public.auth_legacy_purge_company_v1(uuid)'),
   ('public.auth_legacy_purge_subjects_v1(uuid)'),
   ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)'),
   ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkeys_v1(uuid,uuid)'),
   ('public.auth_legacy_session_context_v1(uuid,uuid)'),
   ('public.auth_legacy_user_active_v1(uuid,uuid)'),
   ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)'),
   ('public.group_member_org_ids(uuid,uuid)'),
   ('public.group_role_grants_for_user(uuid)'),
   ('public.platform_assign_org_to_group(uuid,uuid)'),
   ('public.platform_attach_group_of_one(uuid)'),
   ('public.platform_attach_membership(uuid,uuid)'),
   ('public.platform_create_group(text,text)'),
   ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)'),
   ('public.platform_create_organization(text,text)'),
   ('public.platform_force_remove_direct_org_children(uuid)'),
   ('public.platform_force_remove_organization(uuid)'),
   ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)'),
   ('public.platform_get_group(uuid)'),
   ('public.platform_get_organization(uuid)'),
   ('public.platform_list_group_accounts(uuid)'),
   ('public.platform_list_groups()'),
   ('public.platform_list_organizations()'),
   ('public.platform_mint_group_row(uuid,text,text)'),
   ('public.platform_mint_missing_group_of_one(uuid)'),
   ('public.platform_remove_org_from_group(uuid,uuid)'),
   ('public.platform_remove_organization(uuid)'),
   ('public.platform_resolve_bootstrap_org(bytea)'),
   ('public.platform_resolve_credential_org(text)'),
   ('public.platform_resolve_token_org(bytea)'),
   ('public.platform_revoke_group_role(uuid,uuid,text)'),
   ('public.platform_set_organization_status(uuid,text)'),
   ('public.platform_update_group(uuid,text,text,text)')) required(identity)
 LEFT JOIN pg_proc p ON p.oid=to_regprocedure(required.identity)
 LEFT JOIN deployment_function_rights checked ON checked.oid=p.oid
), deployment_builtin AS (
 SELECT p.oid, p.proowner, owner.rolsuper AS owner_superuser,
   jsonb_build_object(
     'identity',p.oid::regprocedure::text,'owner',jsonb_build_array('builtin_owner'),
     'owner_superuser',owner.rolsuper,'language',language.lanname,
     'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,
     'returns_set',p.proretset,'leakproof',p.proleakproof,
     'volatility',p.provolatile,'parallel',p.proparallel,
     'result',pg_get_function_result(p.oid),'config',p.proconfig,
     'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
     'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
         CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       ORDER BY CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb),
     'account_owner_execute',has_function_privilege((SELECT oid FROM pg_roles WHERE rolname='console_account_owner'),p.oid,'EXECUTE'),
     'startup_execute',has_function_privilege((SELECT oid FROM deployment_startup),p.oid,'EXECUTE')
   ) AS record
 FROM pg_proc p JOIN pg_roles owner ON owner.oid=p.proowner
 JOIN pg_language language ON language.oid=p.prolang
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_memberships AS (
 SELECT jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
   CASE WHEN m.roleid=(SELECT oid FROM pg_roles WHERE rolname='pg_read_all_stats')
         AND m.member=o.oid AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option
         AND grantor.rolsuper AND grantor.rolcanlogin
         AND grantor.rolname NOT IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer')
     THEN jsonb_build_array('trusted_maintenance_grantor')
     ELSE jsonb_build_array('role',pg_get_userbyid(m.grantor)) END,
   m.admin_option,m.inherit_option,m.set_option) AS record
 FROM deployment_observer_active o JOIN pg_auth_members m ON m.roleid=o.oid OR m.member=o.oid
 JOIN pg_roles grantor ON grantor.oid=m.grantor
), deployment_observer_settings AS (
 SELECT CASE WHEN d.setdatabase=0 THEN 'global'
          WHEN d.setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) THEN 'current'
          ELSE 'other' END AS scope,
        d.setconfig IS NULL AS config_is_null,
        (SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
           FROM unnest(d.setconfig) setting) AS config_hashes
 FROM deployment_observer_active o JOIN pg_db_role_setting d ON d.setrole=o.oid
), deployment_observer_external_routine_grants AS (
 SELECT jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
    CASE WHEN p.oid=to_regprocedure('pg_catalog.pg_control_system()')
          AND a.grantor=p.proowner AND owner.rolsuper
      THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
    CASE WHEN a.grantee=0 THEN jsonb_build_array('public') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
    a.privilege_type,a.is_grantable) AS record
 FROM deployment_observer_active o CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_roles owner ON owner.oid=p.proowner
 CROSS JOIN LATERAL aclexplode(p.proacl) a
 WHERE p.proowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)
), deployment_observer_boundary AS (
 SELECT CASE WHEN NOT (SELECT present FROM deployment_observer_local) THEN jsonb_build_object('present',false)
 ELSE jsonb_build_object(
   'present',true,
   'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,rolcanlogin,rolinherit,
       rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconnlimit,rolvaliduntil,
       rolconfig IS NULL,(SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
         FROM unnest(rolconfig) setting)) ORDER BY rolname) FROM deployment_observer_active),
   'memberships',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_memberships),
   'settings',(SELECT jsonb_agg(jsonb_build_array(scope,config_is_null,config_hashes)
       ORDER BY scope,config_is_null,config_hashes::text COLLATE "C") FROM deployment_observer_settings),
   'external_routine_grants',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_external_routine_grants),
   'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind)
       ORDER BY n.nspname,c.relname) FROM deployment_observer_active o JOIN pg_class c ON c.relowner=o.oid
       JOIN pg_namespace n ON n.oid=c.relnamespace),
   'owner_schemas',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o JOIN pg_namespace n ON n.nspowner=o.oid),
   'schema_create',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o CROSS JOIN pg_namespace n
       WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(o.oid,n.oid,'CREATE')),
   'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_default_acl d
       LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace CROSS JOIN LATERAL aclexplode(d.defaclacl) a
       WHERE d.defaclrole=o.oid OR a.grantee=o.oid OR a.grantor=o.oid),
   'external_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
       CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)),
   'external_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_attribute col JOIN pg_class c ON c.oid=col.attrelid
       JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(col.attacl) a
       WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid))
 ) END AS record
), deployment_boundary AS (
 SELECT jsonb_build_object(
   'builtin',(SELECT record FROM deployment_builtin),
   'observer',(SELECT record FROM deployment_observer_boundary),
   'startup_table_rights',(SELECT jsonb_agg(jsonb_build_array(name,privilege,allowed) ORDER BY name,privilege) FROM deployment_table_rights),
   'startup_column_rights',(SELECT jsonb_agg(jsonb_build_array(name,attname,privilege,allowed) ORDER BY name,attname,privilege) FROM deployment_column_rights),
   'startup_function_rights',(SELECT jsonb_agg(jsonb_build_array(nspname,proname,identity_arguments,allowed,grantable,expected_execute) ORDER BY nspname,proname,identity_arguments) FROM deployment_function_rights),
   'startup_mandatory_function_rights',(SELECT jsonb_agg(jsonb_build_array(identity,present,callable_definer,included,allowed,grantable) ORDER BY identity) FROM deployment_mandatory_functions),
   'startup_managed_database_overrides',(SELECT jsonb_agg(jsonb_build_array(key,override_count) ORDER BY key) FROM deployment_startup_database_overrides),
   'startup_final_rights_valid',
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=18 AND count(oid)=18 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=27 AND count(oid)=27 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=216 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=27 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin)
 ) AS record
), company_startup_rights AS (
 SELECT
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=61 AND count(oid)=61 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=70 AND count(oid)=70 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=560 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=65 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS (
 SELECT jsonb_build_object(
  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),
  'legacy_root_boundary',(SELECT boundary FROM legacy_root_boundary),
  'tables',(SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records),
  'routines',(SELECT jsonb_agg(jsonb_build_object('metadata',record,'extra_valid',extra_valid)
      ORDER BY record->>'schema',record->>'name',record->>'identity_arguments') FROM routine_records),
  'foreign_keys',(SELECT jsonb_agg(record ORDER BY record->>'schema',record->>'source',record->>'name') FROM foreign_key_records),
  'constraint_flags',(SELECT jsonb_agg(jsonb_build_array(k.conrelid::regclass::text,k.conname,
      k.conenforced,k.conperiod,k.contypid=0,k.conparentid=0,k.coninhcount,k.conislocal)
      ORDER BY k.conrelid::regclass::text,k.conname)
    FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)),
  'trigger_links',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM (
    SELECT jsonb_build_array(t.tgrelid::regclass::text,
      CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgfoid::regprocedure::text,
      t.tgparentid=0,t.tgattr::text,t.tgoldtable,t.tgnewtable,
      k.conname,CASE WHEN k.conrelid IS NULL THEN NULL ELSE k.conrelid::regclass::text END,
      CASE WHEN t.tgconstrindid=0 THEN NULL ELSE t.tgconstrindid::regclass::text END) AS record
    FROM pg_trigger t LEFT JOIN pg_constraint k ON k.oid=t.tgconstraint
    WHERE t.tgrelid IN (SELECT oid FROM relations)) triggers),
  'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,
      CASE WHEN rolname='console_auth_rt' THEN NULL ELSE rolcanlogin END,
      rolinherit,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconfig) ORDER BY rolname) FROM protected_roles),
  'memberships',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
      pg_get_userbyid(m.grantor),m.admin_option,m.inherit_option,m.set_option)
      ORDER BY pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),pg_get_userbyid(m.grantor))
    FROM pg_auth_members m WHERE m.roleid IN (SELECT oid FROM protected_roles) OR m.member IN (SELECT oid FROM protected_roles)),
  'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner))
      ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
    WHERE c.relowner IN (SELECT oid FROM owner_roles) AND c.relkind IN ('r','p','v','m','S','f')),
  'owner_schemas',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname)
    FROM pg_namespace n WHERE n.nspowner IN (SELECT oid FROM owner_roles)),
  'schema_create',(SELECT jsonb_agg(jsonb_build_array(r.rolname,n.nspname) ORDER BY r.rolname,n.nspname)
    FROM owner_roles r CROSS JOIN pg_namespace n
    -- The current temporary namespace derives CREATE from database TEMP, not
    -- persistent schema authority. Keep every other namespace/ACL check exact.
    WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(r.oid,n.oid,'CREATE')),
  'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
      pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_default_acl d LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace
    CROSS JOIN LATERAL aclexplode(d.defaclacl) a
    WHERE d.defaclrole IN (SELECT oid FROM protected_roles) OR a.grantee IN (SELECT oid FROM owner_roles)),
  'external_owner_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL)),
  'external_owner_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_attribute col JOIN pg_class c ON c.oid=col.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace
    CROSS JOIN LATERAL aclexplode(col.attacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL))
 ) AS snapshot
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_policy_startup_rights_valid FROM snapshots
) captured;
 expected_final:=CASE predecessor
  WHEN '3755792f52a4f78236a70e509f4f1546588049daa53f7545d8a101c74684d843' THEN 'e94251d48fdee392d7632709b19d80cc04d53ea6742e0f2d6538d68391ab8bf2'
  WHEN '9bde6410d51f8b665e5ca4400820e6d8a549a6c19b7b4d84feb26ef32c4bd604' THEN 'f132054a56641dcb0cc5875d6485846df2091e631d0d902a18356d2202343fd9' ELSE NULL END;
 IF expected_final IS NULL OR prior_rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_company_policy_v2.predecessor_mismatch'; END IF;
-- Generated declared-source candidate. Not a finalized custody installer.
-- source: schema-v2.sql
-- Uninstalled successor source: activation requires exact custody review and populated-upgrade proof.
-- Physical identities, keys, foreign keys, receipt columns and history remain unchanged.
ALTER TABLE public.native_company_policy_inputs_v1
 DROP CONSTRAINT native_company_policy_inputs_v1_codec_version_check,
 DROP CONSTRAINT native_company_policy_inputs_v1_input_bytes_check,
 ADD CONSTRAINT native_company_policy_inputs_v1_codec_shape_v2 CHECK(
  (codec_version=1 AND octet_length(input_bytes) IN (123,148,155,180))
  OR (codec_version=2 AND octet_length(input_bytes) IN (121,147,154,179)));
ALTER TABLE public.native_company_policy_receipts_v1
 DROP CONSTRAINT native_company_policy_receipts_v1_codec_version_check,
 DROP CONSTRAINT native_company_policy_receipts_v1_catalog_version_check,
 DROP CONSTRAINT native_company_policy_receipts_v1_manifest_digest_check,
 ADD CONSTRAINT native_company_policy_receipts_v1_codec_shape_v2 CHECK(
  (codec_version=1 AND catalog_version='native-payroll-collection-read-v1'
   AND manifest_digest=decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex'))
  OR (codec_version=2 AND catalog_version='native-people-directory-v1'
   AND manifest_digest=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')));
ALTER TABLE public.native_company_catalog_installs
 DROP CONSTRAINT native_company_catalog_installs_catalog_policy_shape,
 ADD CONSTRAINT native_company_catalog_installs_catalog_policy_shape_v2 CHECK(
  (policy_receipt_id IS NULL AND catalog_version='native-company-identity-2026-09-19.1'
   AND manifest_digest=decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex'))
  OR (policy_receipt_id IS NOT NULL AND catalog_version='native-payroll-collection-read-v1'
   AND manifest_digest=decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex'))
  OR (policy_receipt_id IS NOT NULL AND catalog_version='native-people-directory-v1'
   AND manifest_digest=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')));
ALTER TABLE public.native_company_object_refs DROP CONSTRAINT native_company_object_refs_object_key_v2,
 ADD CONSTRAINT native_company_object_refs_object_key_v3 CHECK(
  (catalog_version='native-company-identity-2026-09-19.1' AND object_key IN ('company_workspace','company_policy_assignment'))
  OR (catalog_version='native-payroll-collection-read-v1' AND object_key='pay_run')
  OR (catalog_version='native-people-directory-v1' AND object_key='person'));
ALTER TABLE public.native_company_action_refs DROP CONSTRAINT native_company_action_refs_action_key_v2,
 ADD CONSTRAINT native_company_action_refs_action_key_v3 CHECK(
  (catalog_version='native-company-identity-2026-09-19.1' AND action_key IN ('context.discover','company.identity.read','company.policy.read','company.policy.assign','company.policy.revoke'))
  OR (catalog_version='native-payroll-collection-read-v1' AND action_key='payroll.collection.read')
  OR (catalog_version='native-people-directory-v1' AND action_key IN ('people.directory.read','people.directory.create')));
ALTER TABLE public.native_company_property_refs DROP CONSTRAINT native_company_property_refs_property_key_v2,
 ADD CONSTRAINT native_company_property_refs_property_key_v3 CHECK(
  (catalog_version='native-company-identity-2026-09-19.1' AND property_key IN ('company.name','company.slug','assignment.account_id','assignment.scope','assignment.actions','assignment.fields','assignment.valid_from','assignment.valid_until','assignment.state','assignment.revision'))
  OR (catalog_version='native-payroll-collection-read-v1' AND property_key IN ('pay_run.id','pay_run.period_start','pay_run.period_end','pay_run.source_label','pay_run.status','pay_run.calculation_enabled','pay_run.created_by','pay_run.approved_by','pay_run.approved_at','pay_run.close_receipt','pay_run.submitted_by','pay_run.submitted_at','pay_run.decided_by','pay_run.decided_at','pay_run.decision_reason','pay_run.approval_ref','pay_run.created_at','pay_run.updated_at'))
  OR (catalog_version='native-people-directory-v1' AND property_key IN ('person.employee_id','person.person_id','person.legal_name','person.employee_number','person.person_version','person.directory_registered_at')));

-- source: codec-v2.sql
-- Closed successor decoder. Stored discriminators select their exact grammar;
-- the original Payroll decoder and all acknowledged bytes remain unchanged.
CREATE FUNCTION public.native_company_policy_codec_v2(p_input bytea) RETURNS smallint
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF substring(p_input FROM 1 FOR 34)=convert_to('console.company.business-policy','UTF8')||decode('000001','hex') THEN
  RETURN 1;
 ELSIF substring(p_input FROM 1 FOR 32)=convert_to('console.company.people-policy','UTF8')||decode('000001','hex') THEN
  RETURN 2;
 END IF;
 RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
END
$body$;

CREATE FUNCTION public.native_company_policy_decode_v2(p_codec smallint,p_input bytea)
RETURNS TABLE(actor_account_id uuid,org_id uuid,command_id uuid,expected_company_epoch bigint,
 manifest_digest bytea,operation smallint,recipient_account_id uuid,expected_role_revision bigint,
 assignment_id uuid,expected_assignment_revision bigint,expires_at timestamptz,
 codec_version smallint,catalog_version text,action_key text,role_key text)
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE size integer:=octet_length(p_input); witness integer; offset_value integer;
 expiry_us bigint; action_byte integer;
BEGIN
 IF p_codec=1 THEN
  RETURN QUERY SELECT d.*,1::smallint,'native-payroll-collection-read-v1'::text,
   'payroll.collection.read'::text,'native_payroll_collection_read'::text
   FROM public.native_company_policy_decode_v1(p_input) d;
  RETURN;
 END IF;
 IF p_codec<>2 OR size NOT IN (121,147,154,179)
  OR substring(p_input FROM 1 FOR 32)<>convert_to('console.company.people-policy','UTF8')||decode('000001','hex') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 actor_account_id:=encode(substring(p_input FROM 33 FOR 16),'hex')::uuid;
 org_id:=encode(substring(p_input FROM 49 FOR 16),'hex')::uuid;
 command_id:=encode(substring(p_input FROM 65 FOR 16),'hex')::uuid;
 expected_company_epoch:=('x'||encode(substring(p_input FROM 81 FOR 8),'hex'))::bit(64)::bigint;
 manifest_digest:=substring(p_input FROM 89 FOR 32);
 operation:=get_byte(p_input,120)::smallint;
 codec_version:=2; catalog_version:='native-people-directory-v1';
 IF '00000000-0000-0000-0000-000000000000'::uuid IN (actor_account_id,org_id,command_id)
  OR org_id='00000000-0000-0000-0000-00000000face'::uuid OR expected_company_epoch<1
  OR manifest_digest<>decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')
  OR operation NOT BETWEEN 1 AND 3 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 IF operation=1 THEN
  IF size<>121 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input'; END IF;
 ELSE
  IF (operation=2 AND size NOT IN (147,179)) OR (operation=3 AND size<>154) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
  action_byte:=get_byte(p_input,121);
  IF action_byte NOT IN (1,2) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
  action_key:=(ARRAY['people.directory.read','people.directory.create'])[action_byte];
  role_key:=(ARRAY['native_people_directory_read','native_people_directory_create'])[action_byte];
  IF operation=2 THEN
   recipient_account_id:=encode(substring(p_input FROM 123 FOR 16),'hex')::uuid;
   witness:=get_byte(p_input,138);
   IF recipient_account_id='00000000-0000-0000-0000-000000000000'::uuid
    OR NOT ((witness=0 AND size=147) OR (witness=1 AND size=179)) THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
   END IF;
   IF witness=1 THEN offset_value:=140; END IF;
   expiry_us:=('x'||encode(substring(p_input FROM size-7 FOR 8),'hex'))::bit(64)::bigint;
   IF expiry_us NOT BETWEEN -62135596800000000 AND 253402268340000000 OR expiry_us%60000000<>0 THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
   END IF;
   expires_at:=timestamptz '1970-01-01 00:00:00+00'+((expiry_us/60000000)::text||' minutes')::interval;
  ELSE offset_value:=123;
  END IF;
 END IF;
 IF offset_value IS NOT NULL THEN
  expected_role_revision:=('x'||encode(substring(p_input FROM offset_value FOR 8),'hex'))::bit(64)::bigint;
  assignment_id:=encode(substring(p_input FROM offset_value+8 FOR 16),'hex')::uuid;
  expected_assignment_revision:=('x'||encode(substring(p_input FROM offset_value+24 FOR 8),'hex'))::bit(64)::bigint;
  IF expected_role_revision<>1 OR expected_assignment_revision<1
   OR assignment_id='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
 END IF;
 RETURN NEXT;
END
$body$;

-- source: material-v2.sql
-- Additive dual-codec successor; installed only after exact predecessor custody verification.

CREATE OR REPLACE FUNCTION public.identity_native_policy_material_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_operation smallint,p_input bytea)
RETURNS TABLE(actor_account_id uuid,session_id uuid,account_security_generation bigint,
 designation_system_identifier text,designation_database_name text,designation_database_oid bigint,
 designation_revision bigint,designation_receipt_id uuid,org_id uuid,current_group_id uuid,
 group_revision bigint,group_incarnation uuid,membership_id uuid,membership_revision bigint,
 membership_incarnation uuid,company_epoch bigint,current_policy_receipt_id uuid,
 origin_account_id uuid,origin_command_id uuid,origin_receipt_id uuid,administrative_account_id uuid,
 company_actor_admission_receipt_id uuid,birth_assignment_id uuid,birth_role_id uuid,
 observed_at timestamptz,source_xid xid8,source_backend_pid integer)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid; planned_admin uuid;
 planned_recipient uuid; guarded uuid; candidate record; saved bytea; saved_codec smallint; source record; control record;
 family record; group_head record; membership record; head public.company_authority_heads;
 designation public.deployment_operator_head; birth public.company_enrollment_receipts;
 root_assignment public.user_role_assignments; root_role public.policy_roles;
 terminal public.native_company_policy_receipts_v1;
BEGIN
 IF current_setting('transaction_isolation')<>'read committed'
  OR p_account IS NULL OR p_family IS NULL OR p_company IS NULL OR p_command IS NULL
  OR p_operation IS NULL OR p_operation NOT BETWEEN 1 AND 3
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_account,p_family,p_company,p_command)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'native_company_policy.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.group_id,r.administrative_account_id INTO STRICT planned_group,planned_admin
  FROM public.organizations o JOIN public.company_enrollment_receipts r ON r.org_id=o.id
   AND r.account_id=o.origin_account_id AND r.command_id=o.origin_command_id AND r.receipt_id=o.origin_receipt_id
  WHERE o.id=p_company;
 IF p_input IS NOT NULL THEN
  SELECT * INTO STRICT candidate FROM public.native_company_policy_decode_v2(public.native_company_policy_codec_v2(p_input),p_input);
  IF (candidate.actor_account_id,candidate.org_id,candidate.command_id,candidate.operation)
    IS DISTINCT FROM (p_account,p_company,p_command,p_operation) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
 END IF;
 SELECT i.input_bytes,i.codec_version INTO saved,saved_codec FROM public.native_company_policy_inputs_v1 i
  WHERE i.actor_account_id=p_account AND i.command_id=p_command AND i.org_id=p_company;
 IF saved IS NOT NULL THEN
  SELECT d.recipient_account_id INTO planned_recipient FROM public.native_company_policy_decode_v2(saved_codec,saved) d;
 ELSIF p_input IS NOT NULL THEN planned_recipient:=candidate.recipient_account_id;
 END IF;
 SELECT * INTO STRICT group_head FROM public.group_authority_lock_exclusive_v1(planned_group);
 FOR guarded IN SELECT DISTINCT id FROM unnest(ARRAY[p_account,planned_admin,planned_recipient]) id
  WHERE id IS NOT NULL ORDER BY id
 LOOP
  PERFORM 1 FROM public.account_security_lock_exclusive_v1(guarded);
 END LOOP;
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation
  OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' OR family.auth_time IS NULL
  OR NOT isfinite(family.auth_time) OR NOT isfinite(family.created_at)
  OR family.auth_time>family.created_at OR family.created_at>clock_timestamp() THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 PERFORM 1 FROM public.account_login_consent_v1(p_account);
 IF NOT public.account_company_setup_eligibility_v1(p_account) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_company_policy.forbidden';
 END IF;
 SELECT h.* INTO STRICT designation FROM public.deployment_operator_head h WHERE h.singleton=1 AND h.account_id=p_account;
 -- Integrity is checked even when a distinct recipient has become ineligible.
 FOR guarded IN SELECT DISTINCT id FROM unnest(ARRAY[planned_admin,planned_recipient]) id
  WHERE id IS NOT NULL AND id<>p_account ORDER BY id
 LOOP
  PERFORM 1 FROM public.native_company_policy_registration_custody_v1(guarded);
 END LOOP;
 SELECT o.group_id,o.status,o.origin_account_id,o.origin_command_id,o.origin_receipt_id,
  r.administrative_account_id,r.receipt_id INTO STRICT source
  FROM public.organizations o JOIN public.company_enrollment_receipts r ON r.org_id=o.id
   AND r.account_id=o.origin_account_id AND r.command_id=o.origin_command_id AND r.receipt_id=o.origin_receipt_id
  WHERE o.id=p_company;
 IF source.group_id IS DISTINCT FROM planned_group OR source.administrative_account_id IS DISTINCT FROM planned_admin THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
 END IF;
 SELECT i.input_bytes,i.codec_version INTO saved,saved_codec FROM public.native_company_policy_inputs_v1 i
  WHERE i.actor_account_id=p_account AND i.command_id=p_command AND i.org_id=p_company;
 guarded:=NULL;
 IF saved IS NOT NULL THEN
  SELECT d.recipient_account_id INTO guarded FROM public.native_company_policy_decode_v2(saved_codec,saved) d;
 ELSIF p_input IS NOT NULL THEN guarded:=candidate.recipient_account_id;
 END IF;
 IF guarded IS DISTINCT FROM planned_recipient THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
 END IF;
 IF source.status<>'ACTIVE' OR group_head.state<>'ACTIVE'
  OR NOT EXISTS(SELECT 1 FROM public.groups g WHERE g.id=planned_group AND g.status='ACTIVE') THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_company_policy.forbidden';
 END IF;
 SELECT m.membership_id,m.current_revision,m.incarnation INTO STRICT membership
  FROM public.group_memberships m JOIN public.group_membership_revisions r ON
   (r.group_id,r.org_id,r.membership_id,r.revision,r.incarnation)=
   (m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation)
  WHERE m.group_id=planned_group AND m.org_id=p_company AND r.state='ACTIVE' AND r.to_time IS NULL;
 SELECT h.* INTO STRICT head FROM public.company_authority_heads h WHERE h.org_id=p_company FOR UPDATE;
 SELECT r.* INTO STRICT birth FROM public.company_enrollment_receipts r
  JOIN public.company_enrollment_requests q ON q.account_id=r.account_id AND q.command_id=r.command_id
   AND q.committed_receipt_id=r.receipt_id AND q.state='COMMITTED'
  WHERE (r.org_id,r.account_id,r.command_id,r.receipt_id)=
   (head.org_id,head.origin_account_id,head.origin_command_id,head.origin_receipt_id);
 IF (head.origin_account_id,head.origin_command_id,head.origin_receipt_id)
   IS DISTINCT FROM (source.origin_account_id,source.origin_command_id,source.origin_receipt_id)
  OR birth.administrative_account_id IS DISTINCT FROM planned_admin OR birth.root_revision<>1
  OR NOT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company AND a.account_id=planned_admin
    AND a.admission_receipt_id=birth.receipt_id) THEN
  RAISE EXCEPTION 'native_company_policy.material_unavailable';
 END IF;
 SELECT a.* INTO STRICT root_assignment FROM public.user_role_assignments a
  WHERE a.org_id=p_company AND a.id=birth.root_assignment_id FOR SHARE;
 SELECT r.* INTO STRICT root_role FROM public.policy_roles r
  WHERE r.org_id=p_company AND r.id=root_assignment.role_id FOR SHARE;
 IF root_assignment.subject_protocol<>'NATIVE_ACCOUNT' OR root_assignment.account_id<>planned_admin
  OR root_assignment.native_current_revision<>1 OR root_assignment.policy_receipt_id IS NOT NULL
  OR root_role.subject_protocol<>'NATIVE_ACCOUNT' OR root_role.native_current_revision<>1
  OR root_role.policy_receipt_id IS NOT NULL OR root_role.role_key<>'native_company_administration'
  OR (root_assignment.origin_account_id,root_assignment.origin_command_id,root_assignment.origin_receipt_id)
    IS DISTINCT FROM (birth.account_id,birth.command_id,birth.receipt_id)
  OR (root_role.origin_account_id,root_role.origin_command_id,root_role.origin_receipt_id)
    IS DISTINCT FROM (birth.account_id,birth.command_id,birth.receipt_id) THEN
  RAISE EXCEPTION 'native_company_policy.material_unavailable';
 END IF;
 PERFORM public.native_company_policy_assert_current_head_v1(p_company);
 PERFORM ontology_api.lock_native_company_catalog_current_v2(p_company);
 RETURN QUERY SELECT p_account,p_family,control.security_generation,designation.system_identifier,
  designation.database_name,designation.database_oid,designation.revision,designation.receipt_id,
  p_company,planned_group,group_head.revision,group_head.incarnation,membership.membership_id,
  membership.current_revision,membership.incarnation,head.epoch,head.current_policy_receipt_id,
  birth.account_id,birth.command_id,birth.receipt_id,planned_admin,birth.receipt_id,
  root_assignment.id,root_role.id,clock_timestamp(),pg_current_xact_id(),pg_backend_pid();
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'native_company_policy.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: catalog-v2.sql
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

-- source: assignment-v2.sql
-- Uninstalled successor: separate read/create clauses in the existing policy owner.

CREATE OR REPLACE FUNCTION public.native_company_policy_clause_v2(p_org uuid,p_from timestamptz,p_action text) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE action_ref record; fields jsonb;
BEGIN
 IF p_action NOT IN ('people.directory.read','people.directory.create') OR p_action IS NULL THEN
  RAISE EXCEPTION 'native_company_policy.clause_custody_invalid';
 END IF;
 SELECT a.* INTO STRICT action_ref FROM public.native_company_action_refs a
  WHERE a.org_id=p_org AND a.catalog_version='native-people-directory-v1' AND a.action_key=p_action;
 SELECT jsonb_agg(jsonb_build_object('org_id',p.org_id::text,'object_type_id',p.object_type_id::text,
  'property_id',p.property_id::text,'schema_revision',p.schema_revision::text)
  ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision) INTO fields
 FROM public.native_company_property_refs p WHERE p.org_id=p_org
  AND p.catalog_version='native-people-directory-v1' AND p.object_type_id=action_ref.object_type_id
  AND (p_action='people.directory.read' OR p.property_key IN ('person.legal_name','person.employee_number'));
 IF jsonb_array_length(fields) IS DISTINCT FROM (CASE p_action WHEN 'people.directory.read' THEN 6 ELSE 2 END) OR p_from IS NULL OR NOT isfinite(p_from) THEN
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

CREATE OR REPLACE FUNCTION public.native_company_policy_apply_assignment_v1(p_company uuid,p_receipt uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.native_company_policy_receipts_v1; origin public.company_authority_heads;
 clause jsonb; clause_hash bytea; role_hash bytea; d record;
BEGIN
 r:=public.native_company_policy_effect_frame_v1(p_company,p_receipt);
 IF r.operation NOT IN (2,3) THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 SELECT h.* INTO STRICT origin FROM public.company_authority_heads h WHERE h.org_id=p_company;
 SELECT d0.* INTO STRICT d FROM public.native_company_policy_inputs_v1 i
  CROSS JOIN LATERAL public.native_company_policy_decode_v2(i.codec_version,i.input_bytes) d0
  WHERE (i.actor_account_id,i.command_id,i.org_id)=(r.actor_account_id,r.command_id,r.org_id);
 IF r.assignment_revision_before IS NULL THEN
  IF r.codec_version=1 THEN clause:=public.native_company_policy_clause_v1(p_company,r.executed_at);
  ELSE clause:=public.native_company_policy_clause_v2(p_company,r.executed_at,d.action_key); END IF;
  clause_hash:=sha256(convert_to(clause::text,'UTF8'));
  role_hash:=sha256(convert_to(jsonb_build_array(jsonb_build_object('clause_index',1,'clause_digest',encode(clause_hash,'hex')))::text,'UTF8'));
  INSERT INTO public.policy_roles(id,org_id,role_key,display_name,description,status,is_system,
   created_at,updated_at,subject_protocol,native_current_revision,created_by_account_id,updated_by_account_id,
   origin_account_id,origin_command_id,origin_receipt_id,policy_receipt_id)
  VALUES(r.role_id,p_company,d.role_key,CASE d.action_key WHEN 'payroll.collection.read' THEN '급여 목록 열람'
   WHEN 'people.directory.read' THEN '사람 목록 열람' WHEN 'people.directory.create' THEN '사람 목록 등록' END,NULL,'ACTIVE',true,
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
   FROM public.native_company_property_refs p WHERE p.org_id=p_company AND p.catalog_version=r.catalog_version
    AND (d.action_key<>'people.directory.create' OR p.property_key IN ('person.legal_name','person.employee_number'));
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

CREATE OR REPLACE FUNCTION public.native_company_policy_business_clauses_v1(p_org uuid,p_role uuid) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE role_row public.policy_roles; revision public.policy_role_revisions; clause_row public.policy_capability_clauses;
 creator public.native_company_policy_receipts_v1; d record; expected_fields integer; clause jsonb; clause_hash bytea; role_hash bytea;
BEGIN
 SELECT r.* INTO STRICT role_row FROM public.policy_roles r WHERE r.org_id=p_org AND r.id=p_role;
 SELECT r.* INTO STRICT revision FROM public.policy_role_revisions r WHERE r.org_id=p_org AND r.role_id=p_role AND r.revision=1;
 SELECT r.* INTO STRICT creator FROM public.native_company_policy_receipts_v1 r WHERE r.org_id=p_org AND r.receipt_id=role_row.policy_receipt_id;
 SELECT c.* INTO STRICT clause_row FROM public.policy_capability_clauses c WHERE c.org_id=p_org AND c.role_id=p_role AND c.role_revision=1;
 SELECT d0.* INTO STRICT d FROM public.native_company_policy_inputs_v1 i
  CROSS JOIN LATERAL public.native_company_policy_decode_v2(i.codec_version,i.input_bytes) d0
  WHERE (i.actor_account_id,i.command_id,i.org_id)=(creator.actor_account_id,creator.command_id,creator.org_id);
 IF creator.codec_version=1 THEN
  clause:=public.native_company_policy_clause_v1(p_org,revision.valid_from); expected_fields:=18;
 ELSE
  clause:=public.native_company_policy_clause_v2(p_org,revision.valid_from,d.action_key);
  expected_fields:=CASE d.action_key WHEN 'people.directory.read' THEN 6 ELSE 2 END;
 END IF;
 clause_hash:=sha256(convert_to(clause::text,'UTF8'));
 role_hash:=sha256(convert_to(jsonb_build_array(jsonb_build_object('clause_index',1,'clause_digest',encode(clause_hash,'hex')))::text,'UTF8'));
 IF creator.outcome<>'COMMITTED' OR creator.operation<>2 OR creator.assignment_revision_before IS NOT NULL
  OR creator.role_id IS DISTINCT FROM p_role OR role_row.role_key IS DISTINCT FROM d.role_key
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
  OR (SELECT count(*) FROM public.policy_capability_clause_fields f WHERE f.org_id=p_org AND f.role_id=p_role)<>expected_fields
  OR EXISTS(SELECT 1 FROM public.native_company_property_refs p WHERE p.org_id=p_org AND p.catalog_version=creator.catalog_version
   AND (d.action_key<>'people.directory.create' OR p.property_key IN ('person.legal_name','person.employee_number'))
   AND NOT EXISTS(SELECT 1 FROM public.policy_capability_clause_fields f WHERE f.org_id=p_org AND f.role_id=p_role
    AND f.role_revision=1 AND f.clause_index=1 AND f.object_type_id=p.object_type_id AND f.property_id=p.property_id
    AND f.schema_revision=p.schema_revision AND f.manifest_digest=p.manifest_digest AND f.content_digest=p.content_digest)) THEN
  RAISE EXCEPTION 'native_company_policy.clause_custody_invalid';
 END IF;
 RETURN jsonb_build_array(clause);
END
$body$;

-- source: guards-v2.sql
-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_input_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE material record; d record;
BEGIN
 IF TG_TABLE_NAME<>'native_company_policy_inputs_v1' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION 'native_company_policy.input_frame_invalid';
 END IF;
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(NEW.actor_account_id,
  NEW.accepting_session_id,NEW.org_id,NEW.command_id,NEW.operation,NEW.input_bytes);
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(NEW.codec_version,NEW.input_bytes);
 IF NEW.acceptance_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.acceptance_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR NEW.accepted_at<transaction_timestamp() OR NEW.accepted_at>clock_timestamp()
  OR NEW.input_digest IS DISTINCT FROM sha256(NEW.input_bytes) OR NEW.codec_version IS DISTINCT FROM d.codec_version
  OR public.native_company_policy_operation_check_v1(NEW.org_id,NEW.input_bytes,NEW.accepted_at) IS NOT NULL THEN
  RAISE EXCEPTION 'native_company_policy.input_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_receipt_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE material record; accepted public.native_company_policy_inputs_v1; failure text; d record;
BEGIN
 IF TG_TABLE_NAME<>'native_company_policy_receipts_v1' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION 'native_company_policy.receipt_frame_invalid';
 END IF;
 SELECT * INTO STRICT material FROM public.identity_native_policy_material_v1(NEW.actor_account_id,
  NEW.execution_session_id,NEW.org_id,NEW.command_id,NEW.operation,NULL);
 SELECT i.* INTO STRICT accepted FROM public.native_company_policy_inputs_v1 i
  WHERE i.org_id=NEW.org_id AND i.actor_account_id=NEW.actor_account_id AND i.command_id=NEW.command_id;
 SELECT * INTO STRICT d FROM public.native_company_policy_decode_v2(accepted.codec_version,accepted.input_bytes);
 IF NEW.executed_at>=accepted.execution_not_after THEN failure:='intake_expired';
 ELSE failure:=public.native_company_policy_operation_check_v1(NEW.org_id,accepted.input_bytes,NEW.executed_at); END IF;
 IF NEW.codec_version IS DISTINCT FROM d.codec_version OR NEW.catalog_version IS DISTINCT FROM d.catalog_version
  OR NEW.manifest_digest IS DISTINCT FROM d.manifest_digest
  OR (NEW.actor_account_id,NEW.org_id,NEW.command_id,NEW.operation)
   IS DISTINCT FROM (d.actor_account_id,d.org_id,d.command_id,d.operation)
  OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR accepted.acceptance_xid=NEW.effect_xid OR NEW.executed_at<transaction_timestamp() OR NEW.executed_at>clock_timestamp()
  OR NEW.executed_at<accepted.accepted_at OR NEW.epoch_before IS DISTINCT FROM material.company_epoch
  OR NEW.predecessor_receipt_id IS DISTINCT FROM material.current_policy_receipt_id
  OR NEW.outcome IS DISTINCT FROM (CASE WHEN failure IS NULL THEN 'COMMITTED' ELSE 'REJECTED' END)
  OR NEW.result_code IS DISTINCT FROM coalesce(failure,(ARRAY['installed','granted','revoked'])[NEW.operation])
  OR (NEW.outcome='COMMITTED' AND NEW.operation IN (2,3) AND
   (NEW.recipient_account_id IS DISTINCT FROM material.administrative_account_id
    OR (d.assignment_id IS NOT NULL AND (NEW.assignment_id,NEW.assignment_revision_before)
     IS DISTINCT FROM (d.assignment_id,d.expected_assignment_revision))
    OR (d.assignment_id IS NULL AND NEW.assignment_revision_before IS NOT NULL)
    OR (NEW.operation=2 AND (NEW.assignment_valid_from,NEW.assignment_valid_until)
     IS DISTINCT FROM (NEW.executed_at,d.expires_at)))) THEN
  RAISE EXCEPTION 'native_company_policy.receipt_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_participant_receipt_v1(p_table text,p_row jsonb) RETURNS uuid
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE receipt uuid; company uuid:=(p_row->>'org_id')::uuid;
BEGIN
 IF p_table IN ('policy_roles','policy_role_revisions','user_role_assignments','policy_assignment_revisions',
   'native_company_catalog_installs','ont_builtin_catalog_installs','ont_object_types') THEN
  receipt:=(p_row->>'policy_receipt_id')::uuid;
 ELSIF p_table='company_authority_heads' THEN receipt:=(p_row->>'current_policy_receipt_id')::uuid;
 ELSIF p_table IN ('policy_capability_clauses','policy_capability_clause_fields') THEN
  SELECT r.policy_receipt_id INTO receipt FROM public.policy_role_revisions r WHERE r.org_id=company
   AND r.role_id=(p_row->>'role_id')::uuid AND r.revision=(p_row->>'role_revision')::bigint;
 ELSIF p_table IN ('native_company_object_refs','native_company_action_refs','native_company_property_refs') THEN
  SELECT i.policy_receipt_id INTO receipt FROM public.native_company_catalog_installs i
   WHERE i.org_id=company AND i.catalog_version=p_row->>'catalog_version';
 ELSIF p_table IN ('ont_property_defs','ont_action_types') THEN
  SELECT o.policy_receipt_id INTO receipt FROM public.ont_object_types o
   WHERE o.org_id=company AND o.id=(p_row->>'object_type_id')::uuid;
 ELSIF p_table='ont_object_type_key_revisions' THEN
  IF p_row->>'stable_key' NOT IN ('pay_run','person') OR p_row->>'stable_key' IS NULL THEN RETURN NULL; END IF;
  IF NOT EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=company) THEN RETURN NULL; END IF;
  SELECT r.receipt_id INTO STRICT receipt FROM public.native_company_policy_receipts_v1 r WHERE r.org_id=company
   AND r.operation=1 AND r.outcome='COMMITTED'
   AND ((r.codec_version=1 AND p_row->>'stable_key'='pay_run') OR (r.codec_version=2 AND p_row->>'stable_key'='person'))
   AND r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid();
 ELSIF p_table NOT IN ('ont_link_types','ont_analytics','cedar_policy_catalog_entries','ont_object_policies') THEN
  RAISE EXCEPTION 'native_company_policy.participant_unavailable';
 END IF;
 RETURN receipt;
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_participant_admit_v1(p_table text,p_op text,p_old jsonb,p_new jsonb) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE receipt uuid; r public.native_company_policy_receipts_v1; origin public.company_authority_heads;
 company uuid:=coalesce((p_new->>'org_id')::uuid,(p_old->>'org_id')::uuid); previous public.policy_assignment_revisions;
BEGIN
 IF p_op NOT IN ('INSERT','UPDATE','DELETE') THEN RAISE EXCEPTION 'native_company_policy.participant_unavailable'; END IF;
 receipt:=public.native_company_policy_participant_receipt_v1(p_table,coalesce(p_new,p_old));
 IF p_table='user_role_assignments' AND p_op='UPDATE' AND receipt IS NOT NULL THEN
  SELECT v.policy_receipt_id INTO STRICT receipt FROM public.policy_assignment_revisions v WHERE v.org_id=company
   AND v.assignment_id=(p_new->>'id')::uuid AND v.revision=(p_new->>'native_current_revision')::bigint;
 END IF;
 IF receipt IS NULL THEN RETURN false; END IF;
 r:=public.native_company_policy_effect_frame_v1(company,receipt);
 SELECT h.* INTO STRICT origin FROM public.company_authority_heads h WHERE h.org_id=company;
 IF p_op='DELETE' THEN RAISE EXCEPTION 'native_company_policy.immutable'; END IF;
 IF p_op='UPDATE' THEN
  IF p_table='company_authority_heads' THEN
   IF p_old-'epoch'-'current_policy_receipt_id' IS DISTINCT FROM p_new-'epoch'-'current_policy_receipt_id'
    OR (p_old->>'epoch')::bigint IS DISTINCT FROM r.epoch_before
    OR (p_old->>'current_policy_receipt_id')::uuid IS DISTINCT FROM r.predecessor_receipt_id
    OR (p_new->>'epoch')::bigint IS DISTINCT FROM r.epoch_after THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
  ELSIF p_table='user_role_assignments' THEN
   IF p_old-'native_current_revision' IS DISTINCT FROM p_new-'native_current_revision'
    OR (p_new->>'id')::uuid IS DISTINCT FROM r.assignment_id
    OR (p_old->>'native_current_revision')::bigint IS DISTINCT FROM r.assignment_revision_before
    OR (p_new->>'native_current_revision')::bigint IS DISTINCT FROM r.assignment_revision_after THEN
    RAISE EXCEPTION 'native_company_policy.effect_invalid';
   END IF;
  ELSE RAISE EXCEPTION 'native_company_policy.immutable'; END IF;
  RETURN true;
 END IF;
 IF p_table='company_authority_heads' THEN RAISE EXCEPTION 'native_company_policy.effect_invalid'; END IF;
 IF p_new?'origin_account_id' AND
  ((p_new->>'origin_account_id')::uuid,(p_new->>'origin_command_id')::uuid,(p_new->>'origin_receipt_id')::uuid)
   IS DISTINCT FROM (origin.origin_account_id,origin.origin_command_id,origin.origin_receipt_id) THEN
  RAISE EXCEPTION 'native_company_policy.origin_invalid';
 END IF;
 IF p_table IN ('policy_roles','policy_role_revisions','user_role_assignments','policy_assignment_revisions',
  'policy_capability_clauses','policy_capability_clause_fields') THEN
  IF r.operation NOT IN (2,3) OR (p_table<>'policy_assignment_revisions' AND r.assignment_revision_before IS NOT NULL)
   OR (p_new?'role_id' AND (p_new->>'role_id')::uuid IS DISTINCT FROM r.role_id)
   OR (p_table='policy_roles' AND (p_new->>'id')::uuid IS DISTINCT FROM r.role_id)
   OR (p_table='user_role_assignments' AND (p_new->>'id')::uuid IS DISTINCT FROM r.assignment_id)
   OR (p_table='policy_assignment_revisions' AND ((p_new->>'assignment_id')::uuid IS DISTINCT FROM r.assignment_id
     OR (p_new->>'revision')::bigint IS DISTINCT FROM r.assignment_revision_after)) THEN
   RAISE EXCEPTION 'native_company_policy.effect_invalid';
  END IF;
 ELSE
  IF r.operation<>1 OR (p_new?'object_type_id' AND (p_new->>'object_type_id')::uuid IS DISTINCT FROM r.installed_object_type_id)
   OR (p_table='ont_object_types' AND (p_new->>'id')::uuid IS DISTINCT FROM r.installed_object_type_id)
   OR (p_new?'catalog_version' AND p_new->>'catalog_version' IS DISTINCT FROM r.catalog_version)
   OR (p_table='ont_object_type_key_revisions' AND (p_new->>'stable_key' IS DISTINCT FROM (CASE r.codec_version WHEN 1 THEN 'pay_run' WHEN 2 THEN 'person' END) OR (p_new->>'revision')::bigint<>1)) THEN
   RAISE EXCEPTION 'native_company_policy.effect_invalid';
  END IF;
 END IF;
 IF (p_new?'created_at' AND (p_new->>'created_at')::timestamptz IS DISTINCT FROM r.executed_at)
  OR (p_new?'actor_account_id' AND (p_new->>'actor_account_id')::uuid IS DISTINCT FROM r.actor_account_id)
  OR (p_new?'session_id' AND (p_new->>'session_id')::uuid IS DISTINCT FROM r.execution_session_id) THEN
  RAISE EXCEPTION 'native_company_policy.effect_invalid';
 END IF;
 RETURN true;
END
$body$;

-- source: closure-v2.sql
-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_accept_snapshot_v1(i public.native_company_policy_inputs_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT CASE i.codec_version WHEN 1 THEN jsonb_build_object('protocol','COMPANY_BUSINESS_POLICY_V1','command_id',i.command_id::text,
 'intake_receipt_id',i.intake_receipt_id::text,'operation',(ARRAY['InstallPayrollReadCatalogV1','GrantPayrollReadV1','RevokePayrollReadV1'])[i.operation],
 'input_digest',encode(i.input_digest,'hex'),'session_id',i.accepting_session_id::text)
 ELSE jsonb_build_object('protocol','COMPANY_PEOPLE_POLICY_V1','command_id',i.command_id::text,
  'intake_receipt_id',i.intake_receipt_id::text,
  'operation',(ARRAY['InstallPeopleDirectoryCatalogV1','GrantPeopleDirectoryV1','RevokePeopleDirectoryV1'])[i.operation],
  'action',(SELECT d.action_key FROM public.native_company_policy_decode_v2(i.codec_version,i.input_bytes) d),
  'input_digest',encode(i.input_digest,'hex'),'session_id',i.accepting_session_id::text) END $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_complete_snapshot_v1(r public.native_company_policy_receipts_v1) RETURNS jsonb
LANGUAGE sql STABLE STRICT SECURITY INVOKER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT CASE r.codec_version WHEN 1 THEN jsonb_build_object('protocol','COMPANY_BUSINESS_POLICY_V1','command_id',r.command_id::text,
 'intake_receipt_id',r.intake_receipt_id::text,'operation',(ARRAY['InstallPayrollReadCatalogV1','GrantPayrollReadV1','RevokePayrollReadV1'])[r.operation],
 'input_digest',encode(r.input_digest,'hex'),'session_id',r.execution_session_id::text,
 'receipt_id',r.receipt_id::text,'outcome',r.outcome,'result_code',r.result_code,
 'epoch_before',r.epoch_before::text,'epoch_after',r.epoch_after::text,'predecessor_receipt_id',r.predecessor_receipt_id::text)
 ELSE jsonb_build_object('protocol','COMPANY_PEOPLE_POLICY_V1','command_id',r.command_id::text,
  'intake_receipt_id',r.intake_receipt_id::text,
  'operation',(ARRAY['InstallPeopleDirectoryCatalogV1','GrantPeopleDirectoryV1','RevokePeopleDirectoryV1'])[r.operation],
  'action',(SELECT d.action_key FROM public.native_company_policy_inputs_v1 i
    CROSS JOIN LATERAL public.native_company_policy_decode_v2(i.codec_version,i.input_bytes) d
    WHERE (i.actor_account_id,i.command_id,i.org_id,i.codec_version)=(r.actor_account_id,r.command_id,r.org_id,r.codec_version)),
  'input_digest',encode(r.input_digest,'hex'),'session_id',r.execution_session_id::text,
  'receipt_id',r.receipt_id::text,'outcome',r.outcome,'result_code',r.result_code,
  'epoch_before',r.epoch_before::text,'epoch_after',r.epoch_after::text,
  'predecessor_receipt_id',r.predecessor_receipt_id::text) END $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_assert_current_head_v1(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE h public.company_authority_heads; r public.native_company_policy_receipts_v1; i public.native_company_policy_inputs_v1;
BEGIN
 SELECT x.* INTO STRICT h FROM public.company_authority_heads x WHERE x.org_id=p_org;
 IF h.epoch=1 THEN
  IF h.current_policy_receipt_id IS NOT NULL THEN RAISE EXCEPTION 'native_company_policy.head_custody_invalid'; END IF;
  RETURN;
 END IF;
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=h.current_policy_receipt_id;
 SELECT x.* INTO STRICT i FROM public.native_company_policy_inputs_v1 x WHERE x.org_id=p_org
  AND x.actor_account_id=r.actor_account_id AND x.command_id=r.command_id;
 IF r.outcome<>'COMMITTED' OR r.committed_epoch IS DISTINCT FROM h.epoch OR r.epoch_after<>h.epoch
  OR r.epoch_before::numeric+1<>r.epoch_after::numeric OR i.input_digest IS DISTINCT FROM sha256(i.input_bytes)
  OR (i.intake_receipt_id,i.operation,i.codec_version,i.input_digest)
    IS DISTINCT FROM (r.intake_receipt_id,r.operation,r.codec_version,r.input_digest)
  OR (SELECT count(*) FROM public.audit_events e WHERE e.org_id=p_org AND e.action='policy.company_command.complete'
    AND e.target_id=r.receipt_id::text)<>1
  OR NOT EXISTS(SELECT 1 FROM public.audit_events e WHERE e.org_id=p_org AND e.actor=r.actor_account_id
    AND e.action='policy.company_command.complete' AND e.target_type='native_company_policy_receipts_v1'
    AND e.target_id=r.receipt_id::text AND e.before_snap IS NULL AND e.branch_id IS NULL
    AND e.occurred_at=r.executed_at AND e.after_snap=public.native_company_policy_complete_snapshot_v1(r)) THEN
  RAISE EXCEPTION 'native_company_policy.head_custody_invalid';
 END IF;
 PERFORM 1 FROM public.native_company_policy_decode_v2(i.codec_version,i.input_bytes);
END
$body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_assert_effects_v1(p_org uuid,p_receipt uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.native_company_policy_receipts_v1; h public.company_authority_heads;
 a public.user_role_assignments; ar public.policy_assignment_revisions; prior public.policy_assignment_revisions;
 rr public.policy_role_revisions; role_row public.policy_roles; creator public.native_company_policy_receipts_v1;
 participants integer;
BEGIN
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=p_receipt;
 SELECT x.* INTO STRICT h FROM public.company_authority_heads x WHERE x.org_id=p_org;
 SELECT (SELECT count(*) FROM public.policy_roles x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.policy_role_revisions x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.user_role_assignments x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.policy_assignment_revisions x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.native_company_catalog_installs x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.ont_builtin_catalog_installs x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt)
  +(SELECT count(*) FROM public.ont_object_types x WHERE x.org_id=p_org AND x.policy_receipt_id=p_receipt) INTO participants;
 IF r.outcome='REJECTED' THEN
  IF participants<>0 THEN RAISE EXCEPTION 'native_company_policy.effect_closure_invalid'; END IF;
  RETURN;
 END IF;
 IF r.operation=1 THEN
  IF participants<>3 THEN RAISE EXCEPTION 'native_company_policy.effect_closure_invalid'; END IF;
  IF r.codec_version=1 THEN PERFORM public.native_company_policy_assert_payroll_catalog_v1(p_org);
  ELSIF r.codec_version=2 THEN PERFORM public.native_company_policy_assert_people_catalog_v1(p_org);
  ELSE RAISE EXCEPTION 'native_company_policy.effect_closure_invalid'; END IF;
  RETURN;
 END IF;
 IF participants<>(CASE WHEN r.assignment_revision_before IS NULL THEN 4 ELSE 1 END) THEN
  RAISE EXCEPTION 'native_company_policy.effect_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT a FROM public.user_role_assignments x WHERE x.org_id=p_org AND x.id=r.assignment_id;
 SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x
  WHERE x.org_id=p_org AND x.assignment_id=r.assignment_id AND x.revision=r.assignment_revision_after;
 SELECT x.* INTO STRICT role_row FROM public.policy_roles x WHERE x.org_id=p_org AND x.id=r.role_id;
 SELECT x.* INTO STRICT rr FROM public.policy_role_revisions x WHERE x.org_id=p_org AND x.role_id=r.role_id AND x.revision=1;
 SELECT x.* INTO STRICT creator FROM public.native_company_policy_receipts_v1 x WHERE x.org_id=p_org AND x.receipt_id=a.policy_receipt_id;
 IF a.subject_protocol<>'NATIVE_ACCOUNT' OR a.account_id IS DISTINCT FROM r.recipient_account_id
  OR a.role_id IS DISTINCT FROM r.role_id OR a.created_at IS DISTINCT FROM creator.executed_at
  OR a.assigned_by_account_id IS DISTINCT FROM creator.actor_account_id
  OR a.native_current_revision<r.assignment_revision_after OR a.policy_receipt_id IS DISTINCT FROM role_row.policy_receipt_id
  OR (a.origin_account_id,a.origin_command_id,a.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR (role_row.origin_account_id,role_row.origin_command_id,role_row.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR (rr.origin_account_id,rr.origin_command_id,rr.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR (ar.origin_account_id,ar.origin_command_id,ar.origin_receipt_id) IS DISTINCT FROM
   (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
  OR ar.policy_receipt_id IS DISTINCT FROM r.receipt_id OR ar.subject_protocol<>'NATIVE_ACCOUNT'
  OR ar.account_id IS DISTINCT FROM r.recipient_account_id OR ar.role_id IS DISTINCT FROM r.role_id OR ar.role_revision<>1
  OR (ar.state,ar.valid_from,ar.valid_until,ar.actor_account_id,ar.session_id,ar.created_at)
    IS DISTINCT FROM (r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until,r.actor_account_id,r.execution_session_id,r.executed_at)
  OR ar.ceiling_digest IS DISTINCT FROM rr.clause_digest OR ar.valid_from<rr.valid_from
  OR (r.operation=2 AND (ar.valid_from<>r.executed_at OR ar.valid_until>r.executed_at+interval '30 days')) THEN
  RAISE EXCEPTION 'native_company_policy.effect_closure_invalid';
 END IF;
 IF r.assignment_revision_before IS NOT NULL THEN
  SELECT x.* INTO STRICT prior FROM public.policy_assignment_revisions x WHERE x.org_id=p_org
   AND x.assignment_id=r.assignment_id AND x.revision=r.assignment_revision_before;
  IF (prior.account_id,prior.role_id,prior.role_revision,prior.ceiling_digest)
    IS DISTINCT FROM (ar.account_id,ar.role_id,ar.role_revision,ar.ceiling_digest)
   OR (r.operation=3 AND (prior.state<>'ACTIVE' OR (prior.valid_from,prior.valid_until)
     IS DISTINCT FROM (ar.valid_from,ar.valid_until)))
   OR (r.operation=2 AND prior.state='ACTIVE' AND prior.valid_until>r.executed_at) THEN
   RAISE EXCEPTION 'native_company_policy.effect_closure_invalid';
  END IF;
 END IF;
 PERFORM public.native_company_policy_business_clauses_v1(p_org,r.role_id);
END
$body$;

-- source: audit-v2.sql
-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION public.native_company_policy_ontology_snapshot_v1(r public.native_company_policy_receipts_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE SET search_path=pg_catalog,pg_temp
AS $body$ SELECT jsonb_build_object('stable_key',CASE r.codec_version WHEN 1 THEN 'pay_run' WHEN 2 THEN 'person' END,'schema_version',1,'lifecycle_state','published',
 'catalog_version',r.catalog_version,'manifest_digest',encode(r.manifest_digest,'hex')) $body$;

CREATE OR REPLACE FUNCTION public.native_company_policy_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE admitted boolean;
BEGIN
 IF TG_TABLE_NAME<>'audit_events' OR TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_company_policy.audit_frame_invalid'; END IF;
 admitted:=public.native_company_policy_audit_admit_v1(NEW);
 IF NOT admitted AND (NEW.action IN ('policy.company_command.accept','policy.company_command.complete')
  OR (NEW.action='ontology.object_type.builtin_install'
    AND NEW.after_snap->>'catalog_version' IN ('native-payroll-collection-read-v1','native-people-directory-v1'))) THEN
  RAISE EXCEPTION 'native_company_policy.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.company_enrollment_ontology_audit_guard_v2() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; native_target boolean;
BEGIN
 -- The separate ALWAYS native policy audit guard independently requires its exact live frame.
 IF NEW.action='ontology.object_type.builtin_install' AND NEW.after_snap->>'catalog_version' IN ('native-payroll-collection-read-v1','native-people-directory-v1') THEN RETURN NEW; END IF;
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

-- source: current-read-v3.sql
-- Uninstalled dual-codec successor; activation requires exact custody verification.

CREATE OR REPLACE FUNCTION ontology_api.lock_native_company_catalog_current_v2(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); found_count integer; installation record; extra integer; people integer;
BEGIN
 PERFORM set_config('app.current_org',p_org::text,true);
 SELECT n.* INTO STRICT installation FROM public.native_company_catalog_installs n WHERE n.org_id=p_org AND n.catalog_version='native-company-identity-2026-09-19.1';
 IF installation.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
  OR installation.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex') THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 SELECT count(*)::integer INTO extra FROM public.native_company_catalog_installs n WHERE n.org_id=p_org
  AND n.catalog_version='native-payroll-collection-read-v1';
 SELECT count(*)::integer INTO people FROM public.native_company_catalog_installs n WHERE n.org_id=p_org
  AND n.catalog_version='native-people-directory-v1';
 IF extra NOT IN (0,1) OR people NOT IN (0,1)
  OR (SELECT count(*) FROM public.native_company_catalog_installs n WHERE n.org_id=p_org)<>1+extra+people THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM k.org_id FROM public.ont_object_type_key_revisions k
  JOIN public.native_company_object_refs r ON r.org_id=k.org_id AND r.object_key=k.stable_key
  WHERE k.org_id=p_org ORDER BY k.org_id,k.stable_key COLLATE "C" FOR SHARE OF k;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>2+extra+people THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM o.id FROM public.ont_object_types o JOIN public.native_company_object_refs r ON r.org_id=o.org_id AND r.object_type_id=o.id
  WHERE o.org_id=p_org ORDER BY o.org_id,o.id FOR SHARE OF o;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>2+extra+people THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM p.id FROM public.ont_property_defs p JOIN public.native_company_property_refs r ON r.org_id=p.org_id AND r.property_id=p.id
  WHERE p.org_id=p_org ORDER BY p.org_id,p.object_type_id,p.id FOR SHARE OF p;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>10+18*extra+6*people THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM a.id FROM public.ont_action_types a JOIN public.native_company_action_refs r ON r.org_id=a.org_id AND r.action_type_id=a.id
  WHERE a.org_id=p_org ORDER BY a.org_id,a.object_type_id,a.id FOR SHARE OF a;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>5+extra+2*people THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 IF (SELECT count(*) FROM public.native_company_object_refs r WHERE r.org_id=p_org)<>2+extra+people
  OR (SELECT count(*) FROM public.native_company_action_refs r WHERE r.org_id=p_org)<>5+extra+2*people
  OR (SELECT count(*) FROM public.native_company_property_refs r WHERE r.org_id=p_org)<>10+18*extra+6*people
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
 IF (SELECT count(*) FROM public.ont_object_types o WHERE o.org_id=p_org)<>2+extra+people
  OR (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=p_org)<>10+18*extra+6*people
  OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=p_org)<>5+extra+2*people
  OR EXISTS(SELECT 1 FROM public.ont_link_types l WHERE l.org_id=p_org)
  OR EXISTS(SELECT 1 FROM public.ont_analytics a WHERE a.org_id=p_org) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 IF extra=1 THEN PERFORM public.native_company_policy_assert_payroll_catalog_v1(p_org); END IF;
 IF people=1 THEN PERFORM public.native_company_policy_assert_people_catalog_v1(p_org); END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: commands-v2.sql
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

-- source: acl-v2.sql
-- Exact new routine roster; activation requires independently verified custody.
DO $acl$
DECLARE routine record; grantee_name text;
BEGIN
 FOR routine IN SELECT * FROM (VALUES
  ('public.native_company_policy_codec_v2(bytea)','console_account_owner',NULL::text),
  ('public.native_company_policy_decode_v2(smallint,bytea)','console_account_owner',NULL::text),
  ('public.native_company_people_manifest_v1()','console_account_owner','console_ontology_writer'),
  ('public.native_company_policy_assert_people_catalog_v1(uuid)','console_account_owner','console_ontology_writer'),
  ('ontology_api.install_native_company_people_catalog_v1(uuid,uuid)','console_ontology_writer','console_account_owner'),
  ('public.native_company_policy_form_v2(uuid,uuid,uuid,uuid,smallint,smallint,text)','console_account_owner','console_rt'),
  ('public.native_company_policy_clause_v2(uuid,timestamptz,text)','console_account_owner',NULL::text)
 ) AS required(signature,owner_name,executor_name)
 LOOP
  EXECUTE format('ALTER FUNCTION %s OWNER TO %I',routine.signature,routine.owner_name);
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',routine.signature);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p
   CROSS JOIN LATERAL aclexplode(p.proacl) a JOIN pg_roles r ON r.oid=a.grantee
   WHERE p.oid=routine.signature::regprocedure AND r.rolname<>routine.owner_name
  LOOP
   EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',routine.signature,grantee_name);
  END LOOP;
  IF routine.executor_name IS NOT NULL THEN
   EXECUTE format('GRANT EXECUTE ON FUNCTION %s TO %I',routine.signature,routine.executor_name);
  END IF;
 END LOOP;
END
$acl$;

-- Immutable People reference; no grants or business records are created.
DO $people_reference$
BEGIN
 INSERT INTO public.ont_builtin_catalog_allowlist(catalog_version,manifest_digest)
 VALUES('native-people-directory-v1',decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex'))
 ON CONFLICT(catalog_version) DO NOTHING;
 IF (SELECT manifest_digest FROM public.ont_builtin_catalog_allowlist
     WHERE catalog_version='native-people-directory-v1') IS DISTINCT FROM
     decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex') THEN
  RAISE EXCEPTION 'native_company_policy.catalog_reference_mismatch';
 END IF;
END
$people_reference$;

 SET CONSTRAINTS ALL IMMEDIATE;
 SELECT snapshot_sha256,native_policy_startup_rights_valid INTO observed,rights_valid FROM (
-- READ-ONLY disposable capture. Freeze only after independent declared-source comparison.
WITH wanted(name) AS (VALUES
 ('accounts'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('company_actors'),
 ('account_context_candidates'),
 ('deployment_operator_receipts'),
 ('deployment_operator_head'),
 ('audit_events'),
 ('native_company_policy_inputs_v1'),
 ('native_company_policy_receipts_v1'),
 ('cedar_policy_catalog_entries'),
 ('company_authority_heads'),
 ('company_enrollment_effect_bindings'),
 ('company_enrollment_receipts'),
 ('company_enrollment_request_events'),
 ('company_enrollment_requests'),
 ('group_authority_heads'),
 ('group_membership_revisions'),
 ('group_memberships'),
 ('group_role_grants'),
 ('groups'),
 ('native_company_action_refs'),
 ('native_company_catalog_installs'),
 ('native_company_object_refs'),
 ('native_company_property_refs'),
 ('ont_action_types'),
 ('ont_analytics'),
 ('ont_builtin_catalog_allowlist'),
 ('ont_builtin_catalog_installs'),
 ('ont_link_types'),
 ('ont_object_policies'),
 ('ont_object_type_key_revisions'),
 ('ont_object_types'),
 ('ont_property_defs'),
 ('organizations'),
 ('platform_force_removal_effect_bindings'),
 ('platform_force_removal_receipts'),
 ('platform_legacy_catalog_effect_bindings'),
 ('platform_legacy_membership_effect_bindings'),
 ('platform_legacy_topology_effect_bindings'),
 ('platform_legacy_topology_receipts'),
 ('platform_legacy_user_birth_witnesses'),
 ('policy_assignment_revisions'),
 ('policy_capability_clause_fields'),
 ('policy_capability_clauses'),
 ('policy_role_conditions'),
 ('policy_role_permissions'),
 ('policy_role_revisions'),
 ('policy_roles'),
 ('user_role_assignments'),
 ('users')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND (a.attnum>0 OR (a.attnum<0 AND a.attacl IS NOT NULL)) AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), deployment_observer_role AS (
 SELECT * FROM pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_proc p
 CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
   OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a
    JOIN deployment_observer_role r ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('schema',n.nspname,'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE (n.nspname,p.proname) IN (VALUES ('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app')
), related_fks AS (
 SELECT k.* FROM pg_constraint k WHERE k.contype='f'
   AND (k.conrelid IN (SELECT oid FROM relations) OR k.confrelid IN (SELECT oid FROM relations))
), foreign_key_records AS (
 SELECT jsonb_build_object('schema',ns.nspname,'name',k.conname,
   'source',k.conrelid::regclass::text,'target',k.confrelid::regclass::text,
   'namespace',kn.nspname,'type',k.contypid::regtype::text,
   'parent',CASE WHEN k.conparentid=0 THEN NULL ELSE parent.conname END,
   'validated',k.convalidated,'enforced',k.conenforced,'period',k.conperiod,
   'local',k.conislocal,'inherited',k.coninhcount,'noinherit',k.connoinherit,
   'deferrable',k.condeferrable,'deferred',k.condeferred,
   'keys',k.conkey,'foreign_keys',k.confkey,'update',k.confupdtype,
   'delete',k.confdeltype,'match',k.confmatchtype,'delete_columns',k.confdelsetcols,
   'binary_expression',pg_get_expr(k.conbin,k.conrelid),
   'primary_foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conpfeqop) WITH ORDINALITY a(op,ordinal)),
   'primary_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conppeqop) WITH ORDINALITY a(op,ordinal)),
   'foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conffeqop) WITH ORDINALITY a(op,ordinal)),
   'exclusion_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conexclop) WITH ORDINALITY a(op,ordinal)),
   'supporting_index',k.conindid::regclass::text,
   'supporting_index_definition',pg_get_indexdef(k.conindid),
   'index_flags',jsonb_build_array(i.indrelid=k.confrelid,i.indisunique,i.indisvalid,i.indisready,i.indislive,
       i.indimmediate,i.indisexclusion,i.indnkeyatts,i.indnatts,i.indkey::text,
       pg_get_expr(i.indexprs,i.indrelid),pg_get_expr(i.indpred,i.indrelid)),
   'ri',(SELECT jsonb_agg(jsonb_build_object(
       'on',t.tgrelid::regclass::text,'other',t.tgconstrrelid::regclass::text,
       'index',t.tgconstrindid::regclass::text,'function',t.tgfoid::regprocedure::text,
       'name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$',
       'internal',t.tgisinternal,'enabled',t.tgenabled,'type',t.tgtype,
       'arguments',t.tgnargs,'args',encode(t.tgargs,'hex'),'attributes',t.tgattr::text,
       'deferrable',t.tgdeferrable,'deferred',t.tginitdeferred,
       'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,'new_table',t.tgnewtable,
       'condition',pg_get_expr(t.tgqual,t.tgrelid))
       ORDER BY t.tgrelid::regclass::text,t.tgfoid::regprocedure::text)
     FROM pg_trigger t WHERE t.tgconstraint=k.oid)) AS record
 FROM related_fks k JOIN pg_class c ON c.oid=k.conrelid
 JOIN pg_namespace ns ON ns.oid=c.relnamespace
 JOIN pg_namespace kn ON kn.oid=k.connamespace
 LEFT JOIN pg_constraint parent ON parent.oid=k.conparentid
 LEFT JOIN pg_index i ON i.indexrelid=k.conindid
), legacy_root_boundary AS (
-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT jsonb_build_object('root_profile',(SELECT valid FROM root_profile),'projection',(SELECT valid FROM projection),'deactivation_guard',(SELECT valid FROM deactivation_guard),'deactivation_users_acl',(SELECT valid FROM deactivation_users_acl),'root_insert_acl',(SELECT valid FROM root_insert_acl),'receipt_guard',(SELECT valid FROM receipt_guard),'terms_current',(SELECT valid FROM terms_current),'auth7_root_profile',(SELECT valid FROM auth7_root_profile)) AS boundary
), deployment_startup AS (
 SELECT role.* FROM (VALUES ('console_auth_startup')) required(name)
 LEFT JOIN pg_roles role ON role.rolname=required.name
), deployment_rights_relations AS (
 SELECT name,oid,relkind FROM relations
 UNION ALL
 SELECT required.name,c.oid,c.relkind
 FROM (VALUES ('users'),('organizations'),('groups'),('employees'),('persons'),
   ('person_revisions'),('employee_person_bindings'),('group_memberships'),('group_role_grants')) required(name)
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
), deployment_startup_database_settings AS (
 SELECT setting FROM pg_db_role_setting d
 JOIN deployment_startup startup ON startup.oid=d.setrole
 CROSS JOIN LATERAL unnest(d.setconfig) setting
 WHERE d.setdatabase<>0
), deployment_startup_database_overrides AS (
 SELECT wanted.key,count(actual.setting) AS override_count
 FROM (VALUES ('statement_timeout'),('idle_in_transaction_session_timeout'),('transaction_timeout')) wanted(key)
 LEFT JOIN deployment_startup_database_settings actual ON split_part(actual.setting,'=',1)=wanted.key
 GROUP BY wanted.key
), deployment_table_rights AS (
 SELECT r.name, privilege.name AS privilege,
   has_table_privilege(s.oid,r.oid,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) privilege(name)
), deployment_column_rights AS (
 SELECT r.name, a.attname, privilege.name AS privilege,
   has_column_privilege(s.oid,r.oid,a.attnum,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 JOIN pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) privilege(name)
), deployment_function_rights AS (
 SELECT p.oid,n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) AS identity_arguments,
   has_function_privilege(s.oid,p.oid,'EXECUTE') AS allowed,
   has_function_privilege(s.oid,p.oid,'EXECUTE WITH GRANT OPTION') AS grantable,
   p.oid IN (
     to_regprocedure('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)'),
     to_regprocedure('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)')) AS expected_execute
 FROM deployment_startup s CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE p.proowner IN (SELECT oid FROM owner_roles)
    OR (n.nspname='public' AND p.prosecdef)
), deployment_mandatory_functions AS (
 SELECT required.identity,p.oid IS NOT NULL AS present,
   p.prokind='f' AND p.prosecdef AND p.prorettype NOT IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) AS callable_definer,
   checked.oid IS NOT NULL AS included, checked.allowed, checked.grantable
 FROM (VALUES
   ('public.account_session_shared_material_v1(uuid,uuid)'),
   ('public.auth_account_session_shared_material_v1(uuid,uuid)'),
   ('public.account_company_setup_eligibility_v1(uuid)'),
   ('public.auth_legacy_platform_source_material_v1(uuid,uuid)'),
   ('public.account_context_presence_v1(uuid)'),
   ('public.account_login_consent_v1(uuid)'),
   ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])'),
   ('public.account_registration_begin_v1()'),
   ('public.account_security_lock_exclusive_v1(uuid)'),
   ('public.account_security_lock_shared_v1(uuid)'),
   ('public.account_session_logout_v1(uuid,uuid,bigint,interval)'),
   ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.account_terms_current_v1()'),
   ('public.account_terms_registration_head_v1()'),
   ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)'),
   ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)'),
   ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)'),
   ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_cold_start_admin_v1()'),
   ('public.auth_legacy_company_lock_v1(uuid)'),
   ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)'),
   ('public.auth_legacy_group_passkey_flag_v1(uuid)'),
   ('public.auth_legacy_purge_company_v1(uuid)'),
   ('public.auth_legacy_purge_subjects_v1(uuid)'),
   ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)'),
   ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkeys_v1(uuid,uuid)'),
   ('public.auth_legacy_session_context_v1(uuid,uuid)'),
   ('public.auth_legacy_user_active_v1(uuid,uuid)'),
   ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)'),
   ('public.group_member_org_ids(uuid,uuid)'),
   ('public.group_role_grants_for_user(uuid)'),
   ('public.platform_assign_org_to_group(uuid,uuid)'),
   ('public.platform_attach_group_of_one(uuid)'),
   ('public.platform_attach_membership(uuid,uuid)'),
   ('public.platform_create_group(text,text)'),
   ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)'),
   ('public.platform_create_organization(text,text)'),
   ('public.platform_force_remove_direct_org_children(uuid)'),
   ('public.platform_force_remove_organization(uuid)'),
   ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)'),
   ('public.platform_get_group(uuid)'),
   ('public.platform_get_organization(uuid)'),
   ('public.platform_list_group_accounts(uuid)'),
   ('public.platform_list_groups()'),
   ('public.platform_list_organizations()'),
   ('public.platform_mint_group_row(uuid,text,text)'),
   ('public.platform_mint_missing_group_of_one(uuid)'),
   ('public.platform_remove_org_from_group(uuid,uuid)'),
   ('public.platform_remove_organization(uuid)'),
   ('public.platform_resolve_bootstrap_org(bytea)'),
   ('public.platform_resolve_credential_org(text)'),
   ('public.platform_resolve_token_org(bytea)'),
   ('public.platform_revoke_group_role(uuid,uuid,text)'),
   ('public.platform_set_organization_status(uuid,text)'),
   ('public.platform_update_group(uuid,text,text,text)')) required(identity)
 LEFT JOIN pg_proc p ON p.oid=to_regprocedure(required.identity)
 LEFT JOIN deployment_function_rights checked ON checked.oid=p.oid
), deployment_builtin AS (
 SELECT p.oid, p.proowner, owner.rolsuper AS owner_superuser,
   jsonb_build_object(
     'identity',p.oid::regprocedure::text,'owner',jsonb_build_array('builtin_owner'),
     'owner_superuser',owner.rolsuper,'language',language.lanname,
     'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,
     'returns_set',p.proretset,'leakproof',p.proleakproof,
     'volatility',p.provolatile,'parallel',p.proparallel,
     'result',pg_get_function_result(p.oid),'config',p.proconfig,
     'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
     'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
         CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       ORDER BY CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb),
     'account_owner_execute',has_function_privilege((SELECT oid FROM pg_roles WHERE rolname='console_account_owner'),p.oid,'EXECUTE'),
     'startup_execute',has_function_privilege((SELECT oid FROM deployment_startup),p.oid,'EXECUTE')
   ) AS record
 FROM pg_proc p JOIN pg_roles owner ON owner.oid=p.proowner
 JOIN pg_language language ON language.oid=p.prolang
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_memberships AS (
 SELECT jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
   CASE WHEN m.roleid=(SELECT oid FROM pg_roles WHERE rolname='pg_read_all_stats')
         AND m.member=o.oid AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option
         AND grantor.rolsuper AND grantor.rolcanlogin
         AND grantor.rolname NOT IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer')
     THEN jsonb_build_array('trusted_maintenance_grantor')
     ELSE jsonb_build_array('role',pg_get_userbyid(m.grantor)) END,
   m.admin_option,m.inherit_option,m.set_option) AS record
 FROM deployment_observer_active o JOIN pg_auth_members m ON m.roleid=o.oid OR m.member=o.oid
 JOIN pg_roles grantor ON grantor.oid=m.grantor
), deployment_observer_settings AS (
 SELECT CASE WHEN d.setdatabase=0 THEN 'global'
          WHEN d.setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) THEN 'current'
          ELSE 'other' END AS scope,
        d.setconfig IS NULL AS config_is_null,
        (SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
           FROM unnest(d.setconfig) setting) AS config_hashes
 FROM deployment_observer_active o JOIN pg_db_role_setting d ON d.setrole=o.oid
), deployment_observer_external_routine_grants AS (
 SELECT jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
    CASE WHEN p.oid=to_regprocedure('pg_catalog.pg_control_system()')
          AND a.grantor=p.proowner AND owner.rolsuper
      THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
    CASE WHEN a.grantee=0 THEN jsonb_build_array('public') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
    a.privilege_type,a.is_grantable) AS record
 FROM deployment_observer_active o CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_roles owner ON owner.oid=p.proowner
 CROSS JOIN LATERAL aclexplode(p.proacl) a
 WHERE p.proowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)
), deployment_observer_boundary AS (
 SELECT CASE WHEN NOT (SELECT present FROM deployment_observer_local) THEN jsonb_build_object('present',false)
 ELSE jsonb_build_object(
   'present',true,
   'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,rolcanlogin,rolinherit,
       rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconnlimit,rolvaliduntil,
       rolconfig IS NULL,(SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
         FROM unnest(rolconfig) setting)) ORDER BY rolname) FROM deployment_observer_active),
   'memberships',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_memberships),
   'settings',(SELECT jsonb_agg(jsonb_build_array(scope,config_is_null,config_hashes)
       ORDER BY scope,config_is_null,config_hashes::text COLLATE "C") FROM deployment_observer_settings),
   'external_routine_grants',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_external_routine_grants),
   'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind)
       ORDER BY n.nspname,c.relname) FROM deployment_observer_active o JOIN pg_class c ON c.relowner=o.oid
       JOIN pg_namespace n ON n.oid=c.relnamespace),
   'owner_schemas',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o JOIN pg_namespace n ON n.nspowner=o.oid),
   'schema_create',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o CROSS JOIN pg_namespace n
       WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(o.oid,n.oid,'CREATE')),
   'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_default_acl d
       LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace CROSS JOIN LATERAL aclexplode(d.defaclacl) a
       WHERE d.defaclrole=o.oid OR a.grantee=o.oid OR a.grantor=o.oid),
   'external_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
       CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)),
   'external_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_attribute col JOIN pg_class c ON c.oid=col.attrelid
       JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(col.attacl) a
       WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid))
 ) END AS record
), deployment_boundary AS (
 SELECT jsonb_build_object(
   'builtin',(SELECT record FROM deployment_builtin),
   'observer',(SELECT record FROM deployment_observer_boundary),
   'startup_table_rights',(SELECT jsonb_agg(jsonb_build_array(name,privilege,allowed) ORDER BY name,privilege) FROM deployment_table_rights),
   'startup_column_rights',(SELECT jsonb_agg(jsonb_build_array(name,attname,privilege,allowed) ORDER BY name,attname,privilege) FROM deployment_column_rights),
   'startup_function_rights',(SELECT jsonb_agg(jsonb_build_array(nspname,proname,identity_arguments,allowed,grantable,expected_execute) ORDER BY nspname,proname,identity_arguments) FROM deployment_function_rights),
   'startup_mandatory_function_rights',(SELECT jsonb_agg(jsonb_build_array(identity,present,callable_definer,included,allowed,grantable) ORDER BY identity) FROM deployment_mandatory_functions),
   'startup_managed_database_overrides',(SELECT jsonb_agg(jsonb_build_array(key,override_count) ORDER BY key) FROM deployment_startup_database_overrides),
   'startup_final_rights_valid',
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=18 AND count(oid)=18 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=27 AND count(oid)=27 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=216 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=27 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin)
 ) AS record
), company_startup_rights AS (
 SELECT
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=61 AND count(oid)=61 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=70 AND count(oid)=70 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=560 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=65 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS (
 SELECT jsonb_build_object(
  'required_schemas',(SELECT jsonb_agg(jsonb_build_object(
    'name',required.name,'present',n.oid IS NOT NULL,
    'owner',CASE WHEN required.name='pg_catalog' AND n.nspowner=(SELECT proowner FROM deployment_builtin)
       AND owner.rolsuper THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',owner.rolname) END,
    'owner_superuser',owner.rolsuper,'acl_is_null',n.nspacl IS NULL,
    'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
       CASE WHEN required.name='pg_catalog' AND a.grantor=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
       CASE WHEN a.grantee=0 THEN jsonb_build_array('public')
         WHEN required.name='pg_catalog' AND a.grantee=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
       a.privilege_type,a.is_grantable) ORDER BY
       CASE WHEN a.grantor=n.nspowner THEN '' ELSE pg_get_userbyid(a.grantor) END,
       CASE WHEN a.grantee=n.nspowner THEN '' WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable)
      FROM aclexplode(COALESCE(n.nspacl,acldefault('n',n.nspowner))) a),'[]'::jsonb),
    'effective_rights',(SELECT jsonb_agg(jsonb_build_array(r.rolname,
       has_schema_privilege(r.oid,n.oid,'USAGE'),has_schema_privilege(r.oid,n.oid,'CREATE'),
       has_schema_privilege(r.oid,n.oid,'USAGE WITH GRANT OPTION'),
       has_schema_privilege(r.oid,n.oid,'CREATE WITH GRANT OPTION')) ORDER BY r.rolname)
      FROM (SELECT oid,rolname FROM protected_roles UNION SELECT oid,rolname FROM deployment_observer_active) r)
    ) ORDER BY required.name)
    FROM (VALUES ('public'),('ontology_api'),('ont_policy_api'),('leave_api'),('pg_catalog')) required(name)
    LEFT JOIN pg_namespace n ON n.nspname=required.name LEFT JOIN pg_roles owner ON owner.oid=n.nspowner),
  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),
  'legacy_root_boundary',(SELECT boundary FROM legacy_root_boundary),
  'tables',(SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records),
  'routines',(SELECT jsonb_agg(jsonb_build_object('metadata',record,'extra_valid',extra_valid)
      ORDER BY record->>'schema',record->>'name',record->>'identity_arguments') FROM routine_records),
  'foreign_keys',(SELECT jsonb_agg(record ORDER BY record->>'schema',record->>'source',record->>'name') FROM foreign_key_records),
  'constraint_flags',(SELECT jsonb_agg(jsonb_build_array(k.conrelid::regclass::text,k.conname,
      k.conenforced,k.conperiod,k.contypid=0,k.conparentid=0,k.coninhcount,k.conislocal)
      ORDER BY k.conrelid::regclass::text,k.conname)
    FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)),
  'trigger_links',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM (
    SELECT jsonb_build_array(t.tgrelid::regclass::text,
      CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgfoid::regprocedure::text,
      t.tgparentid=0,t.tgattr::text,t.tgoldtable,t.tgnewtable,
      k.conname,CASE WHEN k.conrelid IS NULL THEN NULL ELSE k.conrelid::regclass::text END,
      CASE WHEN t.tgconstrindid=0 THEN NULL ELSE t.tgconstrindid::regclass::text END) AS record
    FROM pg_trigger t LEFT JOIN pg_constraint k ON k.oid=t.tgconstraint
    WHERE t.tgrelid IN (SELECT oid FROM relations)) triggers),
  'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,
      CASE WHEN rolname='console_auth_rt' THEN NULL ELSE rolcanlogin END,
      rolinherit,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconfig) ORDER BY rolname) FROM protected_roles),
  'memberships',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
      pg_get_userbyid(m.grantor),m.admin_option,m.inherit_option,m.set_option)
      ORDER BY pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),pg_get_userbyid(m.grantor))
    FROM pg_auth_members m WHERE m.roleid IN (SELECT oid FROM protected_roles) OR m.member IN (SELECT oid FROM protected_roles)),
  'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner))
      ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
    WHERE c.relowner IN (SELECT oid FROM owner_roles) AND c.relkind IN ('r','p','v','m','S','f')),
  'owner_schemas',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname)
    FROM pg_namespace n WHERE n.nspowner IN (SELECT oid FROM owner_roles)),
  'schema_create',(SELECT jsonb_agg(jsonb_build_array(r.rolname,n.nspname) ORDER BY r.rolname,n.nspname)
    FROM owner_roles r CROSS JOIN pg_namespace n
    -- The current temporary namespace derives CREATE from database TEMP, not
    -- persistent schema authority. Keep every other namespace/ACL check exact.
    WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(r.oid,n.oid,'CREATE')),
  'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
      pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_default_acl d LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace
    CROSS JOIN LATERAL aclexplode(d.defaclacl) a
    WHERE d.defaclrole IN (SELECT oid FROM protected_roles) OR a.grantee IN (SELECT oid FROM owner_roles)),
  'external_owner_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL)),
  'external_owner_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_attribute col JOIN pg_class c ON c.oid=col.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace
    CROSS JOIN LATERAL aclexplode(col.attacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL))
 ) AS snapshot
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_policy_startup_rights_valid FROM snapshots
) captured;
 IF observed IS DISTINCT FROM expected_final OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_company_policy_v2.profile_mismatch'; END IF;
END
$native_policy_v2_custody$;
