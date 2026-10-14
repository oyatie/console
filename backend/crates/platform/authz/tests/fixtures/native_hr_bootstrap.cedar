// Proposed finite static bootstrap gate; absent metadata and actual root decisions are distinct.
permit(principal, action == Action::"hr.policy.read", resource) when {
    principal.state == "ACTIVE" &&
    principal.account_id == resource.original_administrator_b &&
    resource.owner_account_id == resource.original_administrator_b &&
    resource.company_state == "ACTIVE" && resource.group_state == "ACTIVE" && resource.membership_state == "ACTIVE" &&
    resource.company_id == context.requested_company_id &&
    resource.resource_kind == context.resource_kind && resource.command_id == context.command_id &&
    resource.root_read_policy_allowed && resource.root_assign_identity_allowed &&
    resource.root_assignment_revision == "1" && resource.root_role_revision == "1" &&
    resource.assign_identity_registration_revision == "1" && resource.read_policy_registration_revision == "1" &&
    resource.assign_identity_manifest_digest == "0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935" &&
    resource.read_policy_manifest_digest == resource.assign_identity_manifest_digest &&
    resource.company_actor_origin_kind == "COMPANY_ENROLLMENT_V1" &&
    resource.company_actor_admission_receipt_id == resource.origin_receipt_id &&
    resource.company_actor_origin_account_id == resource.origin_account_id &&
    resource.company_actor_origin_command_id == resource.origin_command_id &&
    resource.company_actor_origin_receipt_id == resource.origin_receipt_id &&
    resource.hr_catalog_version == "native-company-hr-appointment-v1" &&
    resource.hr_manifest_digest == "b5c2e0366a162222489477d3b231463ce8dc7686cad82f39219fe0bf223f2803" &&
    ((resource.resource_kind == "POLICY" && resource.command_id == "" && resource.ownership_kind == "NONE") ||
     (resource.resource_kind == "OWN_SETUP" && resource.command_id != "" &&
      ["LOCATOR", "ACCEPTED", "TERMINAL", "FENCE"].contains(resource.ownership_kind))) &&
    ((resource.stage == "ABSENT" && context.intent == "READ") ||
     (resource.stage == "OWN_TENTATIVE_SETUP" && context.intent == "FINALIZE_SETUP" &&
      resource.resource_kind == "OWN_SETUP" && resource.ownership_kind == "ACCEPTED" && resource.own_tentative_verified)) &&
    ["hr.policy.company", "hr.policy.recipient", "hr.policy.role", "hr.policy.actions", "hr.policy.fields", "hr.policy.validity", "hr.policy.revision", "hr.policy.receipt"].containsAll(context.requested_fields) && context.requested_fields.containsAll(["hr.policy.company", "hr.policy.recipient", "hr.policy.role", "hr.policy.actions", "hr.policy.fields", "hr.policy.validity", "hr.policy.revision", "hr.policy.receipt"])
};

permit(principal, action == Action::"hr.policy.setup", resource) when {
    principal.state == "ACTIVE" &&
    principal.account_id == resource.original_administrator_b &&
    resource.owner_account_id == resource.original_administrator_b &&
    resource.company_state == "ACTIVE" && resource.group_state == "ACTIVE" && resource.membership_state == "ACTIVE" &&
    resource.company_id == context.requested_company_id &&
    resource.resource_kind == context.resource_kind && resource.command_id == context.command_id &&
    resource.root_read_policy_allowed && resource.root_assign_identity_allowed &&
    resource.root_assignment_revision == "1" && resource.root_role_revision == "1" &&
    resource.assign_identity_registration_revision == "1" && resource.read_policy_registration_revision == "1" &&
    resource.assign_identity_manifest_digest == "0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935" &&
    resource.read_policy_manifest_digest == resource.assign_identity_manifest_digest &&
    resource.company_actor_origin_kind == "COMPANY_ENROLLMENT_V1" &&
    resource.company_actor_admission_receipt_id == resource.origin_receipt_id &&
    resource.company_actor_origin_account_id == resource.origin_account_id &&
    resource.company_actor_origin_command_id == resource.origin_command_id &&
    resource.company_actor_origin_receipt_id == resource.origin_receipt_id &&
    resource.hr_catalog_version == "native-company-hr-appointment-v1" &&
    resource.hr_manifest_digest == "b5c2e0366a162222489477d3b231463ce8dc7686cad82f39219fe0bf223f2803" &&
    ((resource.resource_kind == "POLICY" && resource.command_id == "" && resource.ownership_kind == "NONE") ||
     (resource.resource_kind == "OWN_SETUP" && resource.command_id != "" &&
      ["LOCATOR", "ACCEPTED", "TERMINAL", "FENCE"].contains(resource.ownership_kind))) &&
    ((resource.stage == "ABSENT" && context.intent == "ADMIT_SETUP" && resource.namespace_available &&
      (resource.ownership_kind == "NONE" || resource.ownership_kind == "LOCATOR" || resource.ownership_kind == "ACCEPTED")) ||
     (resource.stage == "OWN_TENTATIVE_SETUP" && context.intent == "FINALIZE_SETUP" &&
      resource.resource_kind == "OWN_SETUP" && resource.ownership_kind == "ACCEPTED" && resource.own_tentative_verified) ||
     (context.intent == "REFUSE_ACCEPTED_SETUP" && resource.resource_kind == "OWN_SETUP" && resource.ownership_kind == "ACCEPTED" &&
      !resource.own_tentative_verified && !resource.namespace_available &&
      (resource.stage == "ABSENT" || resource.stage == "OTHER_VALID_SETUP"))) &&
    context.requested_fields.isEmpty()
};
