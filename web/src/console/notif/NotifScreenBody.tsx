import { useAuth } from "../../context/auth";
import { NotifScreen } from "./NotifScreen";
import { deriveNotifCapabilities } from "./notifCapabilities";

/** Registry-ready adapter (SCREEN_REGISTRY `notif`). Backend `/me/*` scoping
 *  remains the authority; the client only gates on an authenticated session. */
export function NotifScreenBody() {
  const { api, session } = useAuth();
  const capabilities = deriveNotifCapabilities(session !== undefined);
  return (
    <NotifScreen
      api={api}
      actorId={session?.user_id}
      capabilities={capabilities}
      sessionKey={session?.client_session_incarnation ?? session?.access_token}
    />
  );
}
