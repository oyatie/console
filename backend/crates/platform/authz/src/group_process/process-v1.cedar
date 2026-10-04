// Proposed SDK source only. Strict validation/evaluation has NOT executed.
// All attributes come from private purpose-specific protected material.
// bootstrap_stage is a closed source reconstruction, never a posted boolean.
// V3 adds empty landing and own-receipt proof form; no new business action/field.
// Own-receipt assembly omits current designation/source/process prerequisites;
// current-only schema fields are optional and omitted; no fabricated sentinel.
permit(principal, action in [Action::"identity.verifier.process.adopt/1", Action::"identity.verifier.process.suspend/1", Action::"identity.verifier.process.read/1"], resource)
when {
 principal.state == "ACTIVE" &&
 principal has designation_state && principal has designation_account_id &&
 principal has system_identifier && principal has database_name && principal has database_oid &&
 resource has group_state && resource has system_identifier && resource has database_name && resource has database_oid &&
 resource has policy_state && resource has bootstrap_stage &&
 principal.designation_state == "ACTIVE" &&
 principal.account_id == principal.designation_account_id &&
 principal.system_identifier == resource.system_identifier &&
 principal.database_name == resource.database_name && principal.database_oid == resource.database_oid &&
 resource.group_state == "ACTIVE" && resource.group_id == context.group_id &&
 resource.incarnation == context.incarnation && resource.resource_kind == context.resource_kind &&
 resource.manifest_digest == context.manifest_digest &&
 resource.policy_state == context.policy_state && resource.bootstrap_stage == context.bootstrap_stage &&
 resource.allowed_fields.containsAll(context.requested_fields) &&
 (
  (resource.policy_state == "INSTALLED" && resource.bootstrap_stage == "NONE" &&
   ((action in [Action::"identity.verifier.process.adopt/1", Action::"identity.verifier.process.suspend/1"] && resource.resource_kind == "PROCESS_MUTATION") ||
    (action == Action::"identity.verifier.process.read/1" && ["CURRENT_PROCESS", "PROCESS_FORM"].contains(resource.resource_kind)))) ||
  (resource.policy_state == "ABSENT" && action == Action::"identity.verifier.process.adopt/1" &&
   resource.resource_kind == "PROCESS_MUTATION" && ["FIRST_INTAKE", "FIRST_EXECUTION"].contains(resource.bootstrap_stage)) ||
  (resource.policy_state == "ABSENT" && action == Action::"identity.verifier.process.read/1" &&
   ((resource.resource_kind == "PROCESS_FORM" && resource.bootstrap_stage == "EMPTY_FORM") ||
    (resource.resource_kind == "PROCESS_LANDING" && resource.bootstrap_stage == "EMPTY_LANDING")))
 )
};
permit(principal, action == Action::"identity.verifier.process.receipt.read-own/1", resource)
when {
 principal.state == "ACTIVE" && resource has receipt_actor && principal.account_id == resource.receipt_actor &&
 resource.group_id == context.group_id && resource.incarnation == context.incarnation &&
 ["OWN_RECEIPT", "OWN_RECEIPT_RETRY_FORM"].contains(resource.resource_kind) &&
 resource.resource_kind == context.resource_kind &&
 resource.manifest_digest == context.manifest_digest && resource.allowed_fields.containsAll(context.requested_fields)
};
