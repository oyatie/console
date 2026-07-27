import { describe, expect, it } from "vitest";

import { makePolicyGate, type AuthzProjection } from "../policy/authz";
import { deriveEvaluationCapabilities } from "./evaluationCapabilities";

function projection(
  features: string[],
  branchScope: AuthzProjection["capabilities"][number]["branchScope"] = { kind: "all" },
): AuthzProjection {
  return {
    source: "authz",
    roles: [],
    branchScope: { kind: "all" },
    capabilities: features.map((feature) => ({
      feature,
      permission: "allow" as const,
      branchScope,
    })),
  };
}

describe("deriveEvaluationCapabilities", () => {
  it("maps the three evaluation features and ties calibration to manage", () => {
    const gate = makePolicyGate(
      projection(["evaluation_read", "evaluation_manage", "evaluation_submit"]),
      true,
    );
    expect(deriveEvaluationCapabilities(gate, undefined)).toEqual({
      canRead: true,
      canManage: true,
      canSubmit: true,
      canCalibrate: true,
    });
  });

  it("denies by omission when a feature is absent", () => {
    const gate = makePolicyGate(projection(["evaluation_submit"]), true);
    expect(deriveEvaluationCapabilities(gate, undefined)).toEqual({
      canRead: false,
      canManage: false,
      canSubmit: true,
      canCalibrate: false,
    });
  });

  it("intersects a branch-narrowed grant against the target branch, failing closed", () => {
    const gate = makePolicyGate(
      projection(["evaluation_read"], { kind: "branches", branches: ["branch-a"] }),
      true,
    );
    expect(deriveEvaluationCapabilities(gate, "branch-a").canRead).toBe(true);
    expect(deriveEvaluationCapabilities(gate, "branch-b").canRead).toBe(false);
  });

  it("denies everything on the empty fail-closed projection", () => {
    const gate = makePolicyGate(projection([]), false);
    expect(deriveEvaluationCapabilities(gate, "branch-a")).toEqual({
      canRead: false,
      canManage: false,
      canSubmit: false,
      canCalibrate: false,
    });
  });
});
