export type MaintenanceFeature =
  | "work_order_read_all"
  | "work_order_create"
  | "work_order_edit_intake"
  | "work_order_start"
  | "work_report_submit"
  | "completion_review"
  | "priority_manage"
  | "assignee_manage"
  | "target_manage"
  | "evidence_attach"
  | "settlement_submit"
  | "settlement_review"
  | "org_wide_queue_triage";

/** Canonical policy gate exposes typed feature, permission, and branch queries. */
export interface MaintenancePolicyGate {
  allows: (query: {
    feature: MaintenanceFeature;
    branch: string;
    minPermission: "allow";
  }) => boolean;
}

export interface MaintenanceCapabilities {
  canRead: boolean;
  canCreate: boolean;
  canEditIntake: boolean;
  canAssign: boolean;
  canStart: boolean;
  canSubmitReport: boolean;
  canReview: boolean;
  canManagePriority: boolean;
  canManageTarget: boolean;
  canAttachEvidence: boolean;
  canSettle: boolean;
  canReviewSettlement: boolean;
  canTriage: boolean;
}

/** Pure projection adapter matching the work-order backend feature gates. */
export function deriveMaintenanceCapabilities(
  gate: MaintenancePolicyGate,
  branchId: string,
): MaintenanceCapabilities {
  const allows = (feature: MaintenanceFeature) =>
    gate.allows({ feature, branch: branchId, minPermission: "allow" });
  return {
    canRead: allows("work_order_read_all"),
    canCreate: allows("work_order_create"),
    canEditIntake: allows("work_order_edit_intake"),
    canAssign: allows("assignee_manage"),
    canStart: allows("work_order_start"),
    canSubmitReport: allows("work_report_submit"),
    canReview: allows("completion_review"),
    canManagePriority: allows("priority_manage"),
    canManageTarget: allows("target_manage"),
    canAttachEvidence: allows("evidence_attach"),
    canSettle: allows("settlement_submit"),
    canReviewSettlement: allows("settlement_review"),
    canTriage: allows("org_wide_queue_triage"),
  };
}
