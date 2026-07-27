/** Public, module-owned mount contract for the shared console registry. */
export interface OrgRouteContract {
  /** URL screen key the shell resolves to this module's body. */
  screen: "orgchart";
}

/** Fixture is structural only: it deliberately contains no business records. */
export const ORG_ROUTE_CONTRACT_FIXTURE: OrgRouteContract = {
  screen: "orgchart",
};
