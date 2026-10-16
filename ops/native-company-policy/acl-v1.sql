-- Closed routine/role roster; no serving raw-table access or inherited defaults.
ALTER FUNCTION public.native_company_policy_decode_v1(bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_decode_v1(bytea) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_decode_v1(bytea)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_decode_v1(bytea) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_registration_custody_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_registration_custody_v1(uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_registration_custody_v1(uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_registration_custody_v1(uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.identity_native_policy_material_v1(uuid,uuid,uuid,uuid,smallint,bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_policy_material_v1(uuid,uuid,uuid,uuid,smallint,bytea) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_policy_material_v1(uuid,uuid,uuid,uuid,smallint,bytea)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_policy_material_v1(uuid,uuid,uuid,uuid,smallint,bytea) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_policy_material_v1(uuid,uuid,uuid,uuid,smallint,bytea) TO console_rt;
ALTER FUNCTION public.native_company_policy_operation_check_v1(uuid,bytea,timestamptz) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_operation_check_v1(uuid,bytea,timestamptz) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_operation_check_v1(uuid,bytea,timestamptz)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_operation_check_v1(uuid,bytea,timestamptz) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_preflight_v1(uuid,uuid,uuid,uuid,smallint,bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_preflight_v1(uuid,uuid,uuid,uuid,smallint,bytea) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_preflight_v1(uuid,uuid,uuid,uuid,smallint,bytea)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_preflight_v1(uuid,uuid,uuid,uuid,smallint,bytea) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_preflight_v1(uuid,uuid,uuid,uuid,smallint,bytea) TO console_rt;
ALTER FUNCTION public.native_company_policy_capacity_v1(uuid,uuid,timestamptz) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_capacity_v1(uuid,uuid,timestamptz) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_capacity_v1(uuid,uuid,timestamptz)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_capacity_v1(uuid,uuid,timestamptz) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_prepare_v1(uuid,uuid,uuid,uuid,smallint,bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_prepare_v1(uuid,uuid,uuid,uuid,smallint,bytea) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_prepare_v1(uuid,uuid,uuid,uuid,smallint,bytea)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_prepare_v1(uuid,uuid,uuid,uuid,smallint,bytea) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_prepare_v1(uuid,uuid,uuid,uuid,smallint,bytea) TO console_rt;
ALTER FUNCTION public.native_company_policy_execute_v1(uuid,uuid,uuid,uuid,smallint) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_execute_v1(uuid,uuid,uuid,uuid,smallint) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_execute_v1(uuid,uuid,uuid,uuid,smallint)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_execute_v1(uuid,uuid,uuid,uuid,smallint) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_execute_v1(uuid,uuid,uuid,uuid,smallint) TO console_rt;
ALTER FUNCTION public.native_company_policy_status_v1(uuid,uuid,uuid,uuid,smallint) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_status_v1(uuid,uuid,uuid,uuid,smallint) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_status_v1(uuid,uuid,uuid,uuid,smallint)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_status_v1(uuid,uuid,uuid,uuid,smallint) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_status_v1(uuid,uuid,uuid,uuid,smallint) TO console_rt;
ALTER FUNCTION public.native_company_policy_form_v1(uuid,uuid,uuid,uuid,smallint) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_form_v1(uuid,uuid,uuid,uuid,smallint) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_form_v1(uuid,uuid,uuid,uuid,smallint)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_form_v1(uuid,uuid,uuid,uuid,smallint) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_form_v1(uuid,uuid,uuid,uuid,smallint) TO console_rt;
ALTER FUNCTION public.native_company_policy_manifest_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_manifest_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_manifest_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_manifest_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_manifest_v1() TO console_ontology_writer;
ALTER FUNCTION public.native_company_policy_assert_payroll_catalog_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_assert_payroll_catalog_v1(uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_assert_payroll_catalog_v1(uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_assert_payroll_catalog_v1(uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_assert_payroll_catalog_v1(uuid) TO console_ontology_writer;
ALTER FUNCTION ontology_api.install_native_company_payroll_catalog_v1(uuid,uuid) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.install_native_company_payroll_catalog_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.install_native_company_payroll_catalog_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_ontology_writer' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.install_native_company_payroll_catalog_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION ontology_api.install_native_company_payroll_catalog_v1(uuid,uuid) TO console_account_owner;
ALTER FUNCTION public.native_company_policy_clause_v1(uuid,timestamptz) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_clause_v1(uuid,timestamptz) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_clause_v1(uuid,timestamptz)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_clause_v1(uuid,timestamptz) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_apply_assignment_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_apply_assignment_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_apply_assignment_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_apply_assignment_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_business_clauses_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_business_clauses_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_business_clauses_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_business_clauses_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_immutable_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_immutable_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_immutable_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_input_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_input_guard_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_input_guard_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_input_guard_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_receipt_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_receipt_guard_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_receipt_guard_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_receipt_guard_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_effect_frame_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_effect_frame_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_effect_frame_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_effect_frame_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_effect_frame_v1(uuid,uuid) TO console_ontology_writer;
ALTER FUNCTION public.native_company_policy_participant_receipt_v1(text,jsonb) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_participant_receipt_v1(text,jsonb) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_participant_receipt_v1(text,jsonb)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_participant_receipt_v1(text,jsonb) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_participant_admit_v1(text,text,jsonb,jsonb) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_participant_admit_v1(text,text,jsonb,jsonb) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_participant_admit_v1(text,text,jsonb,jsonb)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_participant_admit_v1(text,text,jsonb,jsonb) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_participant_admit_v1(text,text,jsonb,jsonb) TO console_ontology_writer;
ALTER FUNCTION public.native_company_policy_head_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_head_guard_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_head_guard_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_head_guard_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_accept_snapshot_v1(public.native_company_policy_inputs_v1) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_accept_snapshot_v1(public.native_company_policy_inputs_v1) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_accept_snapshot_v1(public.native_company_policy_inputs_v1)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_accept_snapshot_v1(public.native_company_policy_inputs_v1) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_complete_snapshot_v1(public.native_company_policy_receipts_v1) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_complete_snapshot_v1(public.native_company_policy_receipts_v1) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_complete_snapshot_v1(public.native_company_policy_receipts_v1)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_complete_snapshot_v1(public.native_company_policy_receipts_v1) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_assert_current_head_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_assert_current_head_v1(uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_assert_current_head_v1(uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_assert_current_head_v1(uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_assert_effects_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_assert_effects_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_assert_effects_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_assert_effects_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_input_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_input_closure_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_input_closure_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_input_closure_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_assert_terminal_closure_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_assert_terminal_closure_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_assert_terminal_closure_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_assert_terminal_closure_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_receipt_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_receipt_closure_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_receipt_closure_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_receipt_closure_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_participant_closed_v1(text,jsonb) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_participant_closed_v1(text,jsonb) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_participant_closed_v1(text,jsonb)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_participant_closed_v1(text,jsonb) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_participant_closed_v1(text,jsonb) TO console_ontology_writer;
ALTER FUNCTION public.native_company_policy_ontology_snapshot_v1(public.native_company_policy_receipts_v1) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_ontology_snapshot_v1(public.native_company_policy_receipts_v1) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_ontology_snapshot_v1(public.native_company_policy_receipts_v1)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_ontology_snapshot_v1(public.native_company_policy_receipts_v1) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_audit_admit_v1(public.audit_events) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_audit_admit_v1(public.audit_events) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_audit_admit_v1(public.audit_events)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_audit_admit_v1(public.audit_events) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.native_company_policy_audit_admit_v1(public.audit_events) TO console_ontology_writer;
ALTER FUNCTION public.native_company_policy_audit_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_audit_guard_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_audit_guard_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_audit_guard_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_assert_ontology_audit_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_assert_ontology_audit_v1(uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_assert_ontology_audit_v1(uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_assert_ontology_audit_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION ontology_api.lock_native_company_catalog_current_v2(uuid) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.lock_native_company_catalog_current_v2(uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.lock_native_company_catalog_current_v2(uuid)'::regprocedure AND r.rolname<>'console_ontology_writer' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.lock_native_company_catalog_current_v2(uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION ontology_api.lock_native_company_catalog_current_v2(uuid) TO console_account_owner;
ALTER FUNCTION public.identity_company_projection_v2(uuid,uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_company_projection_v2(uuid,uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_company_projection_v2(uuid,uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_company_projection_v2(uuid,uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_projection_v2(uuid,uuid,uuid) TO console_rt;
ALTER FUNCTION public.identity_company_payroll_projection_v1(uuid,uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_company_payroll_projection_v1(uuid,uuid,uuid) FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_company_payroll_projection_v1(uuid,uuid,uuid)'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_company_payroll_projection_v1(uuid,uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_payroll_projection_v1(uuid,uuid,uuid) TO console_rt;
ALTER FUNCTION public.identity_native_root_guard_v2() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_root_guard_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_root_guard_v2()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_root_guard_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.identity_native_birth_row_guard_v2() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_birth_row_guard_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_birth_row_guard_v2()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_birth_row_guard_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.identity_native_birth_closure_v2() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_birth_closure_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_birth_closure_v2()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_birth_closure_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_catalog_birth_row_guard_v2() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_catalog_birth_row_guard_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_catalog_birth_row_guard_v2()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_catalog_birth_row_guard_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_catalog_birth_closure_v2() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_catalog_birth_closure_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_catalog_birth_closure_v2()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_catalog_birth_closure_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION ontology_api.native_catalog_attribution_guard_v2() OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.native_catalog_attribution_guard_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.native_catalog_attribution_guard_v2()'::regprocedure AND r.rolname<>'console_ontology_writer' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.native_catalog_attribution_guard_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.company_enrollment_ontology_audit_guard_v2() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_ontology_audit_guard_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_ontology_audit_guard_v2()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_ontology_audit_guard_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION ontology_api.protected_audit_writer_guard_v2() OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.protected_audit_writer_guard_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.protected_audit_writer_guard_v2()'::regprocedure AND r.rolname<>'console_ontology_writer' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.protected_audit_writer_guard_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION ontology_api.require_current_transaction_audit_v2() OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.require_current_transaction_audit_v2() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.require_current_transaction_audit_v2()'::regprocedure AND r.rolname<>'console_ontology_writer' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.require_current_transaction_audit_v2() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_participant_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_participant_guard_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_participant_guard_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_participant_guard_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
ALTER FUNCTION public.native_company_policy_participant_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_participant_closure_v1() FROM PUBLIC;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_policy_participant_closure_v1()'::regprocedure AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_policy_participant_closure_v1() FROM %I',grantee_name);
 END LOOP;
END $acl$;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.native_company_policy_inputs_v1'::regclass AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON public.native_company_policy_inputs_v1 FROM %I',grantee_name);
 END LOOP;
END $acl$;
DO $acl$ DECLARE grantee_name text; BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.native_company_policy_receipts_v1'::regclass AND r.rolname<>'console_account_owner' LOOP
  EXECUTE format('REVOKE ALL ON public.native_company_policy_receipts_v1 FROM %I',grantee_name);
 END LOOP;
END $acl$;
GRANT SELECT ON public.ont_object_types,public.ont_object_type_key_revisions,public.ont_property_defs,
 public.ont_action_types,public.ont_builtin_catalog_installs TO console_account_owner;
GRANT REFERENCES ON public.native_company_policy_receipts_v1 TO console_ontology_writer;
GRANT INSERT(policy_receipt_id) ON public.policy_roles,public.user_role_assignments TO console_account_owner;
GRANT INSERT(policy_receipt_id) ON public.ont_object_types,public.ont_builtin_catalog_installs TO console_ontology_writer;
GRANT UPDATE(current_policy_receipt_id) ON public.company_authority_heads TO console_account_owner;

-- Deferred audit closure verifies the current transaction without broad table reads.
GRANT SELECT(xmin) ON public.audit_events TO console_account_owner;
