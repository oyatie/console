export type OrgFeature =
  | "employee_directory_read"
  | "org_change_read"
  | "org_change_draft"
  | "org_change_approve"
  | "org_change_apply";

/** Canonical policy gate — org-change features are org-level, no branch target. */
export interface OrgPolicyGate {
  allows: (query: { feature: OrgFeature; minPermission: "allow" }) => boolean;
}

export interface OrgCapabilities {
  /** Org tree read (hr org-chart + identity structure). */
  canReadTree: boolean;
  /** Org-change governance list/detail. */
  canReadChanges: boolean;
  canDraft: boolean;
  canApprove: boolean;
  canApply: boolean;
}

/** Pure projection adapter matching the orgchange backend feature gates. */
export function deriveOrgCapabilities(gate: OrgPolicyGate): OrgCapabilities {
  const allows = (feature: OrgFeature) => gate.allows({ feature, minPermission: "allow" });
  return {
    canReadTree: allows("employee_directory_read"),
    canReadChanges: allows("org_change_read"),
    canDraft: allows("org_change_draft"),
    canApprove: allows("org_change_approve"),
    canApply: allows("org_change_apply"),
  };
}
