/**
 * Directory capability projection. Mirrors the backend enforcement exactly:
 * the branch-member roster and person card ride the messenger tier (every
 * authenticated role, active branch required), the org-wide HR register is the
 * `employee_directory_read` feature (role floor ADMIN/EXECUTIVE/SUPER_ADMIN,
 * matching `shell/nav.ts` gate semantics: role OR feature grant). The backend
 * re-authorizes every call; this only shapes which controls exist.
 */

const MESSENGER_ROLES: ReadonlySet<string> = new Set([
  "SUPER_ADMIN",
  "ADMIN",
  "EXECUTIVE",
  "MECHANIC",
  "RECEPTIONIST",
  "MEMBER",
]);

const HR_DIRECTORY_ROLES: ReadonlySet<string> = new Set([
  "SUPER_ADMIN",
  "ADMIN",
  "EXECUTIVE",
]);

const EMPLOYEE_DIRECTORY_READ = "employee_directory_read";

export interface DirectoryGrants {
  roles: readonly string[];
  featureGrants: readonly string[];
}

export interface DirectoryCapabilities {
  canRead: boolean;
  canViewPerson: boolean;
  canReadHrDirectory: boolean;
  canMessage: boolean;
}

export function deriveDirectoryCapabilities(
  grants: DirectoryGrants,
  branchId: string | undefined,
): DirectoryCapabilities {
  const memberTier = Boolean(branchId) && grants.roles.some((role) => MESSENGER_ROLES.has(role));
  const hrTier =
    grants.featureGrants.includes(EMPLOYEE_DIRECTORY_READ) ||
    grants.roles.some((role) => HR_DIRECTORY_ROLES.has(role));
  return {
    canRead: memberTier || hrTier,
    canViewPerson: memberTier,
    canReadHrDirectory: hrTier,
    canMessage: memberTier,
  };
}
