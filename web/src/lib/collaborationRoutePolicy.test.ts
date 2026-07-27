import { describe, expect, it } from "vitest";

import { isNavItemVisible } from "../components/shell/nav";
import { canOpenCalendarOwner } from "../console/screens/mywork/myWorkModel";
import { canPresentCollaborationRoute } from "./collaborationRoutePolicy";

describe("collaboration route presentation policy", () => {
  it("keeps the nav guard and My Work receipt affordance aligned", () => {
    const cases = [
      { roles: ["MEMBER"], featureGrants: [] },
      { roles: ["MECHANIC"], featureGrants: [] },
      { roles: ["MEMBER"], featureGrants: ["work_order_read_all"] },
      { roles: ["MEMBER"], featureGrants: ["work_order_create"] },
      { roles: undefined, featureGrants: undefined },
    ] as const;

    for (const { roles, featureGrants } of cases) {
      const expected = isNavItemVisible(
        "collaboration",
        roles,
        undefined,
        featureGrants,
      );
      expect(canPresentCollaborationRoute(roles, featureGrants)).toBe(expected);
      expect(canOpenCalendarOwner(roles, featureGrants)).toBe(expected);
    }
  });
});
