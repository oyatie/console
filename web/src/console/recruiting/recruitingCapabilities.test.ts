import { describe, expect, it } from "vitest";

import { deriveRecruitingCapabilities } from "./recruitingCapabilities";

function gateOf(granted: string[]) {
  return { allows: ({ feature }: { feature: string }) => granted.includes(feature) };
}

describe("deriveRecruitingCapabilities", () => {
  it("denies everything by omission", () => {
    expect(deriveRecruitingCapabilities(gateOf([]))).toEqual({ canRead: false, canManage: false, canHire: false });
  });

  it("grants read-only from recruiting_read", () => {
    expect(deriveRecruitingCapabilities(gateOf(["recruiting_read"]))).toEqual({ canRead: true, canManage: false, canHire: false });
  });

  it("recruiting_manage implies read but not hire", () => {
    expect(deriveRecruitingCapabilities(gateOf(["recruiting_manage"]))).toEqual({ canRead: true, canManage: true, canHire: false });
  });

  it("hire needs recruiting_manage AND employee_directory_manage", () => {
    expect(deriveRecruitingCapabilities(gateOf(["recruiting_manage", "employee_directory_manage"])))
      .toEqual({ canRead: true, canManage: true, canHire: true });
    // Directory grant alone unlocks nothing here (grants, not roles).
    expect(deriveRecruitingCapabilities(gateOf(["employee_directory_manage"])))
      .toEqual({ canRead: false, canManage: false, canHire: false });
  });
});
