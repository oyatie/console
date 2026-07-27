/**
 * Public, module-owned mount contract for the shared console registry.
 * Recruiting is org-scoped (RLS `app.current_org`): the screen needs no route
 * parameters — the authenticated session is the whole contract.
 */
export type RecruitingRouteContract = Record<string, never>;

/** Fixture is structural only: it deliberately contains no business records. */
export const RECRUITING_ROUTE_CONTRACT_FIXTURE: RecruitingRouteContract = {};
