export type EvaluationFeature =
  | "evaluation_read"
  | "evaluation_manage"
  | "evaluation_submit";

/** Canonical policy gate surface consumed by this module (see ../policy/authz). */
export interface EvaluationPolicyGate {
  allows: (query: {
    feature: EvaluationFeature;
    branch?: string;
    minPermission: "allow";
  }) => boolean;
}

export interface EvaluationCapabilities {
  canRead: boolean;
  canManage: boolean;
  canSubmit: boolean;
  canCalibrate: boolean;
}

/**
 * Pure projection adapter over the evaluation feature gates. Evaluation is an
 * org-level module; a branch id, when present, still intersects against the
 * capability's own branch scope (fail closed for branch-narrowed grants).
 */
export function deriveEvaluationCapabilities(
  gate: EvaluationPolicyGate,
  branchId: string | undefined,
): EvaluationCapabilities {
  const allows = (feature: EvaluationFeature) =>
    gate.allows({
      feature,
      ...(branchId === undefined ? {} : { branch: branchId }),
      minPermission: "allow",
    });
  const canManage = allows("evaluation_manage");
  return {
    canRead: allows("evaluation_read"),
    canManage,
    canSubmit: allows("evaluation_submit"),
    canCalibrate: canManage,
  };
}
