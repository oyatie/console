import { useActiveBranchId, useAuth } from "../../context/auth";
import { PayrollScreen } from "./PayrollScreen";
import { derivePayrollCapabilities } from "./payrollCapabilities";
import { usePayrollConsoleAuthz } from "./usePayrollConsoleAuthz";

/**
 * Module-owned route/body adapter. It consumes the console policy authz
 * projection, while shared registration remains intentionally outside this module.
 */
export function PayrollConsoleRoute({ branchId }: { branchId: string }) {
  return <PayrollConsoleBody branchId={branchId} />;
}

export function PayrollConsoleBody({ branchId }: { branchId: string }) {
  const { api, session } = useAuth();
  const authz = usePayrollConsoleAuthz();
  const capabilities = derivePayrollCapabilities(authz, branchId);

  return (
    <PayrollScreen
      api={api}
      branchId={branchId}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
    />
  );
}

/**
 * Zero-prop body for `SCREEN_REGISTRY` ("payroll"). Payroll grants are
 * org-wide-only server-side; the active JWT branch (or the empty sentinel when
 * the session carries none) only narrows branch-scoped grants, which the
 * backend rejects for payroll anyway — fail closed either way.
 */
export function PayrollScreenBody() {
  const branchId = useActiveBranchId();
  return <PayrollConsoleBody branchId={branchId ?? ""} />;
}
