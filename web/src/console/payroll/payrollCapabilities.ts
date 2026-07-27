export type PayrollFeature = "payroll_run_read" | "payroll_run_manage";

/** Canonical policy gate exposes typed feature, permission, and branch queries. */
export interface PayrollPolicyGate {
  allows: (query: {
    feature: PayrollFeature;
    branch: string;
    minPermission: "allow";
  }) => boolean;
}

export interface PayrollCapabilities {
  canRead: boolean;
  /** close / calculate / resolve / submit / withdraw / schedule / attest / issue. */
  canManage: boolean;
  /** Equals canManage; the submitter≠decider SoD rule is server-enforced. */
  canDecide: boolean;
  /** Own payslips are a self-view right; the inbox surface owns their rendering. */
  canReadSelf: boolean;
}

/** Pure projection adapter matching the payroll backend feature gates. */
export function derivePayrollCapabilities(
  gate: PayrollPolicyGate,
  branchId: string,
): PayrollCapabilities {
  const allows = (feature: PayrollFeature) =>
    gate.allows({ feature, branch: branchId, minPermission: "allow" });
  const canManage = allows("payroll_run_manage");
  return {
    canRead: allows("payroll_run_read") || canManage,
    canManage,
    canDecide: canManage,
    canReadSelf: true,
  };
}
