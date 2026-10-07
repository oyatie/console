-- UNINSTALLED initial manager Current-Grant material. No command is accepted.
-- The adapter must first verify only the signed namespace, then call this owner
-- before acquiring Account/family. Final rereads pass the retained Group ID;
-- NULL is permitted only on the first source call in a fresh retained scope.
CREATE FUNCTION public.identity_company_information_manager_current_v1(
 p_account uuid,p_family uuid,p_company uuid,p_command uuid,p_expected_group uuid)
RETURNS TABLE(actor_account_id uuid,session_id uuid,org_id uuid,command_id uuid,
 current_group_id uuid,company_epoch bigint,current_policy_receipt_id uuid,
 context_generation bigint,assignment_id uuid,assignment_revision bigint,
 role_id uuid,role_revision bigint,registered_clauses jsonb,company_name text,
 company_slug text,installed_object_type_id uuid,observed_at timestamptz,
 source_xid xid8,source_backend_pid integer,source_material jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid;
 actual_group uuid; planned_present boolean; actual_present boolean;
 control record; family record; retained_group record; presence record;
 selected record; native_root record; observed timestamptz; material jsonb; account_row jsonb;
 registration jsonb; consent jsonb;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR num_nonnulls(p_account,p_family,p_company,p_command)<>4
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_account,p_family,p_company,p_command,p_expected_group)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 -- The only initial planning read. No labels or source leave this scope.
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 planned_present:=FOUND;
 IF p_expected_group IS NOT NULL AND planned_group IS DISTINCT FROM p_expected_group THEN
  -- In particular, never discover/lock a second Group during finalization.
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 IF planned_group IS NOT NULL THEN
  SELECT * INTO STRICT retained_group
   FROM public.identity_company_information_group_lock_v1(p_company,planned_group);
 END IF;
 -- Actual Group head+row precede the original Account and family rows.
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation
  OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' OR family.auth_time IS NULL
  OR family.created_at IS NULL OR NOT isfinite(family.auth_time) OR NOT isfinite(family.created_at)
  OR family.auth_time>family.created_at OR family.created_at>clock_timestamp() THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 IF control.security_generation IS NULL OR control.security_generation<1
  OR control.revision IS NULL OR control.revision<1
  OR control.context_generation IS NULL OR control.context_generation NOT BETWEEN 1 AND 257 THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_agg(to_jsonb(r) ORDER BY r.terms_kind COLLATE "C") INTO registration
  FROM public.native_company_policy_registration_custody_v1(p_account) r;
 IF registration IS NULL OR jsonb_array_length(registration) NOT BETWEEN 1 AND 8 THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_agg(to_jsonb(c) ORDER BY c.terms_kind COLLATE "C") INTO consent
  FROM public.account_login_consent_v1(p_account) c;
 IF consent IS DISTINCT FROM registration THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
 SELECT * INTO STRICT presence FROM public.account_context_presence_v1(p_account);
 IF presence.context_generation IS DISTINCT FROM control.context_generation THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT o.group_id INTO actual_group FROM public.organizations o WHERE o.id=p_company;
 actual_present:=FOUND;
 IF actual_present IS DISTINCT FROM planned_present OR actual_group IS DISTINCT FROM planned_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_information.lock_plan_changed';
 END IF;
 IF planned_group IS NULL THEN
  -- A present row with no Group violates the finalized Company substrate.
  IF planned_present THEN RAISE EXCEPTION 'company_information.material_unavailable'; END IF;
  -- Authenticated healthy absent Company.
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 -- Only requested Company/membership/history. No whole-Group provenance helper.
 SELECT * INTO STRICT selected FROM public.identity_company_information_selected_lock_v1(p_company,planned_group);
 IF num_nonnulls((selected.company_row->>'origin_account_id')::uuid,
    (selected.company_row->>'origin_command_id')::uuid,(selected.company_row->>'origin_receipt_id')::uuid)=0 THEN
  -- This is finite selected nonnative source validation, not Group-wide health.
  IF EXISTS(SELECT 1 FROM public.company_enrollment_receipts r WHERE r.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.company_enrollment_effect_bindings e WHERE e.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company)
   OR EXISTS(SELECT 1 FROM public.native_company_catalog_installs c WHERE c.org_id=p_company) THEN
   RAISE EXCEPTION 'company_information.material_unavailable';
  END IF;
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 -- This validates actual birth/root/catalog without authenticating as the manager.
 -- Every native corruption check precedes every healthy inactive/nonmanager return.
 SELECT * INTO STRICT native_root FROM public.identity_company_information_root_material_v1(p_company,planned_group);
 IF (selected.membership_revision_row->>'native_account_id',selected.membership_revision_row->>'command_id',
     selected.membership_revision_row->>'command_receipt') IS DISTINCT FROM
    (native_root.root_material->'birth_receipt'->>'account_id',native_root.root_material->'birth_receipt'->>'command_id',
     native_root.root_material->'birth_receipt'->>'receipt_id')
  OR (selected.membership_revision_row->>'from_time')::timestamptz IS DISTINCT FROM
     (native_root.root_material->'birth_receipt'->>'committed_at')::timestamptz THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 observed:=clock_timestamp();
 IF (selected.membership_revision_row->>'from_time')::timestamptz>observed THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 IF retained_group.group_head_row->>'state' IS DISTINCT FROM 'ACTIVE'
  OR retained_group.group_row->>'status' IS DISTINCT FROM 'ACTIVE'
  OR selected.company_row->>'status' IS DISTINCT FROM 'ACTIVE'
  OR native_root.root_material->'role'->>'status' IS DISTINCT FROM 'ACTIVE'
  OR native_root.root_material->'assignment_revision'->>'state' IS DISTINCT FROM 'ACTIVE'
  OR native_root.root_material->'role_revision'->>'state' IS DISTINCT FROM 'ACTIVE'
  OR (native_root.root_material->'assignment_revision'->>'valid_from')::timestamptz>observed
  OR (native_root.root_material->'role_revision'->>'valid_from')::timestamptz>observed
  OR native_root.root_account_id IS DISTINCT FROM p_account THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF EXISTS(SELECT 1 FROM public.native_company_policy_inputs_v1 i
   WHERE i.org_id=p_company AND i.actor_account_id=p_account AND i.command_id=p_command)
  OR EXISTS(SELECT 1 FROM public.native_company_policy_receipts_v1 r
   WHERE r.org_id=p_company AND r.actor_account_id=p_account AND r.command_id=p_command) THEN
  RAISE EXCEPTION 'company_information.material_unavailable';
 END IF;
 SELECT jsonb_build_object('id',a.id,'created_at',a.created_at,'security',to_jsonb(s)) INTO STRICT account_row
  FROM public.accounts a JOIN public.account_security s ON s.account_id=a.id WHERE a.id=p_account;
 material:=native_root.root_material||jsonb_build_object('kind','COMPANY_INFORMATION_MANAGER_CURRENT_SOURCE_V1',
  'request',jsonb_build_object('codec_version',4,'operation','Grant','account_id',p_account,
    'session_id',p_family,'org_id',p_company,'command_id',p_command),
  'account',account_row,'family',jsonb_build_object('id',p_family,'material',to_jsonb(family)),
  'registration',registration,'consent',consent,'context_presence',to_jsonb(presence),
  'group',retained_group.group_row,'group_head',retained_group.group_head_row,'company',selected.company_row,
  'membership',selected.membership_row,'membership_revision',selected.membership_revision_row);
 observed:=clock_timestamp();
 RETURN QUERY SELECT p_account,p_family,p_company,p_command,planned_group,native_root.company_epoch,
  native_root.current_policy_receipt_id,control.context_generation,native_root.assignment_id,
  1::bigint,native_root.role_id,1::bigint,native_root.registered_clauses,
  selected.company_row->>'name',selected.company_row->>'slug',native_root.installed_object_type_id,
  observed,pg_current_xact_id(),pg_backend_pid(),material;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_information.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
