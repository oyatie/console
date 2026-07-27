// 주소록 screen body (ConsoleShell nav "directory") — composition only. Binds
// the authenticated session, the active branch scope, and the shared console
// authz projection to the pure DirectoryScreen; the backend re-authorizes every
// call. Selection rides the `person` search param so it survives refresh/Back,
// and 메시지 lands on the messenger screen's `thread` param.
import { useCallback, useMemo } from "react";
import { useNavigate, useSearchParams } from "react-router";

import { useActiveBranchId, useAuth } from "../../context/auth";
import { useConsoleAuthz } from "../shell/authz";
import { consoleScreenPath } from "../shell/nav";
import { DirectoryScreen } from "./DirectoryScreen";
import { deriveDirectoryCapabilities } from "./directoryCapabilities";

export function DirectoryScreenBody() {
  const { api, session } = useAuth();
  const branchId = useActiveBranchId();
  const { grants } = useConsoleAuthz();
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const capabilities = useMemo(
    () => deriveDirectoryCapabilities(grants, branchId),
    [branchId, grants],
  );

  const onPersonKeyChange = useCallback((key: string | undefined) => {
    setSearchParams((current) => {
      const next = new URLSearchParams(current);
      if (key) next.set("person", key);
      else next.delete("person");
      return next;
    }, { replace: true });
  }, [setSearchParams]);

  const onOpenThread = useCallback((threadId: string) => {
    void navigate({
      pathname: consoleScreenPath("messenger"),
      search: `?thread=${encodeURIComponent(threadId)}`,
    });
  }, [navigate]);

  return (
    <DirectoryScreen
      api={api}
      branchId={branchId}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
      initialPersonKey={searchParams.get("person") ?? undefined}
      onPersonKeyChange={onPersonKeyChange}
      onOpenThread={onOpenThread}
    />
  );
}
