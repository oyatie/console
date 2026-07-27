import { useMemo } from "react";

import { useAuth } from "../../../context/auth";
import { MyWorkBody } from "./MyWorkBody";
import { createMyWorkApi } from "./myWorkApi";
import { canOpenCalendarOwner } from "./myWorkModel";

/** Shell-mounted entry (ConsoleShell nav "mywork"): binds the personal
 *  work API (action-inbox + todos) to the authenticated console client. */
export default function MyWorkScreen() {
  const { api, session } = useAuth();
  const myWorkApi = useMemo(() => createMyWorkApi(api), [api]);
  const canOpenOwner = useMemo(
    () => canOpenCalendarOwner(session?.roles, session?.feature_grants),
    [session?.feature_grants, session?.roles],
  );
  return <MyWorkBody api={myWorkApi} canOpenCalendarOwner={canOpenOwner} />;
}
