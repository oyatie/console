import { useCallback, useMemo } from "react";
import { useNavigate } from "react-router";

import { useAuth } from "../../context/auth";
import { RecruitingScreen } from "./RecruitingScreen";
import { deriveRecruitingCapabilities } from "./recruitingCapabilities";
import { useRecruitingConsoleAuthz } from "./useRecruitingConsoleAuthz";

/** Prop-less registry adapter. Backend authorization remains the authority. */
export function RecruitingScreenBody() {
  const { api, session } = useAuth();
  const gate = useRecruitingConsoleAuthz();
  const capabilities = useMemo(() => deriveRecruitingCapabilities(gate), [gate]);
  const navigate = useNavigate();
  const onNavigate = useCallback((path: string) => { void navigate(path); }, [navigate]);
  return (
    <RecruitingScreen
      api={api}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
      onNavigate={onNavigate}
    />
  );
}
