import { useEffect, useMemo, useState } from "react";

import { useAuth } from "../../context/auth";
import {
  fetchAuthzProjection,
  jwtFloorProjection,
  makePolicyGate,
  type AuthzProjection,
  type PolicyGate,
} from "../policy/authz";

/**
 * Recruiting's route adapter consumes the console's canonical parsed authz
 * projection (`GET /api/v1/me/authz`), failing closed to the JWT floor while
 * the authoritative read is loading or unavailable.
 */
export function useRecruitingConsoleAuthz(): PolicyGate {
  const { session } = useAuth();
  const floor = useMemo(() => jwtFloorProjection(session), [session]);
  const [authoritative, setAuthoritative] = useState<{
    token: string | undefined;
    projection: AuthzProjection;
  }>();
  const token = session?.access_token;
  const projection = authoritative && authoritative.token === token
    ? authoritative.projection
    : floor;

  useEffect(() => {
    const controller = new AbortController();
    void fetchAuthzProjection(token, controller.signal).then((next) => {
      if (!controller.signal.aborted && next) setAuthoritative({ token, projection: next });
    });
    return () => {
      controller.abort();
    };
  }, [token]);

  return useMemo(() => makePolicyGate(projection, projection.source === "authz"), [projection]);
}
