/** Public, module-owned mount contract for the shared console registry. */
export interface MaintenanceRouteContract {
  branchId: string;
}

/** Fixture is structural only: it deliberately contains no business records. */
export const MAINTENANCE_ROUTE_CONTRACT_FIXTURE: MaintenanceRouteContract = {
  branchId: "00000000-0000-4000-8000-000000000000",
};
