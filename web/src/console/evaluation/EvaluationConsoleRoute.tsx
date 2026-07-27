import { useActiveBranchId, useAuth } from "../../context/auth";
import { EvaluationScreen } from "./EvaluationScreen";
import { deriveEvaluationCapabilities } from "./evaluationCapabilities";
import { useEvaluationConsoleAuthz } from "./useEvaluationConsoleAuthz";

/**
 * Module-owned route/body adapter. It consumes the console policy authz
 * projection, while shared registration remains intentionally outside this
 * module (`../screens/registry.ts` mounts {@link EvaluationScreenBody}).
 */
export function EvaluationConsoleRoute() {
  return <EvaluationScreenBody />;
}

export function EvaluationScreenBody() {
  const { api, session } = useAuth();
  const branchId = useActiveBranchId();
  const authz = useEvaluationConsoleAuthz();
  const capabilities = deriveEvaluationCapabilities(authz, branchId);

  return (
    <EvaluationScreen
      api={api}
      branchId={branchId}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
    />
  );
}
