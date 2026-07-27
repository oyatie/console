/** Public, module-owned mount contract for the shared console registry. */
export interface NotifRouteContract {
  screen: "notif";
  path: "/console/notif";
}

/** Fixture is structural only: it deliberately contains no business records. */
export const NOTIF_ROUTE_CONTRACT_FIXTURE: NotifRouteContract = {
  screen: "notif",
  path: "/console/notif",
};
