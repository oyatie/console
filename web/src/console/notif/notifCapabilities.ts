export interface NotifCapabilities {
  canRead: boolean;
  canAck: boolean;
  canMute: boolean;
}

/**
 * 알림 is an ungated all-employee `/me/*` surface: the backend scopes every
 * route to the JWT principal (cross-user = 404), so the only client-side
 * condition is an authenticated session. The capability shape stays so a
 * policy gate can slot in without touching the screen.
 */
export function deriveNotifCapabilities(authenticated: boolean): NotifCapabilities {
  return { canRead: authenticated, canAck: authenticated, canMute: authenticated };
}
