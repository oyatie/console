import { describe, expect, it } from "vitest";

import {
  evidenceRoutePolicy,
  parseEvaluationEvidenceKind,
  parseEvaluationMetricKind,
  restoreEvaluationState,
} from "./evaluationUiPolicy";

describe("evaluation UI boundary policy", () => {
  it("accepts only generated metric and evidence enum values from select controls", () => {
    expect(parseEvaluationMetricKind("KPI")).toBe("KPI");
    expect(parseEvaluationMetricKind("INVENTED")).toBeUndefined();
    expect(parseEvaluationEvidenceKind("WORK_ORDER")).toBe("WORK_ORDER");
    expect(parseEvaluationEvidenceKind("work_order")).toBeUndefined();
  });

  it("restores only a valid persisted Evaluation selection", () => {
    expect(
      restoreEvaluationState(
        JSON.stringify({
          cycleId: "cycle-1",
          view: { kind: "person", employeeId: "employee-1", employeeName: "김성아" },
        }),
      ),
    ).toEqual({
      cycleId: "cycle-1",
      view: { kind: "person", employeeId: "employee-1", employeeName: "김성아" },
    });
    expect(restoreEvaluationState('{"cycleId":12,"view":{"kind":"subject"}}')).toEqual({
      view: { kind: "cycle" },
    });
    expect(restoreEvaluationState("not json")).toEqual({});
  });

  it("fails closed for evidence until an exact authorized object destination exists", () => {
    for (const kind of ["ATTENDANCE", "WORK_ORDER", "APPROVAL", "KPI", "OTHER"] as const) {
      expect(evidenceRoutePolicy(kind)).toEqual({
        kind: "non-drillable",
        reason: "NO_AUTHORIZED_OBJECT_DESTINATION",
      });
    }
  });
});
