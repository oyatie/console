import { describe, expect, it } from "vitest";

import { derivePayrollCapabilities, type PayrollPolicyGate } from "./payrollCapabilities";

function gateOf(features: string[]): PayrollPolicyGate {
  return { allows: ({ feature }) => features.includes(feature) };
}

describe("derivePayrollCapabilities", () => {
  it("denies everything when no payroll feature is granted", () => {
    expect(derivePayrollCapabilities(gateOf([]), "branch-a")).toEqual({
      canRead: false,
      canManage: false,
      canDecide: false,
      canReadSelf: true,
    });
  });

  it("grants read-only from payroll_run_read alone", () => {
    const capabilities = derivePayrollCapabilities(gateOf(["payroll_run_read"]), "branch-a");
    expect(capabilities.canRead).toBe(true);
    expect(capabilities.canManage).toBe(false);
    expect(capabilities.canDecide).toBe(false);
  });

  it("grants read and manage from payroll_run_manage", () => {
    const capabilities = derivePayrollCapabilities(gateOf(["payroll_run_manage"]), "branch-a");
    expect(capabilities).toEqual({ canRead: true, canManage: true, canDecide: true, canReadSelf: true });
  });

  it("passes the target branch through to the gate query", () => {
    const seen: string[] = [];
    const gate: PayrollPolicyGate = {
      allows: ({ branch }) => {
        seen.push(branch);
        return true;
      },
    };
    derivePayrollCapabilities(gate, "branch-b");
    expect(new Set(seen)).toEqual(new Set(["branch-b"]));
  });
});
