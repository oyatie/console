// Proposed finite policy; no legacy role-matrix generation and no wildcard.
// Session/family correlation and typed operation-target admission precede Cedar.
// PRIVATE eligibility only: historical_unbound effects/mints can receive this
// permit but the composite wrapper MUST return UpgradeRequired, never Allow.
permit (
  principal is ConsoleLegacyPlatformV1::Subject,
  action in [
    ConsoleLegacyPlatformV1::Action::"tenant_create",
    ConsoleLegacyPlatformV1::Action::"tenant_list",
    ConsoleLegacyPlatformV1::Action::"tenant_suspend",
    ConsoleLegacyPlatformV1::Action::"tenant_remove",
    ConsoleLegacyPlatformV1::Action::"tenant_health_read",
    ConsoleLegacyPlatformV1::Action::"tenant_manage",
    ConsoleLegacyPlatformV1::Action::"group_manage",
    ConsoleLegacyPlatformV1::Action::"platform_audit_read"
  ],
  resource is ConsoleLegacyPlatformV1::Resource
)
when {
  context.policy_version == "legacy-platform-authority-v1" &&
  principal.home_org == "00000000-0000-0000-0000-00000000face" &&
  principal.active && !principal.account_fenced &&
  principal.roles.contains("SUPER_ADMIN") &&
  resource.platform_scope == "legacy_platform" &&
  ["read", "effect", "mint"].contains(context.use_kind) &&
  (
    context.session_mode == "bound" ||
    context.session_mode == "historical_unbound"
  ) &&
  (
    (context.source_kind == "direct" && context.call_site == "platform_route" && context.source_company == "") ||
    (context.source_kind == "platform_view_as" && context.call_site == "derived_source" &&
      resource.kind == "company" && resource.key == context.source_company &&
      action == ConsoleLegacyPlatformV1::Action::"tenant_health_read" && context.use_kind == "read") ||
    (context.source_kind == "platform_tenant_context" && context.call_site == "derived_source" &&
      resource.kind == "company" && resource.key == context.source_company &&
      action == ConsoleLegacyPlatformV1::Action::"tenant_manage" &&
      ["read", "effect"].contains(context.use_kind))
  ) &&
  (
    (action == ConsoleLegacyPlatformV1::Action::"tenant_create" && resource.kind == "companies" && context.use_kind == "effect") ||
    (action == ConsoleLegacyPlatformV1::Action::"tenant_list" && resource.kind == "companies" && context.use_kind == "read") ||
    (action == ConsoleLegacyPlatformV1::Action::"tenant_suspend" && resource.kind == "company" && context.use_kind == "effect") ||
    (action == ConsoleLegacyPlatformV1::Action::"tenant_remove" && resource.kind == "company" && context.use_kind == "effect") ||
    (action == ConsoleLegacyPlatformV1::Action::"tenant_health_read" &&
      ((resource.kind == "company" && ["read", "mint"].contains(context.use_kind)) ||
       (resource.kind == "operations" && context.source_kind == "direct" && ["read", "effect"].contains(context.use_kind)))) ||
    (action == ConsoleLegacyPlatformV1::Action::"tenant_manage" &&
      ((resource.kind == "company" &&
        ((context.source_kind == "direct" && context.use_kind == "mint") ||
         (context.source_kind == "platform_tenant_context" && ["read", "effect"].contains(context.use_kind)))) ||
       (resource.kind == "operations" && context.source_kind == "direct" && context.use_kind == "effect"))) ||
    (action == ConsoleLegacyPlatformV1::Action::"group_manage" && ["groups", "group", "company"].contains(resource.kind) && ["read", "effect"].contains(context.use_kind)) ||
    (action == ConsoleLegacyPlatformV1::Action::"platform_audit_read" && resource.kind == "platform_audit" && context.use_kind == "read")
  )
};
