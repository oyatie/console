permit (principal, action, resource) when {
    principal.state == "ACTIVE" &&
    principal.account_id == context.requested_actor_id &&
    principal.account_id == resource.recipient_account_id &&
    resource.org_id == context.requested_org_id &&
    resource.object_type_id == context.object_type_id &&
    resource.registered_action_id == context.registered_action_id &&
    resource.registration_revision == "1" &&
    resource.manifest_digest == "591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e" &&
    resource.assignment_state == "ACTIVE" && resource.role_state == "ACTIVE" &&
    resource.role_revision == "1" && resource.delegable == false &&
    resource.valid_from_us <= context.observed_at_us && context.observed_at_us < resource.valid_until_us &&
    resource.allowed_fields == context.requested_fields &&
    ((action == Action::"people.directory.read" && resource.granted_action == "people.directory.read" &&
      ((context.resource_kind == "navigation" && context.resource_id == "navigation") ||
       (context.resource_kind == "collection" && context.resource_id == "collection") ||
       (context.resource_kind == "entry" && context.resource_id != ""))) ||
     (action == Action::"people.directory.create" && resource.granted_action == "people.directory.create" &&
      ((context.resource_kind == "navigation" && context.resource_id == "navigation") ||
       (context.resource_kind == "request" && context.resource_id != ""))))
};
