-- UNINSTALLED private selected-row guard. No sibling/founding Company scan.
-- The manager caller already retains selected Group head+row, then Account/family.
-- This topology-owned helper supplies only the next requested Company guard class.
CREATE FUNCTION public.identity_company_information_selected_lock_v1(p_company uuid,p_group uuid)
RETURNS TABLE(company_row jsonb,membership_row jsonb,membership_revision_row jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); selected_company record;
 member public.group_memberships; history public.group_membership_revisions;
 receipt public.platform_legacy_topology_receipts; decoded record; head public.group_authority_heads;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_company IS NULL OR p_group IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.id,o.group_id,o.slug,o.name,o.status,o.created_at,o.updated_at,
  o.origin_account_id,o.origin_command_id,o.origin_receipt_id INTO STRICT selected_company
  FROM public.organizations o WHERE o.id=p_company FOR SHARE OF o;
 IF selected_company.group_id IS DISTINCT FROM p_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 SELECT m.* INTO STRICT member FROM public.group_memberships m
  WHERE m.org_id=p_company AND m.group_id=p_group FOR SHARE OF m;
 SELECT h.* INTO STRICT history FROM public.group_membership_revisions h
  WHERE (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)=
   (member.group_id,member.org_id,member.membership_id,member.current_revision,member.incarnation)
  FOR SHARE OF h;
 IF selected_company.status IS NULL OR selected_company.status NOT IN('ACTIVE','SUSPENDED','ARCHIVED')
  OR selected_company.created_at IS NULL OR selected_company.updated_at IS NULL
  OR NOT isfinite(selected_company.created_at) OR NOT isfinite(selected_company.updated_at)
  OR selected_company.updated_at<selected_company.created_at
  OR num_nonnulls(selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id) NOT IN(0,3)
  OR member.current_revision IS NULL OR member.current_revision<1
  OR member.membership_id IS NULL OR member.incarnation IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(member.membership_id,member.incarnation,
   selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id)
  OR history.state IS DISTINCT FROM 'ACTIVE' OR history.to_time IS NOT NULL
  OR history.from_time IS DISTINCT FROM member.created_at OR NOT isfinite(history.from_time) OR history.from_time>clock_timestamp() THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 CASE history.provenance_kind
 WHEN 'COMPANY_ENROLLMENT_V1' THEN
  IF history.revision<>1 OR history.legacy_actor_user_id IS NOT NULL OR history.force_actor_user_id IS NOT NULL
   OR (history.native_account_id,history.command_id,history.command_receipt) IS DISTINCT FROM
    (selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id)
   OR num_nonnulls(history.native_account_id,history.command_id,history.command_receipt)<>3 THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 WHEN 'LEGACY_BACKFILL' THEN
  IF history.revision<>1 OR num_nonnulls(selected_company.origin_account_id,selected_company.origin_command_id,
    selected_company.origin_receipt_id,history.native_account_id,history.legacy_actor_user_id,
    history.force_actor_user_id,history.command_id,history.command_receipt)<>0 THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 WHEN 'LEGACY_TOPOLOGY_V1' THEN
  IF num_nonnulls(selected_company.origin_account_id,selected_company.origin_command_id,selected_company.origin_receipt_id,
    history.native_account_id,history.force_actor_user_id)<>0 THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  SELECT r.* INTO STRICT receipt FROM public.platform_legacy_topology_receipts r
   WHERE (r.actor_user_id,r.command_id,r.receipt_id)=
    (history.legacy_actor_user_id,history.command_id,history.command_receipt);
  IF jsonb_typeof(receipt.heads_after) IS DISTINCT FROM 'array' THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  IF jsonb_array_length(receipt.heads_after) NOT BETWEEN 1 AND 2 THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(receipt.input_bytes);
  SELECT h.* INTO STRICT head FROM public.group_authority_heads h WHERE h.group_id=p_group;
  IF receipt.outcome IS DISTINCT FROM 'APPLIED' OR receipt.result_code IS DISTINCT FROM 'applied'
   OR receipt.kind NOT IN(1,4,5) OR receipt.codec_version<>1
   OR receipt.input_digest IS DISTINCT FROM sha256(receipt.input_bytes)
   OR (decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest) IS DISTINCT FROM
    (receipt.actor_user_id,receipt.command_id,receipt.kind,receipt.input_digest)
   OR receipt.result_org_id IS DISTINCT FROM p_company OR receipt.result_group_id IS DISTINCT FROM p_group
   OR receipt.occurred_at IS DISTINCT FROM history.from_time OR history.revision<>1
   OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(receipt.heads_after) entry
    WHERE (entry->>'group_id')::uuid=p_group AND (entry->>'incarnation')::uuid=head.incarnation
     AND (entry->>'revision')::bigint BETWEEN 1 AND head.revision) THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
 ELSE RAISE EXCEPTION 'company_information.material_unavailable';
 END CASE;
 RETURN QUERY SELECT to_jsonb(selected_company),to_jsonb(member),to_jsonb(history);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
