permit (
    principal,
    action in [Action::"InstallPeopleDirectoryCatalogV1", Action::"GrantPeopleDirectoryV1", Action::"RevokePeopleDirectoryV1"],
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
    resource.manifest_digest == "591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e" &&
    context.manifest_digest == resource.manifest_digest
};
