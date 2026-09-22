permit (principal, action == Action::"payroll.collection.read", resource) when {
    principal.state == "ACTIVE" &&
    principal.account_id == resource.recipient_account_id &&
    resource.org_id == context.requested_org_id &&
    resource.object_type_id == context.object_type_id &&
    resource.registered_action_id == context.registered_action_id &&
    resource.registration_revision == "1" &&
    resource.manifest_digest == "07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd" &&
    resource.assignment_state == "ACTIVE" &&
    resource.role_state == "ACTIVE" &&
    resource.role_revision == "1" &&
    resource.delegable == false &&
    resource.valid_from_us <= context.observed_at_us &&
    context.observed_at_us < resource.valid_until_us &&
    resource.allowed_fields == context.requested_fields
};
