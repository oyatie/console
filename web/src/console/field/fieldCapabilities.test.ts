import { describe, expect, it } from "vitest";

import { deriveFieldCapabilities, type FieldFeature, type FieldPolicyGate } from "./fieldCapabilities";

function gate(
  features: FieldFeature[],
  branchScoped: Partial<Record<string, string[]>> = {},
): FieldPolicyGate {
  return {
    allows: ({ feature, branch }) => {
      if (!features.includes(feature)) return false;
      const branches = branchScoped[feature];
      if (branch !== undefined && branches) return branches.includes(branch);
      return true;
    },
  };
}

describe("deriveFieldCapabilities", () => {
  it("denies everything by omission for an empty projection", () => {
    expect(deriveFieldCapabilities(gate([]), "branch-a")).toEqual({
      canRead: false,
      canIntake: false,
      canTriage: false,
      canAccept: false,
      canComment: false,
    });
  });

  it("maps the login tier to read + branch-scoped intake only", () => {
    expect(deriveFieldCapabilities(gate(["login"]), "branch-a")).toEqual({
      canRead: true,
      canIntake: true,
      canTriage: false,
      canAccept: false,
      canComment: false,
    });
  });

  it("keeps intake off when the login capability excludes the active branch", () => {
    const capabilities = deriveFieldCapabilities(
      gate(["login"], { login: ["branch-b"] }),
      "branch-a",
    );
    expect(capabilities.canRead).toBe(true);
    expect(capabilities.canIntake).toBe(false);
  });

  it("keeps intake off without an active branch", () => {
    expect(deriveFieldCapabilities(gate(["login"]), "").canIntake).toBe(false);
  });

  it("maps assignee_manage to triage + acceptance and work_order_start to comment", () => {
    const capabilities = deriveFieldCapabilities(
      gate(["work_order_read_all", "assignee_manage", "work_order_start"]),
      "branch-a",
    );
    expect(capabilities).toEqual({
      canRead: true,
      canIntake: false,
      canTriage: true,
      canAccept: true,
      canComment: true,
    });
  });
});
