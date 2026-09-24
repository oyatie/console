-- Proposal: new People business source; no replacement of historical Payroll SQL.
-- This source retains the inherited Company birth-administrator admission ceiling.
-- A catalog install is not a grant. Caller must authenticate retained credentials,
-- evaluate actual Cedar, and recheck this source before disclosing or committing.
CREATE FUNCTION public.identity_company_people_projection_v1(
 p_account uuid,p_family uuid,p_company uuid,p_action text)
RETURNS TABLE(company_epoch bigint,current_policy_receipt_id uuid,
 assignment_id uuid,assignment_revision bigint,role_id uuid,role_revision bigint,
 registered_clauses jsonb,assignment_valid_from timestamptz,assignment_valid_until timestamptz,
 action_reference jsonb,named_properties jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); identity_row record;
 role_row public.policy_roles; a public.user_role_assignments; ar public.policy_assignment_revisions;
 r public.native_company_policy_receipts_v1; clauses jsonb; at_time timestamptz;
 selected_role text; action_ref public.native_company_action_refs; fields jsonb;
BEGIN
 IF p_action IS NULL OR p_action NOT IN ('people.directory.read','people.directory.create') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_action';
 END IF;
 selected_role:=CASE p_action WHEN 'people.directory.read' THEN 'native_people_directory_read'
  ELSE 'native_people_directory_create' END;
 -- Existing source owns Group-first topology, Account/family, Company and
 -- registered catalog locks; it does not itself authorize the business action.
 SELECT * INTO identity_row FROM public.identity_company_projection_v2(p_account,p_family,p_company);
 IF NOT FOUND THEN RETURN; END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT x.* INTO role_row FROM public.policy_roles x
  WHERE x.org_id=p_company AND x.role_key=selected_role FOR SHARE;
 IF NOT FOUND THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 -- Native-subject unique index binds (Company, Account, role); no grant is
 -- an ordinary denial, distinct from a missing revision of an existing grant.
 SELECT x.* INTO a FROM public.user_role_assignments x WHERE x.org_id=p_company
  AND x.role_id=role_row.id AND x.account_id=p_account AND x.subject_protocol='NATIVE_ACCOUNT' FOR SHARE;
 IF NOT FOUND THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x
  WHERE x.org_id=p_company AND x.assignment_id=a.id AND x.revision=a.native_current_revision;
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x
  WHERE x.org_id=p_company AND x.receipt_id=ar.policy_receipt_id;
 IF ar.valid_from IS NULL OR ar.valid_until IS NULL
  OR NOT isfinite(ar.valid_from) OR NOT isfinite(ar.valid_until)
  OR ar.valid_from>=ar.valid_until OR ar.valid_until-ar.valid_from>interval '30 days'
  OR r.codec_version<>2 OR r.catalog_version<>'native-people-directory-v1'
  OR r.outcome<>'COMMITTED' OR r.operation NOT IN (2,3)
  OR r.assignment_id IS DISTINCT FROM a.id OR r.assignment_revision_after IS DISTINCT FROM ar.revision
  OR a.native_current_revision<1 OR role_row.subject_protocol<>'NATIVE_ACCOUNT'
  OR ar.subject_protocol<>'NATIVE_ACCOUNT' OR ar.account_id IS DISTINCT FROM p_account
  OR ar.role_id IS DISTINCT FROM role_row.id OR ar.role_revision<>1
  OR (r.recipient_account_id,r.role_id,r.role_revision,r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until)
   IS DISTINCT FROM (p_account,role_row.id,1::bigint,ar.state,ar.valid_from,ar.valid_until) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM public.native_company_policy_assert_effects_v1(p_company,r.receipt_id);
 clauses:=public.native_company_policy_business_clauses_v1(p_company,role_row.id);
 SELECT x.* INTO STRICT action_ref FROM public.native_company_action_refs x
  WHERE x.org_id=p_company AND x.catalog_version='native-people-directory-v1' AND x.action_key=p_action;
 SELECT jsonb_agg(jsonb_build_object('key',p.property_key,'org_id',p.org_id::text,
   'object_type_id',p.object_type_id::text,'property_id',p.property_id::text,
   'schema_revision',p.schema_revision::text) ORDER BY p.property_key COLLATE "C") INTO fields
 FROM public.native_company_property_refs p
 WHERE p.org_id=p_company AND p.catalog_version='native-people-directory-v1'
  AND p.object_type_id=action_ref.object_type_id
  AND (p_action='people.directory.read' OR p.property_key IN ('person.legal_name','person.employee_number'));
 IF jsonb_array_length(fields) IS DISTINCT FROM
    (CASE p_action WHEN 'people.directory.read' THEN 6 ELSE 2 END)
  OR action_ref.registration_revision<>1
  OR action_ref.manifest_digest IS DISTINCT FROM
     decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex') THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 at_time:=clock_timestamp();
 IF ar.state<>'ACTIVE' OR role_row.status<>'ACTIVE'
  OR ar.valid_from>at_time OR ar.valid_until<=at_time THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 RETURN QUERY SELECT identity_row.company_epoch,identity_row.current_policy_receipt_id,
  a.id,a.native_current_revision,role_row.id,1::bigint,clauses,ar.valid_from,ar.valid_until,
  jsonb_build_object('org_id',action_ref.org_id::text,'object_type_id',action_ref.object_type_id::text,
   'action_type_id',action_ref.action_type_id::text,'registration_revision',action_ref.registration_revision::text,
   'manifest_digest',encode(action_ref.manifest_digest,'hex')),fields;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- Successor custody installation (same atomic installer transaction).
ALTER FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) FROM PUBLIC;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_catalog.pg_proc p
  CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a
  JOIN pg_catalog.pg_roles r ON r.oid=a.grantee
  WHERE p.oid='public.identity_company_people_projection_v1(uuid,uuid,uuid,text)'::regprocedure
   AND r.rolname<>'console_account_owner'
 LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) TO console_rt;
