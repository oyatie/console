import { useActiveBranchId, useAuth } from "../../context/auth";
import { FieldScreen } from "./FieldScreen";
import { deriveFieldCapabilities } from "./fieldCapabilities";
import { useFieldConsoleAuthz } from "./useFieldConsoleAuthz";

/**
 * Module-owned route/body adapter. It consumes the console policy authz
 * projection, while shared registration remains intentionally outside this module.
 */
export function FieldConsoleRoute({ branchId }: { branchId: string }) {
  return <FieldConsoleBody branchId={branchId} />;
}

export function FieldConsoleBody({ branchId }: { branchId: string }) {
  const { api, session } = useAuth();
  const authz = useFieldConsoleAuthz();
  const capabilities = deriveFieldCapabilities(authz, branchId);

  return (
    <FieldScreen
      api={api}
      branchId={branchId}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
    />
  );
}

/**
 * Zero-prop body for the shared screen registry (`SCREEN_REGISTRY.field`).
 * Resolves the active branch itself; with no branch membership the capability
 * projection fails closed and the screen renders its denied state.
 */
export function FieldScreenBody() {
  const branchId = useActiveBranchId();
  return <FieldConsoleBody branchId={branchId ?? ""} />;
}
