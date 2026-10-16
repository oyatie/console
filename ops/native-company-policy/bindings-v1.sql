
CREATE FUNCTION public.native_company_policy_participant_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$ BEGIN
 IF current_user NOT IN ('console_account_owner','console_ontology_writer') THEN
  IF NEW.policy_receipt_id IS NOT NULL THEN RAISE EXCEPTION 'native_company_policy.owner_required'; END IF;
  RETURN NEW;
 END IF;
 PERFORM public.native_company_policy_participant_admit_v1(TG_TABLE_NAME,TG_OP,
  CASE WHEN TG_OP<>'INSERT' THEN to_jsonb(OLD) END,CASE WHEN TG_OP<>'DELETE' THEN to_jsonb(NEW) END);
 RETURN NEW;
END $body$;
CREATE FUNCTION public.native_company_policy_participant_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$ BEGIN
 PERFORM public.native_company_policy_participant_closed_v1(TG_TABLE_NAME,to_jsonb(NEW)); RETURN NEW;
END $body$;
CREATE TRIGGER native_company_policy_inputs_v1_guard BEFORE INSERT ON public.native_company_policy_inputs_v1
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_input_guard_v1();
ALTER TABLE public.native_company_policy_inputs_v1 ENABLE ALWAYS TRIGGER native_company_policy_inputs_v1_guard;
CREATE TRIGGER native_company_policy_inputs_v1_immutable BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_company_policy_inputs_v1
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_company_policy_immutable_v1();
ALTER TABLE public.native_company_policy_inputs_v1 ENABLE ALWAYS TRIGGER native_company_policy_inputs_v1_immutable;
CREATE CONSTRAINT TRIGGER native_company_policy_inputs_v1_closure AFTER INSERT ON public.native_company_policy_inputs_v1
 DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_input_closure_v1();
ALTER TABLE public.native_company_policy_inputs_v1 ENABLE ALWAYS TRIGGER native_company_policy_inputs_v1_closure;
CREATE TRIGGER native_company_policy_receipts_v1_guard BEFORE INSERT ON public.native_company_policy_receipts_v1
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_receipt_guard_v1();
ALTER TABLE public.native_company_policy_receipts_v1 ENABLE ALWAYS TRIGGER native_company_policy_receipts_v1_guard;
CREATE TRIGGER native_company_policy_receipts_v1_immutable BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_company_policy_receipts_v1
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_company_policy_immutable_v1();
ALTER TABLE public.native_company_policy_receipts_v1 ENABLE ALWAYS TRIGGER native_company_policy_receipts_v1_immutable;
CREATE CONSTRAINT TRIGGER native_company_policy_receipts_v1_closure AFTER INSERT ON public.native_company_policy_receipts_v1
 DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_receipt_closure_v1();
ALTER TABLE public.native_company_policy_receipts_v1 ENABLE ALWAYS TRIGGER native_company_policy_receipts_v1_closure;
CREATE TRIGGER policy_roles_policy_guard_v1 BEFORE INSERT ON public.policy_roles
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.policy_roles ENABLE ALWAYS TRIGGER policy_roles_policy_guard_v1;
CREATE CONSTRAINT TRIGGER policy_roles_policy_closure_v1 AFTER INSERT ON public.policy_roles
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.policy_roles ENABLE ALWAYS TRIGGER policy_roles_policy_closure_v1;
CREATE TRIGGER policy_role_revisions_policy_guard_v1 BEFORE INSERT ON public.policy_role_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.policy_role_revisions ENABLE ALWAYS TRIGGER policy_role_revisions_policy_guard_v1;
CREATE CONSTRAINT TRIGGER policy_role_revisions_policy_closure_v1 AFTER INSERT ON public.policy_role_revisions
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.policy_role_revisions ENABLE ALWAYS TRIGGER policy_role_revisions_policy_closure_v1;
CREATE TRIGGER user_role_assignments_policy_guard_v1 BEFORE INSERT ON public.user_role_assignments
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.user_role_assignments ENABLE ALWAYS TRIGGER user_role_assignments_policy_guard_v1;
CREATE CONSTRAINT TRIGGER user_role_assignments_policy_closure_v1 AFTER INSERT ON public.user_role_assignments
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.user_role_assignments ENABLE ALWAYS TRIGGER user_role_assignments_policy_closure_v1;
CREATE TRIGGER policy_assignment_revisions_policy_guard_v1 BEFORE INSERT ON public.policy_assignment_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.policy_assignment_revisions ENABLE ALWAYS TRIGGER policy_assignment_revisions_policy_guard_v1;
CREATE CONSTRAINT TRIGGER policy_assignment_revisions_policy_closure_v1 AFTER INSERT ON public.policy_assignment_revisions
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.policy_assignment_revisions ENABLE ALWAYS TRIGGER policy_assignment_revisions_policy_closure_v1;
CREATE TRIGGER native_company_catalog_installs_policy_guard_v1 BEFORE INSERT ON public.native_company_catalog_installs
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.native_company_catalog_installs ENABLE ALWAYS TRIGGER native_company_catalog_installs_policy_guard_v1;
CREATE CONSTRAINT TRIGGER native_company_catalog_installs_policy_closure_v1 AFTER INSERT ON public.native_company_catalog_installs
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.native_company_catalog_installs ENABLE ALWAYS TRIGGER native_company_catalog_installs_policy_closure_v1;
CREATE TRIGGER ont_builtin_catalog_installs_policy_guard_v1 BEFORE INSERT ON public.ont_builtin_catalog_installs
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.ont_builtin_catalog_installs ENABLE ALWAYS TRIGGER ont_builtin_catalog_installs_policy_guard_v1;
CREATE CONSTRAINT TRIGGER ont_builtin_catalog_installs_policy_closure_v1 AFTER INSERT ON public.ont_builtin_catalog_installs
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.ont_builtin_catalog_installs ENABLE ALWAYS TRIGGER ont_builtin_catalog_installs_policy_closure_v1;
CREATE TRIGGER ont_object_types_policy_guard_v1 BEFORE INSERT ON public.ont_object_types
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_guard_v1();
ALTER TABLE public.ont_object_types ENABLE ALWAYS TRIGGER ont_object_types_policy_guard_v1;
CREATE CONSTRAINT TRIGGER ont_object_types_policy_closure_v1 AFTER INSERT ON public.ont_object_types
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.ont_object_types ENABLE ALWAYS TRIGGER ont_object_types_policy_closure_v1;
DROP TRIGGER company_authority_heads_immutable_v1 ON public.company_authority_heads;
CREATE TRIGGER company_authority_heads_immutable_v1 BEFORE DELETE OR TRUNCATE ON public.company_authority_heads
 FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
ALTER TABLE public.company_authority_heads ENABLE ALWAYS TRIGGER company_authority_heads_immutable_v1;
CREATE TRIGGER company_authority_heads_policy_update_v1 BEFORE UPDATE ON public.company_authority_heads
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_head_guard_v1();
ALTER TABLE public.company_authority_heads ENABLE ALWAYS TRIGGER company_authority_heads_policy_update_v1;
CREATE CONSTRAINT TRIGGER company_authority_heads_policy_closure_v1 AFTER UPDATE ON public.company_authority_heads
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.company_authority_heads ENABLE ALWAYS TRIGGER company_authority_heads_policy_closure_v1;
CREATE CONSTRAINT TRIGGER user_role_assignments_policy_update_closure_v1 AFTER UPDATE ON public.user_role_assignments
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_participant_closure_v1();
ALTER TABLE public.user_role_assignments ENABLE ALWAYS TRIGGER user_role_assignments_policy_update_closure_v1;
CREATE TRIGGER native_company_policy_audit_guard_v1 BEFORE INSERT ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.native_company_policy_audit_guard_v1();
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_company_policy_audit_guard_v1;
