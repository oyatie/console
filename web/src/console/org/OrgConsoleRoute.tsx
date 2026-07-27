import { useCallback } from "react";
import { useNavigate } from "react-router";

import { useAuth } from "../../context/auth";
import { consoleScreenPath } from "../shell/nav";
import { OrgChartScreen } from "./OrgChartScreen";
import { deriveOrgCapabilities } from "./orgCapabilities";
import { useOrgConsoleAuthz } from "./useOrgConsoleAuthz";

/**
 * Registry-ready body for the shared console registry ("orgchart" screen key).
 * Backend authorization remains the authority; the projection only shapes
 * what renders (deny-by-omission).
 */
export function OrgChartScreenBody() {
  const { api, session } = useAuth();
  const gate = useOrgConsoleAuthz();
  const capabilities = deriveOrgCapabilities(gate);
  const navigate = useNavigate();
  const onNavigate = useCallback((screen: string) => {
    void navigate(consoleScreenPath(screen));
  }, [navigate]);

  return (
    <OrgChartScreen
      api={api}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
      onNavigate={onNavigate}
    />
  );
}
