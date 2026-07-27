export type FieldFeature =
  | "login"
  | "work_order_read_all"
  | "work_order_start"
  | "assignee_manage";

/** Canonical policy gate exposes typed feature, permission, and branch queries. */
export interface FieldPolicyGate {
  allows: (query: {
    feature: FieldFeature;
    branch?: string;
    minPermission: "allow";
  }) => boolean;
}

export interface FieldCapabilities {
  /** Read the site list/detail — `Login`-tier (the server confines rows). */
  canRead: boolean;
  /** File an internal issue ticket on the active branch (`Login` on branch). */
  canIntake: boolean;
  /** Assign / transition / link tickets (`AssigneeManage`). */
  canTriage: boolean;
  /** Record customer acceptance on a RESOLVED ticket (`AssigneeManage`). */
  canAccept: boolean;
  /** Comment on tickets (`WorkOrderStart`). */
  canComment: boolean;
}

/** Pure projection adapter matching the support/field backend feature gates. */
export function deriveFieldCapabilities(
  gate: FieldPolicyGate,
  branchId: string,
): FieldCapabilities {
  const allows = (feature: FieldFeature, branch?: string) =>
    gate.allows({ feature, ...(branch === undefined ? {} : { branch }), minPermission: "allow" });
  const login = allows("login");
  const triage = allows("assignee_manage");
  return {
    canRead: login || allows("work_order_read_all"),
    canIntake: branchId !== "" && allows("login", branchId),
    canTriage: triage,
    canAccept: triage,
    canComment: allows("work_order_start"),
  };
}
