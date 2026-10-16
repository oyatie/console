permit (
    principal,
    action in [Action::"InstallPayrollReadCatalogV1", Action::"GrantPayrollReadV1", Action::"RevokePayrollReadV1"],
    resource
) when {
    principal.state == "ACTIVE" &&
    principal.designation_state == "ACTIVE" &&
    principal.account_id == principal.designation_account_id &&
    principal.deployment_system_identifier == resource.deployment_system_identifier &&
    principal.deployment_database_name == resource.deployment_database_name &&
    principal.deployment_database_oid == resource.deployment_database_oid &&
    resource.state == "ACTIVE" &&
    resource.group_state == "ACTIVE" &&
    resource.membership_state == "ACTIVE" &&
    resource.org_id == context.requested_org_id &&
    resource.current_group_id == context.requested_group_id &&
    resource.administrative_account_id == context.recipient_account_id &&
    resource.company_actor_admission_receipt_id == resource.origin_receipt_id &&
    resource.manifest_digest == "07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd" &&
    context.manifest_digest == resource.manifest_digest
};
