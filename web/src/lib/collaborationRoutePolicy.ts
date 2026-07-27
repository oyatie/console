/**
 * UI presentation policy for the collaboration route.
 *
 * This does not authorize data access: the route guard and backend remain
 * authoritative.  It gives independent UI surfaces one stable answer for
 * whether advertising the collaboration destination is truthful.  Group roles
 * intentionally do not grant this route.
 */
export const COLLABORATION_ROUTE_ROLES = [
  "SUPER_ADMIN",
  "ADMIN",
  "EXECUTIVE",
  "MECHANIC",
  "RECEPTIONIST",
] as const;

export const COLLABORATION_ROUTE_FEATURE = "work_order_read_all" as const;

export function canPresentCollaborationRoute(
  roles: readonly string[] | undefined,
  featureGrants: readonly string[] | undefined,
): boolean {
  return (
    roles?.some((role) =>
      (COLLABORATION_ROUTE_ROLES as readonly string[]).includes(role),
    ) === true || featureGrants?.includes(COLLABORATION_ROUTE_FEATURE) === true
  );
}
