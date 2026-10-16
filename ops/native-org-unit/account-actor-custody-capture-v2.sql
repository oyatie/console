-- EXECUTABLE UNREGISTERED V2 PRECURSOR FOR ADOPTED V3 TEST PREREQUISITES.
-- Proposed owning input: ops/native-org-unit/account-actor-custody-capture-v2.sql.
-- One read-only query; caller fixes search_path=pg_catalog,pg_temp and jit=off.
-- No installer, privileged inverse, phase registry, classifier or serving authority.
-- The three embedded historical bodies have only their final ;/LF removed and
-- one uniform snapshots AS MATERIALIZED insertion, with exact inverse checks.
WITH historical76_query AS MATERIALIZED (
 SELECT * FROM (
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
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('employees'),
 ('persons'),
 ('person_revisions'),
 ('employee_person_bindings'),
 ('ont_action_command_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employment_source_bindings'),
 ('employment_revisions'),
 ('leave_balance_import_receipts'),
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
 ('users'),
 ('org_units'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings')
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
 WHERE (n.nspname,p.proname) IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1'),('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))
 OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))
 OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN ('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')
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
     AND (SELECT count(*)=73 AND count(oid)=73 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=82 AND count(oid)=82 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=656 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=73 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS MATERIALIZED (
 SELECT jsonb_build_object(
  'native_directory_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND starts_with(c.relname,'native_people_')),
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
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_directory_startup_rights_valid FROM snapshots
 ) frozen_historical76
), historical83_query AS MATERIALIZED (
 SELECT * FROM (
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
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('employees'),
 ('persons'),
 ('person_revisions'),
 ('employee_person_bindings'),
 ('ont_action_command_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employment_source_bindings'),
 ('employment_revisions'),
 ('leave_balance_import_receipts'),
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
 ('users'),
 ('org_units'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_results_v1'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_versions_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1')
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
 WHERE (n.nspname,p.proname) IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1'),('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))
 OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))
 OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN ('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))
 OR (n.nspname,p.proname) IN (VALUES ('public','identity_native_group_process_execute_v1'),('public','identity_native_group_process_incarnation_selector_v1'),('public','identity_native_group_process_material_v1'),('public','identity_native_group_process_navigation_candidates_v1'),('public','identity_native_group_process_prepare_v1'),('public','native_group_process_accept_snapshot_v1'),('public','native_group_process_account_material_v1'),('public','native_group_process_action_roster_v1'),('public','native_group_process_assert_current_v1'),('public','native_group_process_assert_input_v1'),('public','native_group_process_assert_result_v1'),('public','native_group_process_audit_guard_v1'),('public','native_group_process_audit_truncate_guard_v1'),('public','native_group_process_classify_v1'),('public','native_group_process_command_guard_v1'),('public','native_group_process_complete_snapshot_v1'),('public','native_group_process_current_context_v1'),('public','native_group_process_decode_v1'),('public','native_group_process_deferred_guard_v1'),('public','native_group_process_effect_guard_v1'),('public','native_group_process_group_guard_v1'),('public','native_group_process_head_bytes_v1'),('public','native_group_process_head_ref_v1'),('public','native_group_process_head_statement_v1'),('public','native_group_process_i64_v1'),('public','native_group_process_immutable_statement_v1'),('public','native_group_process_input_guard_v1'),('public','native_group_process_micros_v1'),('public','native_group_process_original_material_v1'),('public','native_group_process_participant_guard_v1'),('public','native_group_process_policy_head_bytes_v1'),('public','native_group_process_policy_ref_v1'),('public','native_group_process_registered_actions_v1'),('public','native_group_process_registration_bytes_v1'),('public','native_group_process_result_bytes_v2'),('public','native_group_process_source_v1'),('public','native_group_process_text_v1'),('public','native_group_process_topology_fence_v1'),('public','native_group_process_uuid_v1'),('public','native_group_process_validate_frame_v1'),('public','native_group_process_version_bytes_v1'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')
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
     AND (SELECT count(*)=83 AND count(oid)=83 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=92 AND count(oid)=92 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=736 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=83 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS MATERIALIZED (
 SELECT jsonb_build_object(
  'native_group_process_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE starts_with(c.relname,'native_group_process') OR starts_with(c.relname,'native_group_identity_policy') OR starts_with(c.relname,'identity_native_group_process')),
  'native_group_process_schema_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname) FROM pg_namespace n WHERE starts_with(n.nspname,'native_group_process') OR starts_with(n.nspname,'native_group_identity_policy') OR starts_with(n.nspname,'identity_native_group_process')),
  'native_group_process_type_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,t.typname,t.typtype,pg_get_userbyid(t.typowner)) ORDER BY n.nspname,t.typname) FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace WHERE starts_with(t.typname,'native_group_process') OR starts_with(t.typname,'native_group_identity_policy') OR starts_with(t.typname,'identity_native_group_process')),
  'native_group_process_routine_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),p.prokind,pg_get_userbyid(p.proowner)) ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid)) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'native_group_process') OR starts_with(p.proname,'native_group_identity_policy') OR starts_with(p.proname,'identity_native_group_process')),
  'native_directory_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND starts_with(c.relname,'native_people_')),
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
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_group_process_startup_rights_valid FROM snapshots
 ) frozen_historical83
), original73_query AS MATERIALIZED (
 SELECT * FROM (
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
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('employees'),
 ('persons'),
 ('person_revisions'),
 ('employee_person_bindings'),
 ('ont_action_command_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employment_source_bindings'),
 ('employment_revisions'),
 ('leave_balance_import_receipts'),
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
 WHERE (n.nspname,p.proname) IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1'),('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))
 OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')
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
     AND (SELECT count(*)=73 AND count(oid)=73 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=82 AND count(oid)=82 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=656 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=73 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=61 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin) AS valid
), snapshots AS MATERIALIZED (
 SELECT jsonb_build_object(
  'native_directory_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND starts_with(c.relname,'native_people_')),
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
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,(SELECT valid FROM company_startup_rights) AS native_directory_startup_rights_valid FROM snapshots
 ) frozen_original73
), wanted(name) AS (VALUES
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
 ('native_people_inputs_v1'),
 ('native_people_terminals_v1'),
 ('employees'),
 ('persons'),
 ('person_revisions'),
 ('employee_person_bindings'),
 ('ont_action_command_receipts'),
 ('employee_employment_profiles'),
 ('employee_lifecycle_events'),
 ('employment_source_bindings'),
 ('employment_revisions'),
 ('leave_balance_import_receipts'),
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
 ('users'),
 ('org_units'),
 ('org_unit_revisions'),
 ('org_unit_source_bindings'),
 ('native_group_process_inputs_v1'),
 ('native_group_process_effects_v1'),
 ('native_group_process_results_v1'),
 ('native_group_identity_policy_heads_v1'),
 ('native_group_process_versions_v1'),
 ('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1')
), group_wanted(name) AS (VALUES
 ('native_group_process_inputs_v1'),('native_group_process_effects_v1'),
 ('native_group_process_results_v1'),('native_group_identity_policy_heads_v1'),
 ('native_group_process_versions_v1'),('native_group_process_head_revisions_v1'),
 ('native_group_process_heads_v1')
), selected_relations AS MATERIALIZED (
 SELECT c.* FROM wanted w JOIN pg_catalog.pg_namespace n ON n.nspname='public'
 JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), selected_attributes AS MATERIALIZED (
 SELECT a.* FROM pg_catalog.pg_attribute a JOIN selected_relations r ON r.oid=a.attrelid
 WHERE a.attnum<>0
), selected_defaults AS MATERIALIZED (
 SELECT d.* FROM pg_catalog.pg_attrdef d WHERE d.adrelid IN (SELECT oid FROM selected_relations)
), related_fks AS MATERIALIZED (
 SELECT k.* FROM pg_catalog.pg_constraint k WHERE k.contype='f'
 AND (k.conrelid IN (SELECT oid FROM selected_relations) OR k.confrelid IN (SELECT oid FROM selected_relations))
), selected_constraints AS MATERIALIZED (
 SELECT k.* FROM pg_catalog.pg_constraint k WHERE k.conrelid IN (SELECT oid FROM selected_relations)
 OR k.oid IN (SELECT oid FROM related_fks)
), selected_indexes AS MATERIALIZED (
 SELECT i.* FROM pg_catalog.pg_index i WHERE i.indrelid IN (SELECT oid FROM selected_relations)
 OR i.indexrelid IN (SELECT conindid FROM related_fks)
), selected_triggers AS MATERIALIZED (
 SELECT t.* FROM pg_catalog.pg_trigger t WHERE t.tgrelid IN (SELECT oid FROM selected_relations)
 OR t.tgconstraint IN (SELECT oid FROM related_fks)
), selected_rules AS MATERIALIZED (
 SELECT r.* FROM pg_catalog.pg_rewrite r WHERE r.ev_class IN (SELECT oid FROM selected_relations)
), selected_policies AS MATERIALIZED (
 SELECT p.* FROM pg_catalog.pg_policy p WHERE p.polrelid IN (SELECT oid FROM selected_relations)
), deployment_observer_role AS (
 SELECT * FROM pg_catalog.pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_catalog.pg_proc p
 CROSS JOIN LATERAL pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a
 WHERE p.oid=pg_catalog.to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
 OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a JOIN deployment_observer_role r
 ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), owner_roles AS (
 SELECT * FROM pg_catalog.pg_roles WHERE rolname IN
 ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')
), protected_roles AS (
 SELECT * FROM pg_catalog.pg_roles WHERE rolname IN
 ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup',
 'console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')
), frozen_ordinary_routines AS MATERIALIZED (
 SELECT p.* FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE
(n.nspname,p.proname) IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1'),('ontology_api','install_native_company_people_catalog_v1'),('public','native_company_people_manifest_v1'),('public','native_company_policy_assert_people_catalog_v1'),('public','native_company_policy_clause_v2'),('public','native_company_policy_codec_v2'),('public','native_company_policy_decode_v2'),('public','native_company_policy_form_v2'),('ontology_api','install_native_company_payroll_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v2'),('ontology_api','native_catalog_attribution_guard_v2'),('ontology_api','protected_audit_writer_guard_v2'),('ontology_api','require_current_transaction_audit_v2'),('public','company_enrollment_ontology_audit_guard_v2'),('public','identity_company_payroll_projection_v1'),('public','identity_company_projection_v2'),('public','identity_native_birth_closure_v2'),('public','identity_native_birth_row_guard_v2'),('public','identity_native_policy_material_v1'),('public','identity_native_root_guard_v2'),('public','native_company_catalog_birth_closure_v2'),('public','native_company_catalog_birth_row_guard_v2'),('public','native_company_policy_accept_snapshot_v1'),('public','native_company_policy_apply_assignment_v1'),('public','native_company_policy_assert_current_head_v1'),('public','native_company_policy_assert_effects_v1'),('public','native_company_policy_assert_ontology_audit_v1'),('public','native_company_policy_assert_payroll_catalog_v1'),('public','native_company_policy_assert_terminal_closure_v1'),('public','native_company_policy_audit_admit_v1'),('public','native_company_policy_audit_guard_v1'),('public','native_company_policy_business_clauses_v1'),('public','native_company_policy_capacity_v1'),('public','native_company_policy_clause_v1'),('public','native_company_policy_complete_snapshot_v1'),('public','native_company_policy_decode_v1'),('public','native_company_policy_effect_frame_v1'),('public','native_company_policy_execute_v1'),('public','native_company_policy_form_v1'),('public','native_company_policy_head_guard_v1'),('public','native_company_policy_immutable_v1'),('public','native_company_policy_input_closure_v1'),('public','native_company_policy_input_guard_v1'),('public','native_company_policy_manifest_v1'),('public','native_company_policy_ontology_snapshot_v1'),('public','native_company_policy_operation_check_v1'),('public','native_company_policy_participant_admit_v1'),('public','native_company_policy_participant_closed_v1'),('public','native_company_policy_participant_closure_v1'),('public','native_company_policy_participant_guard_v1'),('public','native_company_policy_participant_receipt_v1'),('public','native_company_policy_preflight_v1'),('public','native_company_policy_prepare_v1'),('public','native_company_policy_receipt_closure_v1'),('public','native_company_policy_receipt_guard_v1'),('public','native_company_policy_registration_custody_v1'),('public','native_company_policy_status_v1'),('ont_policy_api','attach_object_policy_rows'),('ont_policy_api','attach_object_policy_rows_core_v1'),('ont_policy_api','install_native_company_policy_v1'),('ontology_api','insert_children'),('ontology_api','install_builtin_catalog'),('ontology_api','install_builtin_catalog_core_v1'),('ontology_api','install_native_company_catalog_v1'),('ontology_api','lock_native_company_catalog_current_v1'),('ontology_api','native_catalog_attribution_guard_v1'),('ontology_api','prepare_legacy_object_type_write'),('ontology_api','protected_audit_writer_guard'),('ontology_api','require_current_transaction_audit'),('public','account_company_context_candidates_v1'),('public','account_company_native_rows_present_v1'),('public','account_context_presence_v1'),('public','account_legacy_topology_roots_lock_v1'),('public','auth_legacy_bootstrap_issue_v1'),('public','auth_legacy_bootstrap_issued_v1'),('public','auth_legacy_bootstrap_receipt_matches_v1'),('public','company_actor_entitlement_shape_v2'),('public','company_effect_binding_guard_v1'),('public','company_enrollment_assert_closure_v1'),('public','company_enrollment_audit_guard_v1'),('public','company_enrollment_audit_v1'),('public','company_enrollment_binding_v1'),('public','company_enrollment_cancel_v1'),('public','company_enrollment_catalog_binding_v1'),('public','company_enrollment_event_guard_v1'),('public','company_enrollment_execute_v1'),('public','company_enrollment_intake_closure_v1'),('public','company_enrollment_ontology_audit_guard_v1'),('public','company_enrollment_ontology_audit_v1'),('public','company_enrollment_prepare_v1'),('public','company_enrollment_receipt_intake_guard_v1'),('public','company_enrollment_request_guard_v1'),('public','company_enrollment_status_v1'),('public','company_enrollment_topology_v1'),('public','company_native_topology_birth_closure_v1'),('public','company_topology_history_immutable_v1'),('public','company_topology_truncate_guard_v1'),('public','company_topology_write_guard_v1'),('public','group_authority_lock_exclusive_v1'),('public','group_authority_lock_shared_v1'),('public','identity_company_actor_birth_guard_v1'),('public','identity_company_candidate_birth_guard_v1'),('public','identity_company_context_generation_guard_v1'),('public','identity_company_existing_catalog_closure_v1'),('public','identity_company_projection_v1'),('public','identity_enroll_company_administration_v1'),('public','identity_native_any_origin_v1'),('public','identity_native_birth_closure_v1'),('public','identity_native_birth_row_guard_v1'),('public','identity_native_immutable_v1'),('public','identity_native_legacy_child_guard_v1'),('public','identity_native_root_guard_v1'),('public','identity_native_truncate_guard_v1'),('public','native_company_catalog_birth_closure_v1'),('public','native_company_catalog_birth_row_guard_v1'),('public','native_company_catalog_immutable_v1'),('public','platform_assign_org_to_group'),('public','platform_attach_membership'),('public','platform_company_removal_cohort_v1'),('public','platform_create_organization_core_v1'),('public','platform_force_effect_admit_v1'),('public','platform_force_effect_closed_v1'),('public','platform_force_frame_closed_v1'),('public','platform_force_frame_guard_v1'),('public','platform_force_receipt_guard_v1'),('public','platform_force_remove_command_v1'),('public','platform_force_remove_decode_input_v1'),('public','platform_force_remove_direct_org_children'),('public','platform_force_remove_lock_plan_v1'),('public','platform_force_remove_plan_v1'),('public','platform_legacy_catalog_binding_closed_v1'),('public','platform_legacy_catalog_binding_write_guard_v1'),('public','platform_legacy_catalog_live_audit_v1'),('public','platform_legacy_catalog_receipt_audit_v1'),('public','platform_legacy_command_frame_closed_v1'),('public','platform_legacy_command_frame_guard_v1'),('public','platform_legacy_entity_effect_closed_v1'),('public','platform_legacy_grant_effect_closed_v1'),('public','platform_legacy_grant_write_guard_v1'),('public','platform_legacy_head_effect_closed_v1'),('public','platform_legacy_membership_binding_closed_v1'),('public','platform_legacy_membership_binding_v1'),('public','platform_legacy_membership_binding_write_guard_v1'),('public','platform_legacy_receipt_closed_v1'),('public','platform_legacy_topology_command_v1'),('public','platform_legacy_topology_decode_input_v1'),('public','platform_legacy_topology_lock_plan_v1'),('public','platform_legacy_topology_plan_v1'),('public','platform_legacy_topology_receipts_immutable_v1'),('public','platform_legacy_user_birth_capture_v1'),('public','platform_legacy_user_birth_witness_closed_v1'),('public','platform_legacy_user_birth_witness_guard_v1'),('public','platform_legacy_user_delete_guard_v1'),('public','platform_legacy_user_update_guard_v1'),('public','platform_mint_group_row')) OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN (SELECT oid FROM selected_relations) AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1','account_company_setup_eligibility_v1','account_session_shared_material_v1','auth_account_session_shared_material_v1','account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
 OR ((n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_')))
 OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))
 OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))
 OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN ('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))
 OR (n.nspname,p.proname) IN (VALUES ('public','identity_native_group_process_execute_v1'),('public','identity_native_group_process_incarnation_selector_v1'),('public','identity_native_group_process_material_v1'),('public','identity_native_group_process_navigation_candidates_v1'),('public','identity_native_group_process_prepare_v1'),('public','native_group_process_accept_snapshot_v1'),('public','native_group_process_account_material_v1'),('public','native_group_process_action_roster_v1'),('public','native_group_process_assert_current_v1'),('public','native_group_process_assert_input_v1'),('public','native_group_process_assert_result_v1'),('public','native_group_process_audit_guard_v1'),('public','native_group_process_audit_truncate_guard_v1'),('public','native_group_process_classify_v1'),('public','native_group_process_command_guard_v1'),('public','native_group_process_complete_snapshot_v1'),('public','native_group_process_current_context_v1'),('public','native_group_process_decode_v1'),('public','native_group_process_deferred_guard_v1'),('public','native_group_process_effect_guard_v1'),('public','native_group_process_group_guard_v1'),('public','native_group_process_head_bytes_v1'),('public','native_group_process_head_ref_v1'),('public','native_group_process_head_statement_v1'),('public','native_group_process_i64_v1'),('public','native_group_process_immutable_statement_v1'),('public','native_group_process_input_guard_v1'),('public','native_group_process_micros_v1'),('public','native_group_process_original_material_v1'),('public','native_group_process_participant_guard_v1'),('public','native_group_process_policy_head_bytes_v1'),('public','native_group_process_policy_ref_v1'),('public','native_group_process_registered_actions_v1'),('public','native_group_process_registration_bytes_v1'),('public','native_group_process_result_bytes_v2'),('public','native_group_process_source_v1'),('public','native_group_process_text_v1'),('public','native_group_process_topology_fence_v1'),('public','native_group_process_uuid_v1'),('public','native_group_process_validate_frame_v1'),('public','native_group_process_version_bytes_v1'))
), mandatory_identities(identity) AS (VALUES
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
   ('public.platform_update_group(uuid,text,text,text)')
), ordinary_routines AS MATERIALIZED (
 SELECT p.* FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE p.oid IN (SELECT oid FROM frozen_ordinary_routines)
 OR p.proowner IN (SELECT oid FROM owner_roles) OR (n.nspname='public' AND p.prosecdef)
 OR p.oid IN (SELECT pg_catalog.to_regprocedure(identity) FROM mandatory_identities)
 OR starts_with(p.proname,'native_group_process') OR starts_with(p.proname,'native_group_identity_policy')
 OR starts_with(p.proname,'identity_native_group_process')
), builtin_routines AS MATERIALIZED (
 SELECT p.* FROM pg_catalog.pg_proc p WHERE p.oid=pg_catalog.to_regprocedure('pg_catalog.pg_control_system()')
 OR p.oid IN (SELECT tgfoid FROM selected_triggers WHERE tgisinternal)
), selected_routines AS MATERIALIZED (
 SELECT p.* FROM pg_catalog.pg_proc p WHERE p.oid IN (SELECT oid FROM ordinary_routines)
 OR p.oid IN (SELECT oid FROM builtin_routines)
), required_schemas(name) AS (VALUES ('public'),('ontology_api'),('ont_policy_api'),('leave_api'),('pg_catalog')),
 selected_schemas AS MATERIALIZED (
 SELECT n.* FROM pg_catalog.pg_namespace n WHERE n.nspname IN (SELECT name FROM required_schemas)
), builtin_owner AS MATERIALIZED (
 SELECT r.* FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_roles r ON r.oid=p.proowner
 WHERE p.oid=pg_catalog.to_regprocedure('pg_catalog.pg_control_system()') AND r.rolsuper
), selected_roles AS MATERIALIZED (
 SELECT r.* FROM pg_catalog.pg_roles r WHERE r.oid IN (SELECT oid FROM protected_roles)
 OR r.oid IN (SELECT oid FROM owner_roles) OR r.oid IN (SELECT oid FROM deployment_observer_active)
 OR r.oid IN (SELECT oid FROM builtin_owner)
), row_types AS MATERIALIZED (
 SELECT t.* FROM pg_catalog.pg_type t WHERE t.oid IN (SELECT reltype FROM selected_relations)
 OR t.oid IN (SELECT x.typarray FROM pg_catalog.pg_type x WHERE x.oid IN (SELECT reltype FROM selected_relations))
), core_m(classid,objid,objsubid) AS MATERIALIZED (
 SELECT 'pg_catalog.pg_class'::regclass::oid,oid,0 FROM selected_relations
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,attrelid,attnum::integer FROM selected_attributes
 UNION SELECT 'pg_catalog.pg_attrdef'::regclass::oid,oid,0 FROM selected_defaults
 UNION SELECT 'pg_catalog.pg_constraint'::regclass::oid,oid,0 FROM selected_constraints
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,indexrelid,0 FROM selected_indexes
 UNION SELECT 'pg_catalog.pg_trigger'::regclass::oid,oid,0 FROM selected_triggers
 UNION SELECT 'pg_catalog.pg_rewrite'::regclass::oid,oid,0 FROM selected_rules
 UNION SELECT 'pg_catalog.pg_policy'::regclass::oid,oid,0 FROM selected_policies
 UNION SELECT 'pg_catalog.pg_proc'::regclass::oid,oid,0 FROM selected_routines
 UNION SELECT 'pg_catalog.pg_namespace'::regclass::oid,oid,0 FROM selected_schemas
 UNION SELECT 'pg_catalog.pg_authid'::regclass::oid,oid,0 FROM selected_roles
), used_type_ids(oid) AS MATERIALIZED (
 SELECT atttypid FROM selected_attributes WHERE atttypid<>0
 UNION SELECT prorettype FROM selected_routines WHERE prorettype<>0
 UNION SELECT unnest(proargtypes::oid[]) FROM selected_routines
 UNION SELECT unnest(proallargtypes) FROM selected_routines WHERE proallargtypes IS NOT NULL
 UNION SELECT provariadic FROM selected_routines WHERE provariadic<>0
 UNION SELECT oid FROM row_types
 UNION SELECT d.refobjid FROM pg_catalog.pg_depend d WHERE d.refclassid='pg_catalog.pg_type'::regclass
 AND EXISTS(SELECT 1 FROM core_m m WHERE (m.classid,m.objid,m.objsubid)=(d.classid,d.objid,d.objsubid))
 UNION SELECT t.oid FROM pg_catalog.pg_type t WHERE t.typname='native_org_unit' OR starts_with(t.typname,'native_org_unit_')
 OR starts_with(t.typname,'native_group_process') OR starts_with(t.typname,'native_group_identity_policy')
 OR starts_with(t.typname,'identity_native_group_process')
), used_collation_ids(oid) AS MATERIALIZED (
 SELECT attcollation FROM selected_attributes WHERE attcollation<>0
 UNION SELECT d.refobjid FROM pg_catalog.pg_depend d WHERE d.refclassid='pg_catalog.pg_collation'::regclass
 AND EXISTS(SELECT 1 FROM core_m m WHERE (m.classid,m.objid,m.objsubid)=(d.classid,d.objid,d.objsubid))
), metadata_m(classid,objid,objsubid) AS MATERIALIZED (
 SELECT * FROM core_m
 UNION SELECT 'pg_catalog.pg_type'::regclass::oid,oid,0 FROM used_type_ids
 UNION SELECT 'pg_catalog.pg_collation'::regclass::oid,oid,0 FROM used_collation_ids
), incoming_i(classid,objid,objsubid) AS MATERIALIZED (
 SELECT 'pg_catalog.pg_class'::regclass::oid,oid,0 FROM selected_relations
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,attrelid,attnum::integer FROM selected_attributes
 UNION SELECT 'pg_catalog.pg_attrdef'::regclass::oid,oid,0 FROM selected_defaults
 UNION SELECT 'pg_catalog.pg_constraint'::regclass::oid,oid,0 FROM selected_constraints
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,indexrelid,0 FROM selected_indexes
 UNION SELECT 'pg_catalog.pg_trigger'::regclass::oid,oid,0 FROM selected_triggers
 UNION SELECT 'pg_catalog.pg_rewrite'::regclass::oid,oid,0 FROM selected_rules
 UNION SELECT 'pg_catalog.pg_policy'::regclass::oid,oid,0 FROM selected_policies
 UNION SELECT 'pg_catalog.pg_proc'::regclass::oid,oid,0 FROM ordinary_routines
 WHERE oid NOT IN (SELECT oid FROM builtin_routines)
 UNION SELECT 'pg_catalog.pg_type'::regclass::oid,oid,0 FROM row_types
), ordinary_edges AS MATERIALIZED (
 SELECT d.* FROM pg_catalog.pg_depend d WHERE
 EXISTS(SELECT 1 FROM metadata_m m WHERE (m.classid,m.objid,m.objsubid)=(d.classid,d.objid,d.objsubid))
 OR EXISTS(SELECT 1 FROM incoming_i i WHERE (i.classid,i.objid,i.objsubid)=(d.refclassid,d.refobjid,d.refobjsubid))
 -- A malformed subobject of an otherwise relevant parent is refusal custody.
 OR (EXISTS(SELECT 1 FROM metadata_m m WHERE (m.classid,m.objid)=(d.classid,d.objid)) AND d.objsubid<>0
     AND (d.classid<>'pg_catalog.pg_class'::regclass OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_attribute a WHERE a.attrelid=d.objid AND a.attnum=d.objsubid)))
 OR (EXISTS(SELECT 1 FROM incoming_i i WHERE (i.classid,i.objid)=(d.refclassid,d.refobjid)) AND d.refobjsubid<>0
     AND (d.refclassid<>'pg_catalog.pg_class'::regclass OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_attribute a WHERE a.attrelid=d.refobjid AND a.attnum=d.refobjsubid)))
), current_database_identity AS (
 SELECT oid FROM pg_catalog.pg_database WHERE datname=pg_catalog.current_database()
), shared_edges AS MATERIALIZED (
 SELECT d.* FROM pg_catalog.pg_shdepend d WHERE
 (d.dbid=(SELECT oid FROM current_database_identity) AND d.classid<>'pg_catalog.pg_authid'::regclass
  OR d.dbid=0 AND d.classid='pg_catalog.pg_authid'::regclass)
 AND (EXISTS(SELECT 1 FROM metadata_m m WHERE (m.classid,m.objid,m.objsubid)=(d.classid,d.objid,d.objsubid))
 OR (EXISTS(SELECT 1 FROM metadata_m m WHERE (m.classid,m.objid)=(d.classid,d.objid)) AND d.objsubid<>0
     AND (d.classid<>'pg_catalog.pg_class'::regclass OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_attribute a WHERE a.attrelid=d.objid AND a.attnum=d.objsubid))))
), acl_role_ids(oid) AS (
 SELECT acl.grantor FROM selected_attributes a CROSS JOIN LATERAL pg_catalog.aclexplode(a.attacl) acl
 UNION SELECT acl.grantee FROM selected_attributes a CROSS JOIN LATERAL pg_catalog.aclexplode(a.attacl) acl WHERE acl.grantee<>0
), addresses(classid,objid,objsubid) AS MATERIALIZED (
 SELECT * FROM metadata_m
 UNION SELECT classid,objid,objsubid FROM ordinary_edges
 UNION SELECT refclassid,refobjid,refobjsubid FROM ordinary_edges
 UNION SELECT classid,objid,objsubid FROM shared_edges
 UNION SELECT refclassid,refobjid,0 FROM shared_edges
 UNION SELECT 'pg_catalog.pg_authid'::regclass::oid,oid,0 FROM acl_role_ids
 -- Links are resolution evidence only; they do not enlarge M or I.
 UNION SELECT 'pg_catalog.pg_constraint'::regclass::oid,tgconstraint,0 FROM selected_triggers WHERE tgisinternal AND tgconstraint<>0
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,tgrelid,0 FROM selected_triggers
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,tgconstrrelid,0 FROM selected_triggers WHERE tgconstrrelid<>0
 UNION SELECT 'pg_catalog.pg_class'::regclass::oid,tgconstrindid,0 FROM selected_triggers WHERE tgconstrindid<>0
 UNION SELECT 'pg_catalog.pg_proc'::regclass::oid,tgfoid,0 FROM selected_triggers
), type_format_inputs AS MATERIALIZED (
 SELECT t.*, CASE WHEN t.typelem<>0 AND t.typsubscript=pg_catalog.to_regprocedure('pg_catalog.array_subscript_handler(internal)')
 AND t.typstorage<>'p' THEN t.typelem ELSE t.oid END AS base_oid,
 t.typelem<>0 AND t.typsubscript=pg_catalog.to_regprocedure('pg_catalog.array_subscript_handler(internal)')
 AND t.typstorage<>'p' AS is_array
 FROM pg_catalog.pg_type t
), type_formats AS MATERIALIZED (
 -- Mirror PG18 FORCE_QUALIFY without a permission-sensitive inverse parser.
 SELECT t.oid, CASE b.oid
 WHEN 'pg_catalog.bit'::regtype THEN 'bit' WHEN 'pg_catalog.bool'::regtype THEN 'boolean'
 WHEN 'pg_catalog.bpchar'::regtype THEN 'character' WHEN 'pg_catalog.float4'::regtype THEN 'real'
 WHEN 'pg_catalog.float8'::regtype THEN 'double precision' WHEN 'pg_catalog.int2'::regtype THEN 'smallint'
 WHEN 'pg_catalog.int4'::regtype THEN 'integer' WHEN 'pg_catalog.int8'::regtype THEN 'bigint'
 WHEN 'pg_catalog.numeric'::regtype THEN 'numeric' WHEN 'pg_catalog.interval'::regtype THEN 'interval'
 WHEN 'pg_catalog.time'::regtype THEN 'time without time zone' WHEN 'pg_catalog.timetz'::regtype THEN 'time with time zone'
 WHEN 'pg_catalog.timestamp'::regtype THEN 'timestamp without time zone' WHEN 'pg_catalog.timestamptz'::regtype THEN 'timestamp with time zone'
 WHEN 'pg_catalog.varbit'::regtype THEN 'bit varying' WHEN 'pg_catalog.varchar'::regtype THEN 'character varying'
 WHEN 'pg_catalog.json'::regtype THEN 'json'
 ELSE pg_catalog.quote_ident(n.nspname)||'.'||pg_catalog.quote_ident(b.typname) END
 ||CASE WHEN t.is_array THEN '[]' ELSE '' END AS formatted,
 -- Forward array spelling must retain the actual persistent canonical array.
 b.oid IS NOT NULL AND n.oid IS NOT NULL AND tn.oid IS NOT NULL AND b.typisdefined AND t.typisdefined
 AND n.nspname NOT IN ('pg_temp','pg_toast') AND NOT starts_with(n.nspname,'pg_temp_') AND NOT starts_with(n.nspname,'pg_toast_temp_')
 AND tn.nspname NOT IN ('pg_temp','pg_toast') AND NOT starts_with(tn.nspname,'pg_temp_') AND NOT starts_with(tn.nspname,'pg_toast_temp_')
 AND (NOT t.is_array OR (t.typelem=b.oid AND b.typarray=t.oid)) AS valid
 FROM type_format_inputs t LEFT JOIN pg_catalog.pg_type b ON b.oid=t.base_oid
 LEFT JOIN pg_catalog.pg_namespace n ON n.oid=b.typnamespace
 LEFT JOIN pg_catalog.pg_namespace tn ON tn.oid=t.typnamespace
), native_addresses AS MATERIALIZED (
 SELECT a.*, ident.type AS native_type,ident.object_names AS native_names,ident.object_args AS native_args,
 jsonb_build_object('type',ident.type,'object_names',to_jsonb(ident.object_names),'object_args',to_jsonb(ident.object_args)) AS native_address
 FROM addresses a CROSS JOIN LATERAL pg_catalog.pg_identify_object_as_address(a.classid,a.objid,a.objsubid) ident
), reconstruction AS MATERIALIZED (
 SELECT a.*, CASE a.classid
 WHEN 'pg_catalog.pg_class'::regclass THEN
   CASE c.relkind WHEN 'r' THEN 'table' WHEN 'p' THEN 'table' WHEN 'i' THEN 'index' WHEN 'I' THEN 'index'
   WHEN 'v' THEN 'view' WHEN 'm' THEN 'materialized view' WHEN 'S' THEN 'sequence' WHEN 'f' THEN 'foreign table'
   WHEN 'c' THEN CASE WHEN a.objsubid>0 THEN 'composite type' END END
   ||CASE WHEN a.objsubid<>0 THEN ' column' ELSE '' END
 WHEN 'pg_catalog.pg_proc'::regclass THEN CASE p.prokind WHEN 'p' THEN 'procedure' WHEN 'a' THEN 'aggregate' ELSE 'function' END
 WHEN 'pg_catalog.pg_type'::regclass THEN 'type' WHEN 'pg_catalog.pg_collation'::regclass THEN 'collation'
 WHEN 'pg_catalog.pg_attrdef'::regclass THEN 'default value'
 WHEN 'pg_catalog.pg_constraint'::regclass THEN CASE WHEN k.conrelid<>0 THEN 'table constraint' WHEN k.contypid<>0 THEN 'domain constraint' END
 WHEN 'pg_catalog.pg_trigger'::regclass THEN 'trigger' WHEN 'pg_catalog.pg_rewrite'::regclass THEN 'rule'
 WHEN 'pg_catalog.pg_policy'::regclass THEN 'policy' WHEN 'pg_catalog.pg_namespace'::regclass THEN 'schema'
 WHEN 'pg_catalog.pg_authid'::regclass THEN 'role' WHEN 'pg_catalog.pg_tablespace'::regclass THEN 'tablespace'
 WHEN 'pg_catalog.pg_language'::regclass THEN 'language' WHEN 'pg_catalog.pg_extension'::regclass THEN 'extension'
 WHEN 'pg_catalog.pg_am'::regclass THEN 'access method' WHEN 'pg_catalog.pg_opclass'::regclass THEN 'operator class'
 WHEN 'pg_catalog.pg_opfamily'::regclass THEN 'operator family' WHEN 'pg_catalog.pg_operator'::regclass THEN 'operator'
 END AS expected_type,
 CASE a.classid
 WHEN 'pg_catalog.pg_class'::regclass THEN CASE WHEN a.objsubid=0 THEN ARRAY[cn.nspname::text,c.relname::text] ELSE ARRAY[cn.nspname::text,c.relname::text,ca.attname::text] END
 WHEN 'pg_catalog.pg_proc'::regclass THEN ARRAY[pn.nspname::text,p.proname::text]
 WHEN 'pg_catalog.pg_type'::regclass THEN ARRAY[tf.formatted]
 WHEN 'pg_catalog.pg_collation'::regclass THEN ARRAY[con.nspname::text,co.collname::text]
 WHEN 'pg_catalog.pg_attrdef'::regclass THEN ARRAY[dn.nspname::text,dc.relname::text,da.attname::text]
 WHEN 'pg_catalog.pg_constraint'::regclass THEN CASE WHEN k.conrelid<>0 THEN ARRAY[kn.nspname::text,kc.relname::text,k.conname::text] WHEN k.contypid<>0 THEN ARRAY[ktf.formatted] END
 WHEN 'pg_catalog.pg_trigger'::regclass THEN ARRAY[tn.nspname::text,tc.relname::text,t.tgname::text]
 WHEN 'pg_catalog.pg_rewrite'::regclass THEN ARRAY[rn.nspname::text,rc.relname::text,r.rulename::text]
 WHEN 'pg_catalog.pg_policy'::regclass THEN ARRAY[poln.nspname::text,polc.relname::text,pol.polname::text]
 WHEN 'pg_catalog.pg_namespace'::regclass THEN ARRAY[ns.nspname::text]
 WHEN 'pg_catalog.pg_authid'::regclass THEN ARRAY[role.rolname::text]
 WHEN 'pg_catalog.pg_tablespace'::regclass THEN ARRAY[spc.spcname::text]
 WHEN 'pg_catalog.pg_language'::regclass THEN ARRAY[lang.lanname::text]
 WHEN 'pg_catalog.pg_extension'::regclass THEN ARRAY[ext.extname::text]
 WHEN 'pg_catalog.pg_am'::regclass THEN ARRAY[am.amname::text]
 WHEN 'pg_catalog.pg_opclass'::regclass THEN ARRAY[opcam.amname::text,opcn.nspname::text,opc.opcname::text]
 WHEN 'pg_catalog.pg_opfamily'::regclass THEN ARRAY[opfam.amname::text,opfn.nspname::text,opf.opfname::text]
 WHEN 'pg_catalog.pg_operator'::regclass THEN ARRAY[opn.nspname::text,op.oprname::text]
 END AS expected_names,
 CASE a.classid WHEN 'pg_catalog.pg_proc'::regclass THEN ARRAY(
   SELECT f.formatted FROM unnest(p.proargtypes::oid[]) WITH ORDINALITY arg(oid,ordinal)
   LEFT JOIN type_formats f ON f.oid=arg.oid ORDER BY arg.ordinal)
 WHEN 'pg_catalog.pg_constraint'::regclass THEN CASE WHEN k.conrelid=0 AND k.contypid<>0 THEN ARRAY[k.conname::text] ELSE ARRAY[]::text[] END
 WHEN 'pg_catalog.pg_operator'::regclass THEN ARRAY[olf.formatted,orf.formatted]
 ELSE ARRAY[]::text[] END AS expected_args,
 CASE a.classid
 WHEN 'pg_catalog.pg_class'::regclass THEN cn.nspname
 WHEN 'pg_catalog.pg_proc'::regclass THEN pn.nspname
 WHEN 'pg_catalog.pg_collation'::regclass THEN con.nspname
 WHEN 'pg_catalog.pg_attrdef'::regclass THEN dn.nspname
 WHEN 'pg_catalog.pg_constraint'::regclass THEN CASE WHEN k.conrelid<>0 THEN kn.nspname ELSE ktn.nspname END
 WHEN 'pg_catalog.pg_trigger'::regclass THEN tn.nspname
 WHEN 'pg_catalog.pg_rewrite'::regclass THEN rn.nspname
 WHEN 'pg_catalog.pg_policy'::regclass THEN poln.nspname
 WHEN 'pg_catalog.pg_namespace'::regclass THEN ns.nspname
 WHEN 'pg_catalog.pg_opclass'::regclass THEN opcn.nspname
 WHEN 'pg_catalog.pg_opfamily'::regclass THEN opfn.nspname
 WHEN 'pg_catalog.pg_operator'::regclass THEN opn.nspname
 END AS association_namespace,
 CASE a.classid
 WHEN 'pg_catalog.pg_class'::regclass THEN c.oid IS NOT NULL AND cn.oid IS NOT NULL
   AND NOT starts_with(cn.nspname,'pg_temp_') AND NOT starts_with(cn.nspname,'pg_toast_temp_')
   AND (a.objsubid=0 OR (c.relkind IN ('r','p','f')
        OR a.objsubid>0 AND c.relkind IN ('v','m','c','S'))
        AND ca.attrelid=c.oid AND ca.attnum=a.objsubid AND NOT ca.attisdropped AND ca.atttypid<>0)
 WHEN 'pg_catalog.pg_proc'::regclass THEN a.objsubid=0 AND p.oid IS NOT NULL AND pn.oid IS NOT NULL AND p.prokind IN ('f','p','a','w')
   AND NOT EXISTS(SELECT 1 FROM unnest(p.proargtypes::oid[]) arg(oid) LEFT JOIN type_formats f ON f.oid=arg.oid WHERE f.valid IS NOT TRUE)
 WHEN 'pg_catalog.pg_type'::regclass THEN a.objsubid=0 AND tf.valid IS TRUE
 WHEN 'pg_catalog.pg_collation'::regclass THEN a.objsubid=0 AND co.oid IS NOT NULL AND con.oid IS NOT NULL
 WHEN 'pg_catalog.pg_attrdef'::regclass THEN a.objsubid=0 AND d.oid IS NOT NULL AND dc.oid IS NOT NULL AND dn.oid IS NOT NULL
   AND da.attnum=d.adnum AND da.attnum>0 AND NOT da.attisdropped AND da.atthasdef
 WHEN 'pg_catalog.pg_constraint'::regclass THEN a.objsubid=0 AND k.oid IS NOT NULL
   AND (k.conrelid<>0 AND k.contypid=0 AND kc.oid IS NOT NULL AND kn.oid=k.connamespace
        OR k.conrelid=0 AND k.contypid<>0 AND ktf.valid IS TRUE)
 WHEN 'pg_catalog.pg_trigger'::regclass THEN a.objsubid=0 AND t.oid IS NOT NULL AND tc.oid IS NOT NULL AND tn.oid IS NOT NULL
 WHEN 'pg_catalog.pg_rewrite'::regclass THEN a.objsubid=0 AND r.oid IS NOT NULL AND rc.oid IS NOT NULL AND rn.oid IS NOT NULL
 WHEN 'pg_catalog.pg_policy'::regclass THEN a.objsubid=0 AND pol.oid IS NOT NULL AND polc.oid IS NOT NULL AND poln.oid IS NOT NULL
 WHEN 'pg_catalog.pg_namespace'::regclass THEN a.objsubid=0 AND ns.oid IS NOT NULL AND NOT starts_with(ns.nspname,'pg_temp_') AND NOT starts_with(ns.nspname,'pg_toast_temp_')
 WHEN 'pg_catalog.pg_authid'::regclass THEN a.objsubid=0 AND role.oid IS NOT NULL
 WHEN 'pg_catalog.pg_tablespace'::regclass THEN a.objsubid=0 AND spc.oid IS NOT NULL
 WHEN 'pg_catalog.pg_language'::regclass THEN a.objsubid=0 AND lang.oid IS NOT NULL
 WHEN 'pg_catalog.pg_extension'::regclass THEN a.objsubid=0 AND ext.oid IS NOT NULL
 WHEN 'pg_catalog.pg_am'::regclass THEN a.objsubid=0 AND am.oid IS NOT NULL
 WHEN 'pg_catalog.pg_opclass'::regclass THEN a.objsubid=0 AND opc.oid IS NOT NULL AND opcam.oid IS NOT NULL AND opcn.oid IS NOT NULL
 WHEN 'pg_catalog.pg_opfamily'::regclass THEN a.objsubid=0 AND opf.oid IS NOT NULL AND opfam.oid IS NOT NULL AND opfn.oid IS NOT NULL
 WHEN 'pg_catalog.pg_operator'::regclass THEN a.objsubid=0 AND op.oid IS NOT NULL AND opn.oid IS NOT NULL
   AND op.oprleft<>0 AND op.oprright<>0 AND olf.valid IS TRUE AND orf.valid IS TRUE
 ELSE FALSE END AS catalog_valid
 FROM native_addresses a
 LEFT JOIN pg_catalog.pg_class c ON a.classid='pg_catalog.pg_class'::regclass AND c.oid=a.objid
 LEFT JOIN pg_catalog.pg_namespace cn ON cn.oid=c.relnamespace
 LEFT JOIN pg_catalog.pg_attribute ca ON ca.attrelid=c.oid AND ca.attnum=a.objsubid AND a.objsubid<>0
 LEFT JOIN pg_catalog.pg_proc p ON a.classid='pg_catalog.pg_proc'::regclass AND p.oid=a.objid
 LEFT JOIN pg_catalog.pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN type_formats tf ON a.classid='pg_catalog.pg_type'::regclass AND tf.oid=a.objid
 LEFT JOIN pg_catalog.pg_collation co ON a.classid='pg_catalog.pg_collation'::regclass AND co.oid=a.objid
 LEFT JOIN pg_catalog.pg_namespace con ON con.oid=co.collnamespace
 LEFT JOIN pg_catalog.pg_attrdef d ON a.classid='pg_catalog.pg_attrdef'::regclass AND d.oid=a.objid
 LEFT JOIN pg_catalog.pg_class dc ON dc.oid=d.adrelid LEFT JOIN pg_catalog.pg_namespace dn ON dn.oid=dc.relnamespace
 LEFT JOIN pg_catalog.pg_attribute da ON da.attrelid=d.adrelid AND da.attnum=d.adnum
 LEFT JOIN pg_catalog.pg_constraint k ON a.classid='pg_catalog.pg_constraint'::regclass AND k.oid=a.objid
 LEFT JOIN pg_catalog.pg_class kc ON kc.oid=k.conrelid LEFT JOIN pg_catalog.pg_namespace kn ON kn.oid=kc.relnamespace
 LEFT JOIN type_formats ktf ON ktf.oid=k.contypid
 LEFT JOIN pg_catalog.pg_type kt ON kt.oid=k.contypid LEFT JOIN pg_catalog.pg_namespace ktn ON ktn.oid=kt.typnamespace
 LEFT JOIN pg_catalog.pg_trigger t ON a.classid='pg_catalog.pg_trigger'::regclass AND t.oid=a.objid
 LEFT JOIN pg_catalog.pg_class tc ON tc.oid=t.tgrelid LEFT JOIN pg_catalog.pg_namespace tn ON tn.oid=tc.relnamespace
 LEFT JOIN pg_catalog.pg_rewrite r ON a.classid='pg_catalog.pg_rewrite'::regclass AND r.oid=a.objid
 LEFT JOIN pg_catalog.pg_class rc ON rc.oid=r.ev_class LEFT JOIN pg_catalog.pg_namespace rn ON rn.oid=rc.relnamespace
 LEFT JOIN pg_catalog.pg_policy pol ON a.classid='pg_catalog.pg_policy'::regclass AND pol.oid=a.objid
 LEFT JOIN pg_catalog.pg_class polc ON polc.oid=pol.polrelid LEFT JOIN pg_catalog.pg_namespace poln ON poln.oid=polc.relnamespace
 LEFT JOIN pg_catalog.pg_namespace ns ON a.classid='pg_catalog.pg_namespace'::regclass AND ns.oid=a.objid
 LEFT JOIN pg_catalog.pg_roles role ON a.classid='pg_catalog.pg_authid'::regclass AND role.oid=a.objid
 LEFT JOIN pg_catalog.pg_tablespace spc ON a.classid='pg_catalog.pg_tablespace'::regclass AND spc.oid=a.objid
 LEFT JOIN pg_catalog.pg_language lang ON a.classid='pg_catalog.pg_language'::regclass AND lang.oid=a.objid
 LEFT JOIN pg_catalog.pg_extension ext ON a.classid='pg_catalog.pg_extension'::regclass AND ext.oid=a.objid
 LEFT JOIN pg_catalog.pg_am am ON a.classid='pg_catalog.pg_am'::regclass AND am.oid=a.objid
 LEFT JOIN pg_catalog.pg_opclass opc ON a.classid='pg_catalog.pg_opclass'::regclass AND opc.oid=a.objid
 LEFT JOIN pg_catalog.pg_am opcam ON opcam.oid=opc.opcmethod LEFT JOIN pg_catalog.pg_namespace opcn ON opcn.oid=opc.opcnamespace
 LEFT JOIN pg_catalog.pg_opfamily opf ON a.classid='pg_catalog.pg_opfamily'::regclass AND opf.oid=a.objid
 LEFT JOIN pg_catalog.pg_am opfam ON opfam.oid=opf.opfmethod LEFT JOIN pg_catalog.pg_namespace opfn ON opfn.oid=opf.opfnamespace
 LEFT JOIN pg_catalog.pg_operator op ON a.classid='pg_catalog.pg_operator'::regclass AND op.oid=a.objid
 LEFT JOIN pg_catalog.pg_namespace opn ON opn.oid=op.oprnamespace
 LEFT JOIN type_formats olf ON olf.oid=op.oprleft LEFT JOIN type_formats orf ON orf.oid=op.oprright
), checked_native AS MATERIALIZED (
 SELECT a.*, (catalog_valid IS TRUE AND native_type=expected_type AND native_names=expected_names AND native_args=expected_args
 AND native_type IS NOT NULL AND native_names IS NOT NULL AND native_args IS NOT NULL
 AND (association_namespace IS NULL OR association_namespace NOT IN ('pg_temp','pg_toast')
 AND NOT starts_with(association_namespace,'pg_temp_') AND NOT starts_with(association_namespace,'pg_toast_temp_'))
 AND NOT EXISTS(SELECT 1 FROM unnest(native_names||native_args) part WHERE part IS NULL)) IS TRUE AS native_valid
 FROM reconstruction a
), checked_constraint_addresses AS MATERIALIZED (
 SELECT * FROM checked_native
 WHERE classid='pg_catalog.pg_constraint'::regclass AND objsubid=0
), checked_relation_addresses AS MATERIALIZED (
 SELECT * FROM checked_native
 WHERE classid='pg_catalog.pg_class'::regclass AND objsubid=0
), checked_routine_addresses AS MATERIALIZED (
 SELECT * FROM checked_native
 WHERE classid='pg_catalog.pg_proc'::regclass AND objsubid=0
), ri_maps AS MATERIALIZED (
 SELECT a.classid,a.objid,a.objsubid,
 jsonb_build_object('type','foreign key RI trigger','foreign_key',fk.native_address,'on',onrel.native_address,
 'routine',routine.native_address,'trigger_type',t.tgtype,'other',otherrel.native_address,'index',idx.native_address) AS semantic_address,
 (t.tgisinternal AND k.contype='f' AND k.conrelid<>0 AND k.confrelid<>0 AND k.conindid<>0
 AND t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$' AND t.tgenabled='O'
 AND t.tgconstraint=k.oid AND t.tgconstrindid=k.conindid AND t.tgparentid=0
 AND t.tgnargs=0 AND t.tgargs='\x'::bytea AND cardinality(t.tgattr::smallint[])=0
 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL AND t.tgqual IS NULL
 AND fk.native_valid AND onrel.native_valid AND otherrel.native_valid AND idx.native_valid AND routine.native_valid
 AND pn.nspname='pg_catalog' AND p.prokind='f' AND p.pronargs=0 AND p.prorettype='pg_catalog.trigger'::regtype
 AND i.indexrelid=k.conindid AND i.indrelid=k.confrelid AND i.indisvalid AND i.indisready AND i.indislive AND i.indisunique
 AND (
   p.proname IN ('RI_FKey_check_ins','RI_FKey_check_upd') AND t.tgrelid=k.conrelid AND t.tgconstrrelid=k.confrelid
   AND t.tgtype=CASE p.proname WHEN 'RI_FKey_check_ins' THEN 5 ELSE 17 END
   AND t.tgdeferrable=k.condeferrable AND t.tginitdeferred=k.condeferred
   OR t.tgrelid=k.confrelid AND t.tgconstrrelid=k.conrelid AND t.tgtype IN (9,17)
   AND p.proname=('RI_FKey_'||CASE CASE WHEN t.tgtype=9 THEN k.confdeltype ELSE k.confupdtype END
     WHEN 'a' THEN 'noaction' WHEN 'r' THEN 'restrict' WHEN 'c' THEN 'cascade' WHEN 'n' THEN 'setnull' WHEN 'd' THEN 'setdefault' END
     ||CASE WHEN t.tgtype=9 THEN '_del' ELSE '_upd' END)
   AND t.tgdeferrable=(k.condeferrable AND CASE WHEN t.tgtype=9 THEN k.confdeltype ELSE k.confupdtype END='a')
   AND t.tginitdeferred=(k.condeferred AND CASE WHEN t.tgtype=9 THEN k.confdeltype ELSE k.confupdtype END='a')
 )) IS TRUE AS ri_valid,
 jsonb_build_object('internal',t.tgisinternal,'enabled',t.tgenabled,'deferrable',t.tgdeferrable,
 'initially_deferred',t.tginitdeferred,'arguments',t.tgnargs,'argument_bytes',encode(t.tgargs,'hex'),
 'attributes',to_jsonb(t.tgattr::smallint[]),'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,
 'new_table',t.tgnewtable,'generated_name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$') AS flags
 FROM checked_native a JOIN pg_catalog.pg_trigger t ON a.classid='pg_catalog.pg_trigger'::regclass AND t.oid=a.objid AND t.tgisinternal
 LEFT JOIN pg_catalog.pg_constraint k ON k.oid=t.tgconstraint
 LEFT JOIN pg_catalog.pg_index i ON i.indexrelid=k.conindid
 LEFT JOIN pg_catalog.pg_proc p ON p.oid=t.tgfoid LEFT JOIN pg_catalog.pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN checked_constraint_addresses fk ON (fk.classid,fk.objid,fk.objsubid)=('pg_catalog.pg_constraint'::regclass::oid,k.oid,0)
 LEFT JOIN checked_relation_addresses onrel ON (onrel.classid,onrel.objid,onrel.objsubid)=('pg_catalog.pg_class'::regclass::oid,t.tgrelid,0)
 LEFT JOIN checked_relation_addresses otherrel ON (otherrel.classid,otherrel.objid,otherrel.objsubid)=('pg_catalog.pg_class'::regclass::oid,t.tgconstrrelid,0)
 LEFT JOIN checked_relation_addresses idx ON (idx.classid,idx.objid,idx.objsubid)=('pg_catalog.pg_class'::regclass::oid,t.tgconstrindid,0)
 LEFT JOIN checked_routine_addresses routine ON (routine.classid,routine.objid,routine.objsubid)=('pg_catalog.pg_proc'::regclass::oid,t.tgfoid,0)
), stable_addresses AS MATERIALIZED (
 SELECT a.classid,a.objid,a.objsubid,
 CASE WHEN a.native_valid IS NOT TRUE THEN NULL
 WHEN t.tgisinternal THEN CASE WHEN ri.ri_valid IS TRUE THEN ri.semantic_address ELSE NULL END
 WHEN a.classid='pg_catalog.pg_authid'::regclass AND a.objid IN (SELECT oid FROM builtin_owner) AND a.objsubid=0
 THEN jsonb_build_object('type','role','object_names',jsonb_build_array('builtin_owner'),'object_args','[]'::jsonb)
 ELSE a.native_address END AS address,
 (a.native_valid AND (t.oid IS NULL OR NOT t.tgisinternal OR ri.ri_valid IS TRUE)) IS TRUE AS valid,
 CASE WHEN t.tgisinternal AND a.native_valid IS TRUE AND ri.ri_valid IS TRUE THEN ri.flags ELSE NULL END AS ri_flags
 FROM checked_native a LEFT JOIN pg_catalog.pg_trigger t ON a.classid='pg_catalog.pg_trigger'::regclass AND t.oid=a.objid
 LEFT JOIN ri_maps ri ON (ri.classid,ri.objid,ri.objsubid)=(a.classid,a.objid,a.objsubid)
), stable_role_addresses AS MATERIALIZED (
 SELECT * FROM stable_addresses
 WHERE classid='pg_catalog.pg_authid'::regclass AND objsubid=0
), row_type_valid AS (
 SELECT (count(*)=(SELECT count(*) FROM selected_relations) AND bool_and(
 r.reltype<>0 AND t.oid=r.reltype AND t.typrelid=r.oid AND t.typtype='c' AND t.typarray<>0
 AND ar.oid=t.typarray AND ar.typelem=t.oid AND ar.typrelid=0
 AND r.relnamespace=t.typnamespace AND r.relname=t.typname AND ar.typnamespace=t.typnamespace)) IS TRUE AS valid
 FROM selected_relations r LEFT JOIN pg_catalog.pg_type t ON t.oid=r.reltype LEFT JOIN pg_catalog.pg_type ar ON ar.oid=t.typarray
), resolution AS (
 SELECT ((SELECT count(*) FROM addresses)>0 AND (SELECT count(*) FROM stable_addresses)=(SELECT count(*) FROM addresses)
 AND (SELECT count(DISTINCT (classid,objid,objsubid)) FROM stable_addresses)=(SELECT count(*) FROM addresses)
 AND (SELECT count(DISTINCT address) FROM stable_addresses)=(SELECT count(*) FROM addresses)
 AND (SELECT bool_and(valid AND address IS NOT NULL) FROM stable_addresses) IS TRUE
 AND (SELECT valid FROM row_type_valid) IS TRUE
 AND (SELECT count(*) FROM current_database_identity)=1
 AND NOT EXISTS(SELECT 1 FROM related_fks k WHERE (SELECT count(*) FROM pg_catalog.pg_trigger t WHERE t.tgconstraint=k.oid)<>4)
 AND NOT EXISTS(SELECT 1 FROM used_type_ids u LEFT JOIN pg_catalog.pg_type t ON t.oid=u.oid WHERE t.oid IS NULL)
 AND NOT EXISTS(SELECT 1 FROM used_collation_ids u LEFT JOIN pg_catalog.pg_collation c ON c.oid=u.oid WHERE c.oid IS NULL)) IS TRUE AS valid
), attribute_records AS MATERIALIZED (
 SELECT col.address AS identity,jsonb_build_object('relation',rel.address,'column',col.address,'attnum',a.attnum,'name',a.attname,
 'type',typ.address,'typmod',a.atttypmod,'collation',coll.address,'collation_present',a.attcollation<>0,
 'length',a.attlen,'by_value',a.attbyval,'alignment',a.attalign,'storage',a.attstorage,'compression',a.attcompression,
 'dimensions',a.attndims,'statistics_target',a.attstattarget,'not_null',a.attnotnull,'has_default',a.atthasdef,
 'has_missing',a.atthasmissing,'identity',a.attidentity,'generated',a.attgenerated,'dropped',a.attisdropped,
 'local',a.attislocal,'inheritance_count',a.attinhcount,'missing_value',CASE WHEN a.attmissingval IS NULL THEN NULL
 ELSE jsonb_build_object('type',typ.address,'value',to_jsonb(a.attmissingval)) END,
 'acl_is_null',a.attacl IS NULL,'acl',COALESCE((SELECT jsonb_agg(jsonb_build_object('grantor',grantor.address,
 'grantee',CASE WHEN acl.grantee=0 THEN jsonb_build_object('type','ACL PUBLIC sentinel') ELSE grantee.address END,
 'privilege',acl.privilege_type,'grantable',acl.is_grantable) ORDER BY grantor.address::text COLLATE "C",
 CASE WHEN acl.grantee=0 THEN jsonb_build_object('type','ACL PUBLIC sentinel') ELSE grantee.address END::text COLLATE "C",
 acl.privilege_type COLLATE "C",acl.is_grantable)
 FROM pg_catalog.aclexplode(a.attacl) acl
 LEFT JOIN stable_role_addresses grantor ON (grantor.classid,grantor.objid,grantor.objsubid)=('pg_catalog.pg_authid'::regclass::oid,acl.grantor,0)
 LEFT JOIN stable_role_addresses grantee ON (grantee.classid,grantee.objid,grantee.objsubid)=('pg_catalog.pg_authid'::regclass::oid,acl.grantee,0)),'[]'::jsonb)) AS record,
 (col.valid AND rel.valid AND typ.valid AND (a.attcollation=0 OR coll.valid)
 AND NOT a.attisdropped AND (NOT a.atthasmissing AND a.attmissingval IS NULL
 OR a.atthasmissing AND a.attmissingval IS NOT NULL
 AND a.atttypid IN ('pg_catalog.text'::regtype,'pg_catalog.bool'::regtype,'pg_catalog.text[]'::regtype,'pg_catalog.uuid[]'::regtype))) IS TRUE AS valid
 FROM selected_attributes a
 LEFT JOIN stable_addresses col ON (col.classid,col.objid,col.objsubid)=('pg_catalog.pg_class'::regclass::oid,a.attrelid,a.attnum::integer)
 LEFT JOIN stable_addresses rel ON (rel.classid,rel.objid,rel.objsubid)=('pg_catalog.pg_class'::regclass::oid,a.attrelid,0)
 LEFT JOIN stable_addresses typ ON (typ.classid,typ.objid,typ.objsubid)=('pg_catalog.pg_type'::regclass::oid,a.atttypid,0)
 LEFT JOIN stable_addresses coll ON (coll.classid,coll.objid,coll.objsubid)=('pg_catalog.pg_collation'::regclass::oid,a.attcollation,0)
), default_records AS MATERIALIZED (
 SELECT obj.address AS identity,jsonb_build_object('address',obj.address,'relation',rel.address,'column',col.address,
 'expression',pg_catalog.pg_get_expr(d.adbin,d.adrelid,false)) AS record,
 (obj.valid AND rel.valid AND col.valid AND d.adnum>0 AND pg_catalog.pg_get_expr(d.adbin,d.adrelid,false) IS NOT NULL) IS TRUE AS valid
 FROM selected_defaults d
 LEFT JOIN stable_addresses obj ON (obj.classid,obj.objid,obj.objsubid)=('pg_catalog.pg_attrdef'::regclass::oid,d.oid,0)
 LEFT JOIN stable_addresses rel ON (rel.classid,rel.objid,rel.objsubid)=('pg_catalog.pg_class'::regclass::oid,d.adrelid,0)
 LEFT JOIN stable_addresses col ON (col.classid,col.objid,col.objsubid)=('pg_catalog.pg_class'::regclass::oid,d.adrelid,d.adnum::integer)
), comment_records AS MATERIALIZED (
 SELECT s.address AS identity,jsonb_build_object('address',s.address,'present',d.objoid IS NOT NULL,'text',d.description) AS record
 FROM metadata_m m LEFT JOIN stable_addresses s USING(classid,objid,objsubid)
 LEFT JOIN pg_catalog.pg_description d ON (d.classoid,d.objoid,d.objsubid)=(m.classid,m.objid,m.objsubid)
 WHERE m.classid<>'pg_catalog.pg_authid'::regclass
), shared_comment_records AS MATERIALIZED (
 SELECT s.address AS identity,jsonb_build_object('address',s.address,'present',d.objoid IS NOT NULL,'text',d.description) AS record
 FROM metadata_m m LEFT JOIN stable_addresses s USING(classid,objid,objsubid)
 LEFT JOIN pg_catalog.pg_shdescription d ON (d.classoid,d.objoid)=(m.classid,m.objid)
 WHERE m.classid='pg_catalog.pg_authid'::regclass AND m.objsubid=0
), ordinary_edge_records AS MATERIALIZED (
 SELECT dep.address AS dependent,ref.address AS referenced,d.deptype::text AS deptype,
 jsonb_build_object('dependent',dep.address,'referenced',ref.address,'dependency_type',d.deptype::text) AS record,
 (dep.valid AND ref.valid) IS TRUE AS valid
 FROM ordinary_edges d
 LEFT JOIN stable_addresses dep ON (dep.classid,dep.objid,dep.objsubid)=(d.classid,d.objid,d.objsubid)
 LEFT JOIN stable_addresses ref ON (ref.classid,ref.objid,ref.objsubid)=(d.refclassid,d.refobjid,d.refobjsubid)
), shared_edge_records AS MATERIALIZED (
 SELECT dep.address AS dependent,ref.address AS referenced,d.deptype::text AS deptype,
 jsonb_build_object('dependent',dep.address,'referenced',ref.address,'dependency_type',d.deptype::text) AS record,
 (dep.valid AND ref.valid) IS TRUE AS valid
 FROM shared_edges d
 LEFT JOIN stable_addresses dep ON (dep.classid,dep.objid,dep.objsubid)=(d.classid,d.objid,d.objsubid)
 LEFT JOIN stable_addresses ref ON (ref.classid,ref.objid,ref.objsubid)=(d.refclassid,d.refobjid,0)
), org_relation_names AS MATERIALIZED (
 SELECT n.nspname,c.relname,c.relkind,pg_catalog.pg_get_userbyid(c.relowner) AS owner
 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 WHERE c.relname='native_org_unit' OR starts_with(c.relname,'native_org_unit_')
), org_schema_names AS MATERIALIZED (
 SELECT n.nspname,pg_catalog.pg_get_userbyid(n.nspowner) AS owner FROM pg_catalog.pg_namespace n
 WHERE n.nspname='native_org_unit' OR starts_with(n.nspname,'native_org_unit_')
), org_type_names AS MATERIALIZED (
 SELECT n.nspname,t.typname,t.typtype,pg_catalog.pg_get_userbyid(t.typowner) AS owner
 FROM pg_catalog.pg_type t JOIN pg_catalog.pg_namespace n ON n.oid=t.typnamespace
 WHERE t.typname='native_org_unit' OR starts_with(t.typname,'native_org_unit_')
), org_routine_names AS MATERIALIZED (
 SELECT p.*,n.nspname,pg_catalog.pg_get_userbyid(p.proowner) AS owner FROM pg_catalog.pg_proc p
 JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'native_org_unit_')
), provenance_routines AS MATERIALIZED (
 SELECT p.*,n.nspname FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE starts_with(p.proname,'account_company_provenance')
), guard_relations(name) AS (VALUES ('ont_action_command_receipts'),('org_unit_revisions'),('org_unit_source_bindings'),('org_units')),
 org_namespace_predicate AS (
 SELECT (NOT EXISTS(SELECT 1 FROM org_relation_names) AND NOT EXISTS(SELECT 1 FROM org_schema_names)
 AND NOT EXISTS(SELECT 1 FROM org_type_names)
 AND (SELECT count(*)=1 AND bool_and(nspname='public' AND proname='native_org_unit_closed_guard_v1'
 AND prokind='f' AND pronargs=0 AND prorettype='pg_catalog.trigger'::regtype AND owner='console_account_owner'
 AND encode(sha256(convert_to(prosrc,'UTF8')),'hex')='afda867ce859a820e3cebe033e1b1597532f99bb07a31174be494f480770dd3f') FROM org_routine_names) IS TRUE
 AND (SELECT count(*)=2 AND bool_and(nspname='public' AND prokind='f' AND proowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname=CASE proname WHEN 'account_company_provenance_lock_v1' THEN 'console_app' ELSE 'console_account_owner' END)
 AND ((proname='account_company_provenance_v1' AND proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector)
 OR (proname='account_company_provenance_lock_v1' AND proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector))) FROM provenance_routines) IS TRUE
 AND (SELECT count(*)=4 AND count(DISTINCT c.oid)=4 AND bool_and(n.nspname='public' AND c.relkind='r' AND NOT c.relispartition AND owner.rolname='console_app')
 FROM guard_relations g LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=g.name LEFT JOIN pg_catalog.pg_roles owner ON owner.oid=c.relowner) IS TRUE
 AND (SELECT count(*)=8 AND bool_and(t.tgenabled='A' AND NOT t.tgisinternal AND t.tgfoid=(SELECT oid FROM org_routine_names)
 AND t.tgtype=CASE t.tgname WHEN 'native_org_unit_closed_row_v1' THEN 31 ELSE 34 END)
 FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='public' AND c.relname IN (SELECT name FROM guard_relations)
 AND t.tgname IN ('native_org_unit_closed_row_v1','native_org_unit_closed_truncate_v1')) IS TRUE
 AND (SELECT count(*)=3 AND bool_and(t.tgenabled='A') FROM pg_catalog.pg_trigger t JOIN pg_catalog.pg_class c ON c.oid=t.tgrelid
 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND
 (c.relname,t.tgname) IN (VALUES ('org_unit_revisions','trg_org_unit_revisions_immutable'),
 ('org_unit_source_bindings','trg_org_unit_source_bindings_immutable'),('ont_action_command_receipts','trg_ont_action_command_receipts_immutable')))) IS TRUE AS valid
), group_routine_identities(identity) AS (VALUES
 ('public.native_group_process_uuid_v1(bytea,integer)'),
 ('public.native_group_process_i64_v1(bytea,integer)'),
 ('public.native_group_process_text_v1(text,integer)'),
 ('public.native_group_process_micros_v1(timestamptz)'),
 ('public.native_group_process_decode_v1(bytea)'),
 ('public.native_group_process_policy_ref_v1(smallint,bigint,bytea)'),
 ('public.native_group_process_head_ref_v1(uuid,bigint,bigint,bytea,bytea,text,timestamptz)'),
 ('public.native_group_process_action_roster_v1()'),
 ('public.native_group_process_registration_bytes_v1(uuid,uuid,text,bytea,bytea,bytea,jsonb)'),
 ('public.native_group_process_policy_head_bytes_v1(public.native_group_identity_policy_heads_v1)'),
 ('public.native_group_process_version_bytes_v1(public.native_group_process_versions_v1)'),
 ('public.native_group_process_head_bytes_v1(public.native_group_process_head_revisions_v1)'),
 ('public.native_group_process_result_bytes_v2(public.native_group_process_results_v1)'),
 ('public.native_group_process_registered_actions_v1()'),
 ('public.native_group_process_source_v1(uuid,uuid)'),
 ('public.native_group_process_group_guard_v1(uuid,boolean)'),
 ('public.native_group_process_command_guard_v1(uuid,uuid,boolean)'),
 ('public.native_group_process_topology_fence_v1()'),
 ('public.native_group_process_account_material_v1(uuid,uuid)'),
 ('public.native_group_process_original_material_v1(uuid,uuid,uuid,uuid)'),
 ('public.identity_native_group_process_material_v1(uuid,uuid,uuid,uuid,uuid,smallint,bytea)'),
 ('public.identity_native_group_process_incarnation_selector_v1(uuid,uuid,uuid,uuid)'),
 ('public.identity_native_group_process_navigation_candidates_v1(uuid,uuid,bytea)'),
 ('public.native_group_process_accept_snapshot_v1(public.native_group_process_inputs_v1)'),
 ('public.native_group_process_complete_snapshot_v1(public.native_group_process_results_v1)'),
 ('public.native_group_process_assert_input_v1(uuid,uuid)'),
 ('public.native_group_process_assert_result_v1(uuid,uuid)'),
 ('public.native_group_process_assert_current_v1(uuid,uuid)'),
 ('public.native_group_process_audit_guard_v1()'),
 ('public.native_group_process_audit_truncate_guard_v1()'),
 ('public.native_group_process_current_context_v1(uuid,uuid,uuid,uuid,boolean)'),
 ('public.native_group_process_classify_v1(public.native_group_process_effects_v1,jsonb)'),
 ('public.native_group_process_validate_frame_v1(public.native_group_process_effects_v1)'),
 ('public.native_group_process_input_guard_v1()'),
 ('public.native_group_process_effect_guard_v1()'),
 ('public.native_group_process_participant_guard_v1()'),
 ('public.native_group_process_deferred_guard_v1()'),
 ('public.native_group_process_immutable_statement_v1()'),
 ('public.native_group_process_head_statement_v1()'),
 ('public.identity_native_group_process_prepare_v1(uuid,uuid,uuid,bytea)'),
 ('public.identity_native_group_process_execute_v1(uuid,uuid,uuid)')
), group_routine_expected AS MATERIALIZED (
 SELECT identity,pg_catalog.to_regprocedure(identity) AS oid FROM group_routine_identities
), required_routine_specs(selector,schema_name,routine_name,argument_names,required_in_group) AS (VALUES
 ('public.account_session_shared_material_v1(uuid,uuid)','public','account_session_shared_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_account_session_shared_material_v1(uuid,uuid)','public','auth_account_session_shared_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.account_company_setup_eligibility_v1(uuid)','public','account_company_setup_eligibility_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_platform_source_material_v1(uuid,uuid)','public','auth_legacy_platform_source_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.account_context_presence_v1(uuid)','public','account_context_presence_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.account_login_consent_v1(uuid)','public','account_login_consent_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])','public','account_registration_activate_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','bigint','pg_catalog.bytea','pg_catalog.text[]','pg_catalog.bytea[]']::text[],FALSE),
 ('public.account_registration_begin_v1()','public','account_registration_begin_v1',ARRAY[]::text[],FALSE),
 ('public.account_security_lock_exclusive_v1(uuid)','public','account_security_lock_exclusive_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.account_security_lock_shared_v1(uuid)','public','account_security_lock_shared_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.account_session_logout_v1(uuid,uuid,bigint,interval)','public','account_session_logout_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','bigint','interval']::text[],FALSE),
 ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)','public','account_session_refresh_reuse_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea','bigint','interval']::text[],FALSE),
 ('public.account_terms_current_v1()','public','account_terms_current_v1',ARRAY[]::text[],FALSE),
 ('public.account_terms_registration_head_v1()','public','account_terms_registration_head_v1',ARRAY[]::text[],FALSE),
 ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)','public','auth_account_logout_revoke_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','bigint','interval']::text[],FALSE),
 ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)','public','auth_account_refresh_reuse_revoke_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea','bigint','interval']::text[],FALSE),
 ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)','public','auth_account_registration_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)','public','auth_legacy_audit_append_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.text','pg_catalog.text','pg_catalog.text','pg_catalog.uuid','pg_catalog.jsonb','pg_catalog.jsonb','character','character','timestamp with time zone','pg_catalog.uuid','pg_catalog.text','pg_catalog.text','pg_catalog.text','pg_catalog.text','pg_catalog.text[]','boolean','pg_catalog.text']::text[],FALSE),
 ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)','public','auth_legacy_bootstrap_issue_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea','timestamp with time zone','timestamp with time zone']::text[],FALSE),
 ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)','public','auth_legacy_bootstrap_seed_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea','timestamp with time zone','timestamp with time zone']::text[],FALSE),
 ('public.auth_legacy_cold_start_admin_v1()','public','auth_legacy_cold_start_admin_v1',ARRAY[]::text[],FALSE),
 ('public.auth_legacy_company_lock_v1(uuid)','public','auth_legacy_company_lock_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)','public','auth_legacy_deactivate_credentials_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','timestamp with time zone']::text[],FALSE),
 ('public.auth_legacy_group_passkey_flag_v1(uuid)','public','auth_legacy_group_passkey_flag_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_purge_company_v1(uuid)','public','auth_legacy_purge_company_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_purge_subjects_v1(uuid)','public','auth_legacy_purge_subjects_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)','public','auth_legacy_reset_credentials_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea','timestamp with time zone','timestamp with time zone']::text[],FALSE),
 ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)','public','auth_legacy_self_bootstrap_replace_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea','timestamp with time zone','timestamp with time zone']::text[],FALSE),
 ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)','public','auth_legacy_self_passkey_count_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)','public','auth_legacy_self_passkey_delete_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)','public','auth_legacy_self_passkey_state_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_self_passkeys_v1(uuid,uuid)','public','auth_legacy_self_passkeys_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_session_context_v1(uuid,uuid)','public','auth_legacy_session_context_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_user_active_v1(uuid,uuid)','public','auth_legacy_user_active_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)','public','auth_legacy_user_has_passkey_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.group_member_org_ids(uuid,uuid)','public','group_member_org_ids',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.group_role_grants_for_user(uuid)','public','group_role_grants_for_user',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_assign_org_to_group(uuid,uuid)','public','platform_assign_org_to_group',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.platform_attach_group_of_one(uuid)','public','platform_attach_group_of_one',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_attach_membership(uuid,uuid)','public','platform_attach_membership',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.platform_create_group(text,text)','public','platform_create_group',ARRAY['pg_catalog.text','pg_catalog.text']::text[],FALSE),
 ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)','public','platform_create_group_account',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.text','pg_catalog.text','pg_catalog.text[]','pg_catalog.text','pg_catalog.uuid']::text[],FALSE),
 ('public.platform_create_organization(text,text)','public','platform_create_organization',ARRAY['pg_catalog.text','pg_catalog.text']::text[],FALSE),
 ('public.platform_force_remove_direct_org_children(uuid)','public','platform_force_remove_direct_org_children',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_force_remove_organization(uuid)','public','platform_force_remove_organization',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)','public','platform_force_remove_organization_command',ARRAY['pg_catalog.uuid','pg_catalog.uuid','character','character','timestamp with time zone']::text[],FALSE),
 ('public.platform_get_group(uuid)','public','platform_get_group',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_get_organization(uuid)','public','platform_get_organization',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_list_group_accounts(uuid)','public','platform_list_group_accounts',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_list_groups()','public','platform_list_groups',ARRAY[]::text[],FALSE),
 ('public.platform_list_organizations()','public','platform_list_organizations',ARRAY[]::text[],FALSE),
 ('public.platform_mint_group_row(uuid,text,text)','public','platform_mint_group_row',ARRAY['pg_catalog.uuid','pg_catalog.text','pg_catalog.text']::text[],FALSE),
 ('public.platform_mint_missing_group_of_one(uuid)','public','platform_mint_missing_group_of_one',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_remove_org_from_group(uuid,uuid)','public','platform_remove_org_from_group',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('public.platform_remove_organization(uuid)','public','platform_remove_organization',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.platform_resolve_bootstrap_org(bytea)','public','platform_resolve_bootstrap_org',ARRAY['pg_catalog.bytea']::text[],FALSE),
 ('public.platform_resolve_credential_org(text)','public','platform_resolve_credential_org',ARRAY['pg_catalog.text']::text[],FALSE),
 ('public.platform_resolve_token_org(bytea)','public','platform_resolve_token_org',ARRAY['pg_catalog.bytea']::text[],FALSE),
 ('public.platform_revoke_group_role(uuid,uuid,text)','public','platform_revoke_group_role',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.text']::text[],FALSE),
 ('public.platform_set_organization_status(uuid,text)','public','platform_set_organization_status',ARRAY['pg_catalog.uuid','pg_catalog.text']::text[],FALSE),
 ('public.platform_update_group(uuid,text,text,text)','public','platform_update_group',ARRAY['pg_catalog.uuid','pg_catalog.text','pg_catalog.text','pg_catalog.text']::text[],FALSE),
 ('public.native_group_process_uuid_v1(bytea,integer)','public','native_group_process_uuid_v1',ARRAY['pg_catalog.bytea','integer']::text[],TRUE),
 ('public.native_group_process_i64_v1(bytea,integer)','public','native_group_process_i64_v1',ARRAY['pg_catalog.bytea','integer']::text[],TRUE),
 ('public.native_group_process_text_v1(text,integer)','public','native_group_process_text_v1',ARRAY['pg_catalog.text','integer']::text[],TRUE),
 ('public.native_group_process_micros_v1(timestamptz)','public','native_group_process_micros_v1',ARRAY['timestamp with time zone']::text[],TRUE),
 ('public.native_group_process_decode_v1(bytea)','public','native_group_process_decode_v1',ARRAY['pg_catalog.bytea']::text[],TRUE),
 ('public.native_group_process_policy_ref_v1(smallint,bigint,bytea)','public','native_group_process_policy_ref_v1',ARRAY['smallint','bigint','pg_catalog.bytea']::text[],TRUE),
 ('public.native_group_process_head_ref_v1(uuid,bigint,bigint,bytea,bytea,text,timestamptz)','public','native_group_process_head_ref_v1',ARRAY['pg_catalog.uuid','bigint','bigint','pg_catalog.bytea','pg_catalog.bytea','pg_catalog.text','timestamp with time zone']::text[],TRUE),
 ('public.native_group_process_action_roster_v1()','public','native_group_process_action_roster_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_registration_bytes_v1(uuid,uuid,text,bytea,bytea,bytea,jsonb)','public','native_group_process_registration_bytes_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.text','pg_catalog.bytea','pg_catalog.bytea','pg_catalog.bytea','pg_catalog.jsonb']::text[],TRUE),
 ('public.native_group_process_policy_head_bytes_v1(public.native_group_identity_policy_heads_v1)','public','native_group_process_policy_head_bytes_v1',ARRAY['public.native_group_identity_policy_heads_v1']::text[],TRUE),
 ('public.native_group_process_version_bytes_v1(public.native_group_process_versions_v1)','public','native_group_process_version_bytes_v1',ARRAY['public.native_group_process_versions_v1']::text[],TRUE),
 ('public.native_group_process_head_bytes_v1(public.native_group_process_head_revisions_v1)','public','native_group_process_head_bytes_v1',ARRAY['public.native_group_process_head_revisions_v1']::text[],TRUE),
 ('public.native_group_process_result_bytes_v2(public.native_group_process_results_v1)','public','native_group_process_result_bytes_v2',ARRAY['public.native_group_process_results_v1']::text[],TRUE),
 ('public.native_group_process_registered_actions_v1()','public','native_group_process_registered_actions_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_source_v1(uuid,uuid)','public','native_group_process_source_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.native_group_process_group_guard_v1(uuid,boolean)','public','native_group_process_group_guard_v1',ARRAY['pg_catalog.uuid','boolean']::text[],TRUE),
 ('public.native_group_process_command_guard_v1(uuid,uuid,boolean)','public','native_group_process_command_guard_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','boolean']::text[],TRUE),
 ('public.native_group_process_topology_fence_v1()','public','native_group_process_topology_fence_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_account_material_v1(uuid,uuid)','public','native_group_process_account_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.native_group_process_original_material_v1(uuid,uuid,uuid,uuid)','public','native_group_process_original_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.identity_native_group_process_material_v1(uuid,uuid,uuid,uuid,uuid,smallint,bytea)','public','identity_native_group_process_material_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','smallint','pg_catalog.bytea']::text[],TRUE),
 ('public.identity_native_group_process_incarnation_selector_v1(uuid,uuid,uuid,uuid)','public','identity_native_group_process_incarnation_selector_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.identity_native_group_process_navigation_candidates_v1(uuid,uuid,bytea)','public','identity_native_group_process_navigation_candidates_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea']::text[],TRUE),
 ('public.native_group_process_accept_snapshot_v1(public.native_group_process_inputs_v1)','public','native_group_process_accept_snapshot_v1',ARRAY['public.native_group_process_inputs_v1']::text[],TRUE),
 ('public.native_group_process_complete_snapshot_v1(public.native_group_process_results_v1)','public','native_group_process_complete_snapshot_v1',ARRAY['public.native_group_process_results_v1']::text[],TRUE),
 ('public.native_group_process_assert_input_v1(uuid,uuid)','public','native_group_process_assert_input_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.native_group_process_assert_result_v1(uuid,uuid)','public','native_group_process_assert_result_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.native_group_process_assert_current_v1(uuid,uuid)','public','native_group_process_assert_current_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.native_group_process_audit_guard_v1()','public','native_group_process_audit_guard_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_audit_truncate_guard_v1()','public','native_group_process_audit_truncate_guard_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_current_context_v1(uuid,uuid,uuid,uuid,boolean)','public','native_group_process_current_context_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','boolean']::text[],TRUE),
 ('public.native_group_process_classify_v1(public.native_group_process_effects_v1,jsonb)','public','native_group_process_classify_v1',ARRAY['public.native_group_process_effects_v1','pg_catalog.jsonb']::text[],TRUE),
 ('public.native_group_process_validate_frame_v1(public.native_group_process_effects_v1)','public','native_group_process_validate_frame_v1',ARRAY['public.native_group_process_effects_v1']::text[],TRUE),
 ('public.native_group_process_input_guard_v1()','public','native_group_process_input_guard_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_effect_guard_v1()','public','native_group_process_effect_guard_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_participant_guard_v1()','public','native_group_process_participant_guard_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_deferred_guard_v1()','public','native_group_process_deferred_guard_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_immutable_statement_v1()','public','native_group_process_immutable_statement_v1',ARRAY[]::text[],TRUE),
 ('public.native_group_process_head_statement_v1()','public','native_group_process_head_statement_v1',ARRAY[]::text[],TRUE),
 ('public.identity_native_group_process_prepare_v1(uuid,uuid,uuid,bytea)','public','identity_native_group_process_prepare_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid','pg_catalog.bytea']::text[],TRUE),
 ('public.identity_native_group_process_execute_v1(uuid,uuid,uuid)','public','identity_native_group_process_execute_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid','pg_catalog.uuid']::text[],TRUE),
 ('public.native_org_unit_closed_guard_v1()','public','native_org_unit_closed_guard_v1',ARRAY[]::text[],FALSE),
 ('public.account_company_provenance_v1(uuid)','public','account_company_provenance_v1',ARRAY['pg_catalog.uuid']::text[],FALSE),
 ('public.account_company_provenance_lock_v1(uuid,uuid)','public','account_company_provenance_lock_v1',ARRAY['pg_catalog.uuid','pg_catalog.uuid']::text[],FALSE),
 ('pg_catalog.pg_control_system()','pg_catalog','pg_control_system',ARRAY[]::text[],FALSE),
 ('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)','public','deployment_operator_designate_v1',ARRAY['pg_catalog.text','pg_catalog.text','bigint','pg_catalog.uuid','pg_catalog.uuid','bigint','bigint']::text[],FALSE),
 ('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)','public','deployment_operator_revoke_v1',ARRAY['pg_catalog.text','pg_catalog.text','bigint','pg_catalog.uuid','pg_catalog.uuid','bigint','pg_catalog.text']::text[],FALSE)
), required_routine_addresses AS MATERIALIZED (
 SELECT selector,required_in_group,
 jsonb_build_object('type','function','object_names',jsonb_build_array(schema_name,routine_name),
 'object_args',to_jsonb(argument_names)) AS expected_address,
 pg_catalog.to_regprocedure(selector) AS oid FROM required_routine_specs
), group_namespace_predicate AS (
 SELECT (
 -- Closed76 has no Group namespace and precisely the seven absent wanted relations.
 (NOT EXISTS(SELECT 1 FROM selected_relations r WHERE r.relname IN (SELECT name FROM group_wanted))
 AND (SELECT snapshot->'native_group_process_relation_namespace' IS NULL OR snapshot->'native_group_process_relation_namespace'='null'::jsonb FROM historical83_query)
 AND (SELECT snapshot->'native_group_process_schema_namespace' IS NULL OR snapshot->'native_group_process_schema_namespace'='null'::jsonb FROM historical83_query)
 AND (SELECT snapshot->'native_group_process_type_namespace' IS NULL OR snapshot->'native_group_process_type_namespace'='null'::jsonb FROM historical83_query)
 AND (SELECT snapshot->'native_group_process_routine_namespace' IS NULL OR snapshot->'native_group_process_routine_namespace'='null'::jsonb FROM historical83_query))
 OR (
 (SELECT count(*)=7 AND bool_and(relkind='r' AND NOT relispartition AND relowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_account_owner'))
 FROM selected_relations WHERE relname IN (SELECT name FROM group_wanted)) IS TRUE
 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_namespace n WHERE starts_with(n.nspname,'native_group_process') OR starts_with(n.nspname,'native_group_identity_policy') OR starts_with(n.nspname,'identity_native_group_process'))
 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class c WHERE
 (starts_with(c.relname,'native_group_process') OR starts_with(c.relname,'native_group_identity_policy') OR starts_with(c.relname,'identity_native_group_process'))
 AND NOT (c.oid IN (SELECT oid FROM selected_relations WHERE relname IN (SELECT name FROM group_wanted))
 OR c.relkind IN ('i','I') AND c.oid IN (SELECT i.indexrelid FROM selected_indexes i JOIN selected_relations r ON r.oid=i.indrelid WHERE r.relname IN (SELECT name FROM group_wanted))))
 AND (SELECT count(*)=7 AND bool_and(t.typtype='c' AND t.typowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_account_owner')
 AND t.oid IN (SELECT reltype FROM selected_relations WHERE relname IN (SELECT name FROM group_wanted)))
 FROM pg_catalog.pg_type t WHERE starts_with(t.typname,'native_group_process') OR starts_with(t.typname,'native_group_identity_policy') OR starts_with(t.typname,'identity_native_group_process')) IS TRUE
 AND (SELECT count(*)=41 AND count(DISTINCT p.oid)=41
 AND bool_and(p.oid IS NOT NULL AND n.nspname='public' AND p.prokind='f' AND p.proowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_account_owner'))
 FROM group_routine_expected e LEFT JOIN pg_catalog.pg_proc p ON p.oid=e.oid LEFT JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace) IS TRUE
 AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p WHERE
 (starts_with(p.proname,'native_group_process') OR starts_with(p.proname,'native_group_identity_policy') OR starts_with(p.proname,'identity_native_group_process'))
 AND NOT EXISTS(SELECT 1 FROM group_routine_expected e WHERE e.oid=p.oid))
 )) IS TRUE AS valid
), org_tables AS MATERIALIZED (
 SELECT r.* FROM selected_relations r WHERE r.relname IN ('org_units','org_unit_revisions','org_unit_source_bindings')
), actor_columns AS MATERIALIZED (
 SELECT a.* FROM selected_attributes a JOIN selected_relations r ON r.oid=a.attrelid
 WHERE r.relname='org_unit_revisions' AND a.attname IN ('actor_kind','actor_account_id')
), actor_successors AS MATERIALIZED (
 SELECT k.*,c.relname,n.nspname FROM pg_catalog.pg_constraint k JOIN pg_catalog.pg_class c ON c.oid=k.conrelid
 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 WHERE k.conname IN ('org_unit_revisions_actor_protocol_v1','org_unit_revisions_native_actor_v1','ont_action_receipts_actor_protocol_v2')
), actor_phase_predicate AS (
 SELECT (
 (NOT EXISTS(SELECT 1 FROM actor_columns) AND NOT EXISTS(SELECT 1 FROM actor_successors))
 OR (
 (SELECT count(*)=2 AND count(DISTINCT attnum)=2 AND bool_and(NOT attisdropped AND
 (attnum=11 AND attname='actor_kind' AND atttypid='pg_catalog.text'::regtype AND attnotnull AND atthasdef AND atthasmissing AND to_jsonb(attmissingval)='["USER"]'::jsonb
 OR attnum=12 AND attname='actor_account_id' AND atttypid='pg_catalog.uuid'::regtype AND NOT attnotnull AND NOT atthasdef AND NOT atthasmissing AND attmissingval IS NULL)) FROM actor_columns) IS TRUE
 AND (SELECT count(*)=1 AND bool_and(pg_catalog.pg_get_expr(d.adbin,d.adrelid,false)='''USER''::text') FROM selected_defaults d
 JOIN selected_relations r ON r.oid=d.adrelid WHERE r.relname='org_unit_revisions' AND d.adnum=11) IS TRUE
 AND NOT EXISTS(SELECT 1 FROM selected_defaults d JOIN selected_relations r ON r.oid=d.adrelid WHERE r.relname='org_unit_revisions' AND d.adnum=12)
 AND (SELECT count(*)=2 AND bool_and(d.description=CASE a.attname WHEN 'actor_kind' THEN
 'pd:personal — canonical OrgUnit actor attribution protocol' ELSE 'pd:personal — canonical OrgUnit Account actor identity' END)
 FROM actor_columns a JOIN pg_catalog.pg_description d ON d.classoid='pg_catalog.pg_class'::regclass AND d.objoid=a.attrelid AND d.objsubid=a.attnum) IS TRUE
 AND (SELECT count(*)=3 AND count(DISTINCT conname)=3 AND (bool_and(convalidated IS FALSE) OR bool_and(convalidated IS TRUE))
 AND bool_and(nspname='public' AND NOT condeferrable AND NOT condeferred AND conenforced AND conislocal AND coninhcount=0 AND conparentid=0
 AND (conname='ont_action_receipts_actor_protocol_v2' AND relname='ont_action_command_receipts' AND contype='c'
 OR conname='org_unit_revisions_actor_protocol_v1' AND relname='org_unit_revisions' AND contype='c'
 OR conname='org_unit_revisions_native_actor_v1' AND relname='org_unit_revisions' AND contype='f'
 AND confrelid=(SELECT oid FROM selected_relations WHERE relname='company_actors')
 AND conkey=ARRAY[1,12]::smallint[] AND confkey=ARRAY[1,2]::smallint[] AND confupdtype='r' AND confdeltype='r' AND confmatchtype='s'
 AND conindid=pg_catalog.to_regclass('public.company_actors_pkey') AND contypid=0 AND NOT conperiod
 AND conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid,'pg_catalog.=(uuid,uuid)'::regoperator::oid]
 AND conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid,'pg_catalog.=(uuid,uuid)'::regoperator::oid]
 AND conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid,'pg_catalog.=(uuid,uuid)'::regoperator::oid]))
 FROM actor_successors) IS TRUE
 AND (SELECT count(*)=1 AND bool_and(contype='n' AND convalidated AND conkey=ARRAY[11]::smallint[]) FROM selected_constraints
 WHERE conname='org_unit_revisions_actor_kind_not_null') IS TRUE
 )) IS TRUE AS valid
), required_org_column_identities(relation,attnum,name) AS (VALUES
 ('org_unit_revisions',1,'org_id'),('org_unit_revisions',2,'id'),('org_unit_revisions',3,'org_unit_id'),
 ('org_unit_revisions',4,'version'),('org_unit_revisions',5,'command_id'),('org_unit_revisions',6,'actor_id'),
 ('org_unit_revisions',7,'payload_digest'),('org_unit_revisions',8,'attributes'),('org_unit_revisions',9,'receipt'),('org_unit_revisions',10,'created_at'),
 ('org_unit_source_bindings',1,'org_id'),('org_unit_source_bindings',2,'source_kind'),('org_unit_source_bindings',3,'source_id'),
 ('org_unit_source_bindings',4,'org_unit_id'),('org_unit_source_bindings',5,'actor_id'),('org_unit_source_bindings',6,'payload_digest'),('org_unit_source_bindings',7,'created_at'),
 ('org_units',1,'org_id'),('org_units',2,'id'),('org_units',3,'created_at')
), phase_org_column_identities AS (
 SELECT * FROM required_org_column_identities
 UNION ALL SELECT 'org_unit_revisions',11,'actor_kind' WHERE EXISTS(SELECT 1 FROM actor_columns)
 UNION ALL SELECT 'org_unit_revisions',12,'actor_account_id' WHERE EXISTS(SELECT 1 FROM actor_columns)
), org_startup_checks AS MATERIALIZED (
 SELECT 'table'::text AS kind,r.relname::text AS relation,NULL::smallint AS attnum,NULL::text AS name,p.privilege,
 pg_catalog.has_table_privilege(s.oid,r.oid,p.privilege) AS allowed
 FROM pg_catalog.pg_roles s CROSS JOIN org_tables r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) p(privilege)
 WHERE s.rolname='console_auth_startup'
 UNION ALL
 SELECT 'column',r.relname::text,a.attnum,a.attname::text,p.privilege,pg_catalog.has_column_privilege(s.oid,r.oid,a.attnum,p.privilege)
 FROM pg_catalog.pg_roles s CROSS JOIN org_tables r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) p(privilege) WHERE s.rolname='console_auth_startup'
), org_startup_predicate AS (
 SELECT ((SELECT count(*) FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup')=1
 AND (SELECT count(*)=3 AND count(DISTINCT oid)=3 AND bool_and(relkind='r' AND NOT relispartition AND relowner=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_app')) FROM org_tables) IS TRUE
 AND (SELECT count(*) FROM org_tables r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped)
 =CASE WHEN EXISTS(SELECT 1 FROM actor_columns) THEN 22 ELSE 20 END
 AND (SELECT count(*) FROM org_tables r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 JOIN phase_org_column_identities e ON (e.relation,e.attnum,e.name)=(r.relname::text,a.attnum::integer,a.attname::text))
 =CASE WHEN EXISTS(SELECT 1 FROM actor_columns) THEN 22 ELSE 20 END
 AND (SELECT count(*) FROM org_startup_checks WHERE kind='table')=24
 AND (SELECT count(DISTINCT (relation,privilege)) FROM org_startup_checks WHERE kind='table')=24
 AND (SELECT count(*) FROM org_startup_checks WHERE kind='column')=CASE WHEN EXISTS(SELECT 1 FROM actor_columns) THEN 88 ELSE 80 END
 AND (SELECT count(DISTINCT (relation,attnum,privilege)) FROM org_startup_checks WHERE kind='column')=CASE WHEN EXISTS(SELECT 1 FROM actor_columns) THEN 88 ELSE 80 END
 AND (SELECT bool_and(allowed IS FALSE) FROM org_startup_checks) IS TRUE
 AND (SELECT valid FROM actor_phase_predicate) IS TRUE) IS TRUE AS valid
), roster_records AS MATERIALIZED (
 SELECT s.address AS identity,jsonb_build_object('address',s.address,'present',TRUE,'incoming_reference',
 EXISTS(SELECT 1 FROM incoming_i i WHERE (i.classid,i.objid,i.objsubid)=(m.classid,m.objid,m.objsubid)),
 'RI_flags',s.ri_flags) AS record FROM metadata_m m LEFT JOIN stable_addresses s USING(classid,objid,objsubid)
 UNION ALL
 SELECT jsonb_build_object('type','table','object_names',jsonb_build_array('public',w.name),'object_args','[]'::jsonb),
 jsonb_build_object('expected_address',jsonb_build_object('type','table','object_names',jsonb_build_array('public',w.name),'object_args','[]'::jsonb),
 'present',FALSE,'required_in_group',w.name IN (SELECT name FROM group_wanted))
 FROM wanted w WHERE NOT EXISTS(SELECT 1 FROM selected_relations r WHERE r.relname=w.name)
 UNION ALL
 SELECT jsonb_build_object('type','schema','object_names',jsonb_build_array(r.name),'object_args','[]'::jsonb),
 jsonb_build_object('expected_address',jsonb_build_object('type','schema','object_names',jsonb_build_array(r.name),'object_args','[]'::jsonb),'present',FALSE)
 FROM required_schemas r WHERE NOT EXISTS(SELECT 1 FROM selected_schemas n WHERE n.nspname=r.name)
 UNION ALL
 SELECT jsonb_build_object('type','role','object_names',jsonb_build_array(required.name),'object_args','[]'::jsonb),
 jsonb_build_object('expected_address',jsonb_build_object('type','role','object_names',jsonb_build_array(required.name),'object_args','[]'::jsonb),'present',FALSE)
 FROM (VALUES ('console_account_owner'),('console_terms_owner'),('console_credential_owner'),('console_auth_rt'),('console_auth_startup'),
 ('console_ontology_writer'),('console_ontology_cmd'),('console_platform_force_cmd'),('console_rt'),('console_app'),('console_leave_definer'),('console_leave_cmd')) required(name)
 WHERE NOT EXISTS(SELECT 1 FROM protected_roles role WHERE role.rolname=required.name)
 UNION ALL
 SELECT jsonb_build_object('type','role','object_names',jsonb_build_array('builtin_owner'),'object_args','[]'::jsonb),
 jsonb_build_object('expected_address',jsonb_build_object('type','role','object_names',jsonb_build_array('builtin_owner'),'object_args','[]'::jsonb),'present',FALSE)
 WHERE NOT EXISTS(SELECT 1 FROM builtin_owner)
 UNION ALL
 SELECT expected_address,jsonb_build_object('expected_address',expected_address,'present',FALSE,'required_in_group',required_in_group)
 FROM required_routine_addresses required WHERE required.oid IS NULL
), supplemental AS MATERIALIZED (
 SELECT jsonb_build_object(
 'object_roster',(SELECT COALESCE(jsonb_agg(record ORDER BY identity::text COLLATE "C",record::text COLLATE "C"),'[]'::jsonb) FROM roster_records),
 'attributes',(SELECT COALESCE(jsonb_agg(record ORDER BY identity::text COLLATE "C",record::text COLLATE "C"),'[]'::jsonb) FROM attribute_records),
 'defaults',(SELECT COALESCE(jsonb_agg(record ORDER BY identity::text COLLATE "C",record::text COLLATE "C"),'[]'::jsonb) FROM default_records),
 'comments',(SELECT COALESCE(jsonb_agg(record ORDER BY identity::text COLLATE "C",record::text COLLATE "C"),'[]'::jsonb) FROM comment_records),
 'shared_comments',(SELECT COALESCE(jsonb_agg(record ORDER BY identity::text COLLATE "C",record::text COLLATE "C"),'[]'::jsonb) FROM shared_comment_records),
 'ordinary_dependencies',(SELECT COALESCE(jsonb_agg(record ORDER BY dependent::text COLLATE "C",referenced::text COLLATE "C",deptype COLLATE "C"),'[]'::jsonb) FROM ordinary_edge_records),
 'shared_dependencies',(SELECT COALESCE(jsonb_agg(record ORDER BY dependent::text COLLATE "C",referenced::text COLLATE "C",deptype COLLATE "C"),'[]'::jsonb) FROM shared_edge_records),
 'org_namespaces',jsonb_build_object('relations',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY to_jsonb(x)::text COLLATE "C"),'[]'::jsonb) FROM org_relation_names x),
 'schemas',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY to_jsonb(x)::text COLLATE "C"),'[]'::jsonb) FROM org_schema_names x),
 'types',(SELECT COALESCE(jsonb_agg(to_jsonb(x) ORDER BY to_jsonb(x)::text COLLATE "C"),'[]'::jsonb) FROM org_type_names x),
 'valid',(SELECT valid FROM org_namespace_predicate)),
 'group_namespaces',jsonb_build_object('relations',(SELECT snapshot->'native_group_process_relation_namespace' FROM historical83_query),
 'schemas',(SELECT snapshot->'native_group_process_schema_namespace' FROM historical83_query),
 'types',(SELECT snapshot->'native_group_process_type_namespace' FROM historical83_query),
 'routines',(SELECT snapshot->'native_group_process_routine_namespace' FROM historical83_query),'valid',(SELECT valid FROM group_namespace_predicate)),
 'org_startup_denial',jsonb_build_object('checks',(SELECT COALESCE(jsonb_agg(jsonb_build_object('kind',kind,'relation',relation,'column',name,'attnum',attnum,
 'privilege',privilege,'allowed',allowed) ORDER BY relation COLLATE "C",kind COLLATE "C",attnum,privilege COLLATE "C"),'[]'::jsonb) FROM org_startup_checks),
 'valid',(SELECT valid FROM org_startup_predicate)),
 'resolution_valid',(SELECT valid FROM resolution)) AS value
), complete_snapshot AS MATERIALIZED (
 SELECT jsonb_build_object('schema','console.native_org_unit.account_actor_complete_capture.v2',
 'historical76',h76.snapshot,'historical83',h83.snapshot,
 'historical76_sha256',h76.snapshot_sha256,'historical83_sha256',h83.snapshot_sha256,
 'historical76_rights',h76.native_directory_startup_rights_valid,
 'historical83_rights',h83.native_group_process_startup_rights_valid,
 'original73_rights',h73.native_directory_startup_rights_valid,'supplemental',s.value) AS snapshot,
 ((SELECT valid FROM resolution) IS TRUE AND (SELECT valid FROM org_namespace_predicate) IS TRUE
 AND (SELECT valid FROM group_namespace_predicate) IS TRUE AND (SELECT valid FROM org_startup_predicate) IS TRUE
 AND (SELECT count(*) FROM selected_schemas)=5 AND (SELECT count(*) FROM protected_roles)=12 AND (SELECT count(*) FROM owner_roles)=5
 AND (SELECT count(*) FROM builtin_owner)=1
 AND NOT EXISTS(SELECT 1 FROM required_routine_addresses required
 WHERE (NOT required.required_in_group OR EXISTS(SELECT 1 FROM selected_relations WHERE relname IN (SELECT name FROM group_wanted)))
 AND (required.oid IS NULL OR NOT EXISTS(SELECT 1 FROM selected_routines p
 JOIN stable_addresses address ON (address.classid,address.objid,address.objsubid)=('pg_catalog.pg_proc'::regclass::oid,p.oid,0)
 WHERE p.oid=required.oid AND address.valid IS TRUE AND address.address=required.expected_address)))
 AND (SELECT count(*) FROM selected_relations)=CASE WHEN EXISTS(SELECT 1 FROM selected_relations WHERE relname IN (SELECT name FROM group_wanted)) THEN 83 ELSE 76 END
 AND NOT EXISTS(SELECT 1 FROM selected_relations WHERE relkind<>'r' OR relispartition)
 AND NOT EXISTS(SELECT 1 FROM attribute_records WHERE valid IS NOT TRUE)
 AND NOT EXISTS(SELECT 1 FROM default_records WHERE valid IS NOT TRUE)
 AND NOT EXISTS(SELECT 1 FROM ordinary_edge_records WHERE valid IS NOT TRUE)
 AND NOT EXISTS(SELECT 1 FROM shared_edge_records WHERE valid IS NOT TRUE)
 AND (SELECT count(*) FROM comment_records)=(SELECT count(*) FROM metadata_m WHERE classid<>'pg_catalog.pg_authid'::regclass)
 AND (SELECT count(*) FROM shared_comment_records)=(SELECT count(*) FROM metadata_m WHERE classid='pg_catalog.pg_authid'::regclass)
 AND (SELECT count(*) FROM ordinary_edge_records)=(SELECT count(*) FROM ordinary_edges)
 AND (SELECT count(*) FROM shared_edge_records)=(SELECT count(*) FROM shared_edges)
 AND h76.snapshot_sha256=encode(sha256(convert_to(h76.snapshot::text,'UTF8')),'hex')
 AND h83.snapshot_sha256=encode(sha256(convert_to(h83.snapshot::text,'UTF8')),'hex')
 AND h73.snapshot_sha256=encode(sha256(convert_to(h73.snapshot::text,'UTF8')),'hex')
 AND h76.native_directory_startup_rights_valid IS FALSE
 AND CASE WHEN EXISTS(SELECT 1 FROM selected_relations WHERE relname IN (SELECT name FROM group_wanted))
 THEN h83.native_group_process_startup_rights_valid IS TRUE ELSE h73.native_directory_startup_rights_valid IS TRUE END
 AND pg_catalog.current_setting('search_path')='pg_catalog, pg_temp' AND pg_catalog.current_setting('jit')='off') IS TRUE AS complete_capture_valid
 FROM historical76_query h76 CROSS JOIN historical83_query h83 CROSS JOIN original73_query h73 CROSS JOIN supplemental s
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,complete_capture_valid FROM complete_snapshot;
