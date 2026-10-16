-- UNINSTALLED additive private lock source. Exact successor custody is required.
-- This primitive retains only the selected Group head and Group row, before
-- Account/family. It neither locks the Company row nor establishes a permit.
CREATE FUNCTION public.identity_company_information_group_lock_v1(p_company uuid,p_group uuid)
RETURNS TABLE(group_row jsonb,group_head_row jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid;
 head record; selected_group public.groups;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_company IS NULL OR p_group IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 IF NOT FOUND OR planned_group IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 SELECT * INTO STRICT head FROM public.group_authority_lock_shared_v1(p_group);
 SELECT g.* INTO STRICT selected_group FROM public.groups g WHERE g.id=p_group FOR SHARE OF g;
 -- Do not acquire a Company lock in this Group-class primitive.
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 IF NOT FOUND OR planned_group IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 IF head.revision IS NULL OR head.revision<1 OR head.incarnation IS NULL
  OR head.incarnation='00000000-0000-0000-0000-000000000000'::uuid
  OR head.state NOT IN('ACTIVE','RETIRED') OR head.state IS NULL
  OR selected_group.id IS DISTINCT FROM p_group
  OR selected_group.status NOT IN('ACTIVE','SUSPENDED','ARCHIVED') OR selected_group.status IS NULL
  OR num_nonnulls(selected_group.origin_account_id,selected_group.origin_command_id,
      selected_group.origin_receipt_id) NOT IN(0,3)
  OR selected_group.created_at IS NULL OR selected_group.updated_at IS NULL
  OR NOT isfinite(selected_group.created_at) OR NOT isfinite(selected_group.updated_at)
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(selected_group.origin_account_id,
      selected_group.origin_command_id,selected_group.origin_receipt_id) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 RETURN QUERY SELECT to_jsonb(selected_group),jsonb_build_object('group_id',p_group,
  'revision',head.revision,'incarnation',head.incarnation,'state',head.state);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
