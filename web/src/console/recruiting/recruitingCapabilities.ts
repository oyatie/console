/**
 * Recruiting capabilities are derived from backend feature grants (never from
 * role names): `recruiting_read`, `recruiting_manage`, and — for the hire
 * handshake that creates an employee — `employee_directory_manage` as well.
 * The backend re-authorizes every call; this only shapes what renders
 * (deny-by-omission).
 */
export const RECRUITING_READ = "recruiting_read";
export const RECRUITING_MANAGE = "recruiting_manage";
export const EMPLOYEE_DIRECTORY_MANAGE = "employee_directory_manage";

export interface RecruitingPolicyGate {
  allows: (query: { feature: string; minPermission: "allow" }) => boolean;
}

export interface RecruitingCapabilities {
  canRead: boolean;
  /** Composer, publish/close, applicant pipeline, offers. */
  canManage: boolean;
  /** Hire handshake — also creates the employee object. */
  canHire: boolean;
}

export function deriveRecruitingCapabilities(gate: RecruitingPolicyGate): RecruitingCapabilities {
  const allows = (feature: string) => gate.allows({ feature, minPermission: "allow" });
  const canManage = allows(RECRUITING_MANAGE);
  return {
    canRead: canManage || allows(RECRUITING_READ),
    canManage,
    canHire: canManage && allows(EMPLOYEE_DIRECTORY_MANAGE),
  };
}
