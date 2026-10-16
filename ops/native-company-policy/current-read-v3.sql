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
