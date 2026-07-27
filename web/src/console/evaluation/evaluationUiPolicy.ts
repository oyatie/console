import type { EvaluationEvidenceKind, EvaluationGoalInput } from "./evaluationApi";

export type EvaluationView =
  | { kind: "cycle" }
  | { kind: "subject"; subjectId: string }
  | { kind: "person"; employeeId: string; employeeName: string };

export interface EvaluationStoredState {
  cycleId?: string;
  view?: EvaluationView;
}

const METRIC_KINDS = ["KPI", "ATTENDANCE", "TASK", "CUSTOM"] as const satisfies readonly EvaluationGoalInput["metric_kind"][];
const EVIDENCE_KINDS = ["ATTENDANCE", "WORK_ORDER", "APPROVAL", "KPI", "OTHER"] as const satisfies readonly EvaluationEvidenceKind[];

export type EvidenceRoutePolicy = {
  kind: "non-drillable";
  reason: "NO_AUTHORIZED_OBJECT_DESTINATION";
};

const EVIDENCE_ROUTE_POLICIES: Record<EvaluationEvidenceKind, EvidenceRoutePolicy> = {
  ATTENDANCE: { kind: "non-drillable", reason: "NO_AUTHORIZED_OBJECT_DESTINATION" },
  WORK_ORDER: { kind: "non-drillable", reason: "NO_AUTHORIZED_OBJECT_DESTINATION" },
  APPROVAL: { kind: "non-drillable", reason: "NO_AUTHORIZED_OBJECT_DESTINATION" },
  KPI: { kind: "non-drillable", reason: "NO_AUTHORIZED_OBJECT_DESTINATION" },
  OTHER: { kind: "non-drillable", reason: "NO_AUTHORIZED_OBJECT_DESTINATION" },
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return typeof value === "string" ? value : undefined;
}

export function parseEvaluationMetricKind(value: string): EvaluationGoalInput["metric_kind"] | undefined {
  return METRIC_KINDS.find((kind) => kind === value);
}

export function parseEvaluationEvidenceKind(value: string): EvaluationEvidenceKind | undefined {
  return EVIDENCE_KINDS.find((kind) => kind === value);
}

/**
 * Treat session storage as untrusted input. A malformed saved selection must
 * not manufacture a subject/person route from an unchecked assertion.
 */
export function restoreEvaluationState(raw: string | null): EvaluationStoredState {
  if (!raw) return {};
  try {
    const parsed: unknown = JSON.parse(raw);
    if (!isRecord(parsed)) return {};
    const cycleId = stringField(parsed, "cycleId");
    const view = isRecord(parsed.view) ? parsed.view : undefined;
    if (view?.kind === "subject") {
      const subjectId = stringField(view, "subjectId");
      if (subjectId) return { cycleId, view: { kind: "subject", subjectId } };
    }
    if (view?.kind === "person") {
      const employeeId = stringField(view, "employeeId");
      const employeeName = stringField(view, "employeeName");
      if (employeeId && employeeName) {
        return { cycleId, view: { kind: "person", employeeId, employeeName } };
      }
    }
    return { cycleId, view: { kind: "cycle" } };
  } catch {
    return {};
  }
}

/**
 * An evidence link is only drillable once the Console has an exact object
 * resolver plus an authorization contract for that destination. Broad screen
 * navigation is intentionally not a substitute for an evidence drill-down.
 */
export function evidenceRoutePolicy(kind: EvaluationEvidenceKind): EvidenceRoutePolicy {
  return EVIDENCE_ROUTE_POLICIES[kind];
}
