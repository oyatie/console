import { describe, expect, it } from "vitest";

import { deriveMaintenanceCapabilities } from "./maintenanceCapabilities";

describe("deriveMaintenanceCapabilities", () => {
  const gate = (allows: (feature: string, branch: string) => boolean) => ({
    allows: ({ feature, branch }: { feature: string; branch: string }) => allows(feature, branch),
  });

  it("derives read solely from work_order_read_all, deny-by-omission for the rest", () => {
    const result = deriveMaintenanceCapabilities(gate((feature) => feature === "work_order_read_all"), "branch-1");
    expect(result).toMatchObject({
      canRead: true,
      canCreate: false,
      canAssign: false,
      canStart: false,
      canSubmitReport: false,
      canReview: false,
      canSettle: false,
      canReviewSettlement: false,
      canTriage: false,
    });
  });

  it("maps each mutation feature to exactly its capability", () => {
    const result = deriveMaintenanceCapabilities(
      gate((feature) => ["assignee_manage", "settlement_review"].includes(feature)),
      "branch-1",
    );
    expect(result).toMatchObject({
      canRead: false,
      canAssign: true,
      canReviewSettlement: true,
      canManagePriority: false,
      canSettle: false,
    });
  });

  it("queries the effective gate for the mounted branch, not globally", () => {
    const result = deriveMaintenanceCapabilities(
      gate((feature, branch) => feature === "work_order_read_all" && branch === "branch-2"),
      "branch-1",
    );
    expect(result.canRead).toBe(false);
  });

  it("denies everything when the gate has no effective grants", () => {
    const result = deriveMaintenanceCapabilities(gate(() => false), "branch-1");
    expect(Object.values(result).every((allowed) => !allowed)).toBe(true);
  });
});
