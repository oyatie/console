import { describe, expect, it } from "vitest";

import { deriveOrgCapabilities, type OrgFeature } from "./orgCapabilities";

function gateOf(granted: OrgFeature[]) {
  return {
    allows: ({ feature }: { feature: OrgFeature }) => granted.includes(feature),
  };
}

describe("deriveOrgCapabilities", () => {
  it("denies everything by omission", () => {
    expect(deriveOrgCapabilities(gateOf([]))).toEqual({
      canReadTree: false,
      canReadChanges: false,
      canDraft: false,
      canApprove: false,
      canApply: false,
    });
  });

  it("maps each backend feature gate onto its capability", () => {
    const capabilities = deriveOrgCapabilities(
      gateOf(["employee_directory_read", "org_change_read", "org_change_draft"]),
    );
    expect(capabilities).toEqual({
      canReadTree: true,
      canReadChanges: true,
      canDraft: true,
      canApprove: false,
      canApply: false,
    });
    expect(deriveOrgCapabilities(gateOf(["org_change_approve", "org_change_apply"]))).toMatchObject({
      canApprove: true,
      canApply: true,
      canReadTree: false,
    });
  });
});
