/** Public, module-owned mount contract for the shared console registry. */
export interface PayrollRouteContract {
  branchId: string;
}

/** Fixture is structural only: it deliberately contains no business records. */
export const PAYROLL_ROUTE_CONTRACT_FIXTURE: PayrollRouteContract = {
  branchId: "00000000-0000-4000-8000-000000000000",
};
