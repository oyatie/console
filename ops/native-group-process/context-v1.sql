-- Uninstalled current-identity helper shared by full projections and provisional
-- effect guards. It never asserts incomplete B rows, issues proofs or authorizes
-- a caller; Auth/Cedar and consuming finish remain the retained adapter boundary.
CREATE FUNCTION public.native_group_process_current_context_v1(
 p_account uuid,p_family uuid,p_group uuid,p_incarnation uuid,p_write boolean) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE account_material jsonb; topology public.group_authority_heads; group_row public.groups;
 designation public.deployment_operator_head; birth public.company_enrollment_receipts;
 provenance public.deployment_operator_receipts;
BEGIN
 PERFORM public.native_group_process_group_guard_v1(p_group,p_write);
 PERFORM 1 FROM public.group_authority_lock_shared_v1(p_group);
 account_material:=public.native_group_process_account_material_v1(p_account,p_family);
 IF NOT public.account_company_setup_eligibility_v1(p_account) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
 END IF;
 SELECT h.* INTO STRICT designation FROM public.deployment_operator_head h
  WHERE h.singleton=1 AND h.account_id=p_account;
 SELECT g.* INTO STRICT group_row FROM public.groups g WHERE g.id=p_group;
 SELECT h.* INTO STRICT topology FROM public.group_authority_heads h WHERE h.group_id=p_group;
 IF topology.incarnation IS DISTINCT FROM p_incarnation OR topology.state IS DISTINCT FROM 'ACTIVE'
  OR group_row.status IS DISTINCT FROM 'ACTIVE' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
 END IF;
 SELECT r.* INTO STRICT birth FROM public.company_enrollment_receipts r
  JOIN public.company_enrollment_requests q ON q.account_id=r.account_id AND q.command_id=r.command_id
   AND q.committed_receipt_id=r.receipt_id AND q.state='COMMITTED'
  WHERE (r.group_id,r.account_id,r.command_id,r.receipt_id)=
   (p_group,group_row.origin_account_id,group_row.origin_command_id,group_row.origin_receipt_id);
 SELECT r.* INTO STRICT provenance FROM public.deployment_operator_receipts r WHERE r.receipt_id=birth.designation_receipt_id;
 IF (provenance.system_identifier,provenance.database_name,provenance.database_oid)
  IS DISTINCT FROM (designation.system_identifier,designation.database_name,designation.database_oid) THEN
  RAISE EXCEPTION 'native_group_process.source_unavailable';
 END IF;
 RETURN jsonb_build_object('account',account_material,'group',to_jsonb(group_row),
  'topology',to_jsonb(topology),'designation',to_jsonb(designation),
  'group_deployment',jsonb_build_object('system_identifier',provenance.system_identifier,
   'database_name',provenance.database_name,'database_oid',provenance.database_oid),
  'source',public.native_group_process_source_v1(p_group,p_incarnation));
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;
