-- New branches are gated by same-Company receipt FKs, live frame guards and complete deferred closure.

ALTER TABLE public.native_company_catalog_installs
 DROP CONSTRAINT native_company_catalog_installs_catalog_version_check,
 DROP CONSTRAINT native_company_catalog_installs_manifest_digest_check1,
 ADD CONSTRAINT native_company_catalog_installs_catalog_policy_shape CHECK(
  (policy_receipt_id IS NULL AND catalog_version='native-company-identity-2026-09-19.1'
   AND manifest_digest=decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex'))
  OR (policy_receipt_id IS NOT NULL AND catalog_version='native-payroll-collection-read-v1'
   AND manifest_digest=decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex')));
ALTER TABLE public.native_company_object_refs DROP CONSTRAINT native_company_object_refs_object_key_check,
 ADD CONSTRAINT native_company_object_refs_object_key_v2 CHECK(
  (catalog_version='native-company-identity-2026-09-19.1' AND object_key IN ('company_workspace','company_policy_assignment'))
  OR (catalog_version='native-payroll-collection-read-v1' AND object_key='pay_run'));
ALTER TABLE public.native_company_action_refs DROP CONSTRAINT native_company_action_refs_action_key_check,
 ADD CONSTRAINT native_company_action_refs_action_key_v2 CHECK(
  (catalog_version='native-company-identity-2026-09-19.1' AND action_key IN ('context.discover','company.identity.read','company.policy.read','company.policy.assign','company.policy.revoke'))
  OR (catalog_version='native-payroll-collection-read-v1' AND action_key='payroll.collection.read'));
ALTER TABLE public.native_company_property_refs DROP CONSTRAINT native_company_property_refs_property_key_check,
 ADD CONSTRAINT native_company_property_refs_property_key_v2 CHECK(
  (catalog_version='native-company-identity-2026-09-19.1' AND property_key IN ('company.name','company.slug','assignment.account_id','assignment.scope','assignment.actions','assignment.fields','assignment.valid_from','assignment.valid_until','assignment.state','assignment.revision'))
  OR (catalog_version='native-payroll-collection-read-v1' AND property_key IN ('pay_run.id','pay_run.period_start','pay_run.period_end','pay_run.source_label','pay_run.status','pay_run.calculation_enabled','pay_run.created_by','pay_run.approved_by','pay_run.approved_at','pay_run.close_receipt','pay_run.submitted_by','pay_run.submitted_at','pay_run.decided_by','pay_run.decided_at','pay_run.decision_reason','pay_run.approval_ref','pay_run.created_at','pay_run.updated_at')));
ALTER TABLE public.policy_role_revisions DROP CONSTRAINT policy_role_revisions_check,
 ADD CONSTRAINT policy_role_revisions_actor_policy_v2 CHECK(policy_receipt_id IS NOT NULL OR actor_account_id=origin_account_id);
ALTER TABLE public.policy_assignment_revisions DROP CONSTRAINT policy_assignment_revisions_check,
 ADD CONSTRAINT policy_assignment_revisions_actor_policy_v2 CHECK(policy_receipt_id IS NOT NULL OR actor_account_id=origin_account_id);
