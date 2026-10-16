-- UNEXECUTED privileged successor source, not standalone operator authorization.
-- Outer reviewed custody owner MUST verify exact protected predecessor+staged230,
-- take existing deployment/admission locks, and apply the complete module set in
-- ONE transaction. No historical fingerprint/profile is changed or resealed.
-- Prerequisite order: migration230, codec-v3, reviewed current-source-v2,
-- commands-v2, guards-v2, closure-audit-v2, import-replay-guard-v2, then this file.
DO $precondition$
BEGIN
 IF current_setting('server_encoding')<>'UTF8'
  OR EXISTS(SELECT 1 FROM public.native_people_inputs_v1)
  OR EXISTS(SELECT 1 FROM public.native_people_terminals_v1) THEN
  RAISE EXCEPTION 'people.directory.activation_precondition_invalid';
 END IF;
END
$precondition$;

-- The dormant numbered migration retains CHECK(false) and no runtime rights.
-- Add every Company boundary before any owner grant or staged gate is opened.
ALTER TABLE public.native_people_inputs_v1 ADD CONSTRAINT native_people_inputs_org_v1
 FOREIGN KEY(org_id) REFERENCES public.organizations(id) ON UPDATE RESTRICT ON DELETE RESTRICT;

ALTER TABLE public.native_people_inputs_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_people_terminals_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_people_inputs_v1 ADD CONSTRAINT native_people_inputs_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.native_people_terminals_v1 ADD CONSTRAINT native_people_terminal_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.person_revisions ADD CONSTRAINT person_revisions_native_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.employee_person_bindings ADD CONSTRAINT employee_person_bindings_native_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.ont_action_command_receipts ADD CONSTRAINT ont_action_receipts_native_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;

ALTER TABLE public.person_revisions ADD CONSTRAINT person_revisions_actor_protocol_v1 CHECK((
 (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
 OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL)) IS TRUE);
ALTER TABLE public.employee_person_bindings ADD CONSTRAINT employee_person_bindings_actor_protocol_v1 CHECK((
 (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
 OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL)) IS TRUE);
ALTER TABLE public.ont_action_command_receipts ADD CONSTRAINT ont_action_receipts_actor_protocol_v1 CHECK((
 (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
 OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL
  AND owner='person' AND target='people.create_person' AND action_key='directory_create' AND object_type_id IS NOT NULL)) IS TRUE);
ALTER TABLE public.employees ADD CONSTRAINT employees_provenance_protocol_v1 CHECK((
 (source_kind='LEGACY' AND native_command_id IS NULL AND source_filename IS NOT NULL AND source_sheet IS NOT NULL AND source_row IS NOT NULL
  AND btrim(source_filename)<>'' AND btrim(source_sheet)<>'' AND source_row>0)
 OR (source_kind='NATIVE_DIRECTORY' AND native_command_id IS NOT NULL AND source_filename IS NULL AND source_sheet IS NULL AND source_row IS NULL
  AND source_key='native-directory:'||native_command_id::text)) IS TRUE);

CREATE POLICY native_people_owner_v1 ON public.native_people_inputs_v1 TO console_account_owner
 USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid)
 WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
CREATE POLICY native_people_owner_v1 ON public.native_people_terminals_v1 TO console_account_owner
 USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid)
 WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
-- FORCE RLS from numbered230 remains. No runtime SELECT/INSERT/UPDATE/DELETE on
-- protected tables; reads and transitions use checked functions only.
REVOKE ALL ON public.native_people_inputs_v1,public.native_people_terminals_v1 FROM PUBLIC,console_rt,console_app;
GRANT SELECT,INSERT ON public.native_people_inputs_v1,public.native_people_terminals_v1 TO console_account_owner;

-- Exact read-only columns used by validators; no new canonical DML grants.
GRANT SELECT(id,org_id,company,name,employee_number,source_kind,native_command_id,source_key,raw_row,source_metadata,created_at,updated_at,
 employment_status,identity_resolution_strategy,identity_resolution_confidence,identity_review_required,identity_name_only_merge,
 source_filename,source_sheet,source_row,home_branch_id,hire_date,exit_date,org_unit,job,position,worksite_name,worksite_address,
 leave_accrued,leave_used,leave_remaining,xmin) ON public.employees TO console_account_owner;
GRANT SELECT(org_id,id,created_at,xmin) ON public.persons TO console_account_owner;
GRANT SELECT(org_id,id,person_id,version,command_id,actor_kind,actor_id,actor_account_id,payload_digest,attributes,receipt,created_at,xmin)
 ON public.person_revisions TO console_account_owner;
GRANT SELECT(org_id,employee_id,person_id,actor_kind,actor_id,actor_account_id,payload_digest,created_at,xmin)
 ON public.employee_person_bindings TO console_account_owner;
GRANT SELECT(org_id,command_id,actor_kind,actor_id,actor_account_id,payload_digest,receipt,created_at,owner,target,action_key,object_type_id,xmin)
 ON public.ont_action_command_receipts TO console_account_owner;
GRANT SELECT(org_id,employee_id) ON public.employee_employment_profiles,public.employee_lifecycle_events,
 public.employment_source_bindings,public.leave_balance_import_receipts TO console_account_owner;
GRANT SELECT(org_id,command_id) ON public.employment_revisions TO console_account_owner;
-- organizations and audit_events read custody is already held by existing Account
-- policy owner; no relaxation to their writers or schemas is introduced here.

CREATE TRIGGER native_people_input_frame_v1 BEFORE INSERT ON public.native_people_inputs_v1
 FOR EACH ROW EXECUTE FUNCTION public.native_people_input_guard_v1();
CREATE TRIGGER native_people_terminal_frame_v1 BEFORE INSERT ON public.native_people_terminals_v1
 FOR EACH ROW EXECUTE FUNCTION public.native_people_terminal_guard_v1();
CREATE TRIGGER native_people_employee_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employees
 FOR EACH ROW EXECUTE FUNCTION public.native_people_employee_guard_v1();
CREATE TRIGGER native_people_person_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.persons
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
CREATE TRIGGER native_people_revision_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.person_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
CREATE TRIGGER native_people_binding_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employee_person_bindings
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
CREATE TRIGGER native_people_receipt_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_action_command_receipts
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
DROP TRIGGER trg_console_employee_number_unique ON public.employees;
CREATE TRIGGER trg_console_employee_number_unique BEFORE INSERT OR UPDATE OF org_id,employee_number ON public.employees
 FOR EACH ROW EXECUTE FUNCTION public.console_employee_number_unique();
CREATE TRIGGER native_people_audit_frame_v1 BEFORE INSERT ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.native_people_audit_guard_v1();
CREATE TRIGGER native_people_employee_employment_profiles_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employee_employment_profiles
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.employee_employment_profiles ENABLE ALWAYS TRIGGER native_people_employee_employment_profiles_guard_v1;
CREATE TRIGGER native_people_employee_lifecycle_events_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employee_lifecycle_events
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.employee_lifecycle_events ENABLE ALWAYS TRIGGER native_people_employee_lifecycle_events_guard_v1;
CREATE TRIGGER native_people_employment_source_bindings_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employment_source_bindings
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.employment_source_bindings ENABLE ALWAYS TRIGGER native_people_employment_source_bindings_guard_v1;
CREATE TRIGGER native_people_leave_balance_import_receipts_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.leave_balance_import_receipts
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.leave_balance_import_receipts ENABLE ALWAYS TRIGGER native_people_leave_balance_import_receipts_guard_v1;
CREATE CONSTRAINT TRIGGER native_people_input_closure_v1 AFTER INSERT ON public.native_people_inputs_v1
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_people_deferred_closure_v1();
CREATE CONSTRAINT TRIGGER native_people_terminal_closure_v1 AFTER INSERT ON public.native_people_terminals_v1
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_people_deferred_closure_v1();

ALTER TABLE public.native_people_inputs_v1 ENABLE ALWAYS TRIGGER native_people_input_frame_v1;
ALTER TABLE public.native_people_inputs_v1 ENABLE ALWAYS TRIGGER native_people_input_closure_v1;
ALTER TABLE public.native_people_terminals_v1 ENABLE ALWAYS TRIGGER native_people_terminal_frame_v1;
ALTER TABLE public.native_people_terminals_v1 ENABLE ALWAYS TRIGGER native_people_terminal_closure_v1;
ALTER TABLE public.employees ENABLE ALWAYS TRIGGER native_people_employee_frame_v1;
ALTER TABLE public.employees ENABLE ALWAYS TRIGGER trg_console_employee_number_unique;
ALTER TABLE public.persons ENABLE ALWAYS TRIGGER native_people_person_frame_v1;
ALTER TABLE public.person_revisions ENABLE ALWAYS TRIGGER native_people_revision_frame_v1;
ALTER TABLE public.employee_person_bindings ENABLE ALWAYS TRIGGER native_people_binding_frame_v1;
ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER native_people_receipt_frame_v1;
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_people_audit_frame_v1;

DO $function_acl$
DECLARE signature text; grantee_name text;
BEGIN
 FOREACH signature IN ARRAY ARRAY[
  'public.native_people_history_immutable_v1()',
  'public.native_people_text_valid_v1(text,integer)',
  'public.native_people_encode_v1(public.native_people_inputs_v1)',
  'public.native_people_decode_v1(smallint,bytea)',
  'public.native_people_effect_digest_v1(public.native_people_inputs_v1)',
  'public.native_people_result_v1(public.native_people_inputs_v1)',
  'public.native_people_expectations_match_v1(public.native_people_inputs_v1,jsonb)',
  'public.native_people_current_v1(uuid,uuid,uuid)',
  'public.native_people_prepare_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text)',
  'public.native_people_preflight_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text)',
  'public.native_people_terminal_open_v1(uuid,uuid,uuid,uuid,text)',
  'public.native_people_frame_v1(uuid,uuid)',
  'public.native_people_employee_shape_v1(public.employees,public.native_people_inputs_v1,public.native_people_terminals_v1)',
  'public.native_people_employee_guard_v1()',
  'public.native_people_canonical_guard_v1()',
  'public.native_people_non_directory_effect_guard_v1()',
  'public.console_employee_number_unique()',
  'public.native_people_accept_snapshot_v1(public.native_people_inputs_v1)',
  'public.native_people_terminal_snapshot_v1(public.native_people_terminals_v1)',
  'public.native_people_audit_material_v1(uuid,uuid,uuid,uuid,text)',
  'public.native_people_audit_guard_v1()',
  'public.native_people_input_guard_v1()',
  'public.native_people_terminal_guard_v1()',
  'public.native_people_assert_closed_v1(uuid,uuid,text)',
  'public.native_people_deferred_closure_v1()'
 ] LOOP
  EXECUTE format('ALTER FUNCTION %s OWNER TO console_account_owner',signature);
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',signature);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_catalog.pg_proc p
   CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a JOIN pg_catalog.pg_roles r ON r.oid=a.grantee
   WHERE p.oid=signature::regprocedure AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',signature,grantee_name); END LOOP;
 END LOOP;
END
$function_acl$;
GRANT EXECUTE ON FUNCTION
 public.native_people_prepare_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text),
 public.native_people_preflight_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text),
 public.native_people_terminal_open_v1(uuid,uuid,uuid,uuid,text),
 public.native_people_audit_material_v1(uuid,uuid,uuid,uuid,text) TO console_rt;
-- Current source already has its separately reviewed exact owner/EXECUTE ACL.
-- Existing leave import routine retains its prior console_leave_definer owner/ACL;
-- CREATE OR REPLACE changed only its pre-replay native-target refusal.

-- Open staged gates only after complete guards/closure/RLS/ACL are installed.
ALTER TABLE public.person_revisions DROP CONSTRAINT person_revisions_actor_staged_user_v1;
ALTER TABLE public.employee_person_bindings DROP CONSTRAINT employee_person_bindings_actor_staged_user_v1;
ALTER TABLE public.ont_action_command_receipts DROP CONSTRAINT ont_action_receipts_actor_staged_user_v1;
ALTER TABLE public.employees DROP CONSTRAINT employees_provenance_staged_legacy_v1;
ALTER TABLE public.native_people_inputs_v1 DROP CONSTRAINT native_people_inputs_staged_closed_v1;
ALTER TABLE public.native_people_terminals_v1 DROP CONSTRAINT native_people_terminals_staged_closed_v1;
-- Outer successor captures exact installed shape/functions/ACL/ALWAYS triggers,
-- binds resulting hashes only to the NEW profile, and commits atomically.
