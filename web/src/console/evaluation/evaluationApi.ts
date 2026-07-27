import type { components, paths } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

type EvaluationSchemas = components["schemas"];

export type EvaluationGrade = EvaluationSchemas["EvaluationGrade"];
export type EvaluationCycleKind = EvaluationSchemas["EvaluationCycleKind"];
export type EvaluationCycleStage = EvaluationSchemas["EvaluationCycleStage"];
export type EvaluationCycleTransition = EvaluationSchemas["EvaluationCycleTransition"];
export type EvaluationSubjectState = EvaluationSchemas["EvaluationSubjectState"];
export type EvaluationReviewKind = EvaluationSchemas["EvaluationReviewKind"];
export type EvaluationReviewStatus = EvaluationSchemas["EvaluationReviewStatus"];
export type EvaluationMetricKind = EvaluationSchemas["EvaluationMetricKind"];
export type EvaluationEvidenceKind = EvaluationSchemas["EvaluationEvidenceKind"];
export type EvaluationCycleSummary = EvaluationSchemas["EvaluationCycleSummary"];
export type EvaluationCycleDetail = EvaluationSchemas["EvaluationCycleDetail"];
export type EvaluationSubjectDetail = EvaluationSchemas["EvaluationSubjectDetail"];
export type EvaluationGoal = EvaluationSchemas["EvaluationGoal"];
export type EvaluationReview = EvaluationSchemas["EvaluationReview"];
export type EvaluationPreflightReport = EvaluationSchemas["EvaluationPreflightReport"];
export type EvaluationTaskSummary = EvaluationSchemas["EvaluationTaskItem"];
export type EvaluationLedgerEntry = EvaluationSchemas["EvaluationLedgerEntry"];
export type CreateEvaluationCycleRequest = EvaluationSchemas["CreateEvaluationCycleRequest"];
export type AddEvaluationSubjectRequest = EvaluationSchemas["AddEvaluationSubjectRequest"];
export type EvaluationGoalInput = EvaluationSchemas["EvaluationGoalInput"];
export type ReplaceEvaluationGoalsRequest = EvaluationSchemas["ReplaceEvaluationGoalsRequest"];
export type EvaluationEvidenceLinkInput = EvaluationSchemas["EvaluationEvidenceLinkInput"];
export type SaveEvaluationReviewRequest = EvaluationSchemas["SaveEvaluationReviewRequest"];
export type CalibrateEvaluationRequest = EvaluationSchemas["CalibrateEvaluationSubjectRequest"];
export type ListEvaluationCyclesQuery = NonNullable<
  paths["/api/v1/evaluation/cycles"]["get"]["parameters"]["query"]
>;

export class EvaluationApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
    this.name = "EvaluationApiError";
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object";
}

function message(error: unknown, status: number): string {
  if (isRecord(error) && isRecord(error.error) && typeof error.error.message === "string") {
    return error.error.message;
  }
  return `Evaluation request failed (${String(status)})`;
}

/** Build the module's typed error from the canonical `{error:{message}}` body. */
export function evaluationApiErrorFrom(error: unknown, status: number): EvaluationApiError {
  return new EvaluationApiError(message(error, status), status);
}

function requireData<T>(result: { data?: T; error?: unknown; response: Response }): T {
  if (result.data !== undefined) return result.data;
  throw evaluationApiErrorFrom(result.error, result.response.status);
}

function reviewPathKind(kind: EvaluationReviewKind): "self" | "manager" {
  return kind === "SELF" ? "self" : "manager";
}

/** Evaluation transport bound to the authenticated generated ConsoleApiClient. */
export function createEvaluationApi(api: ConsoleApiClient) {
  return {
    listCycles: async (query?: ListEvaluationCyclesQuery, signal?: AbortSignal) =>
      requireData(
        await api.GET("/api/v1/evaluation/cycles", { params: { query }, signal }),
      ),
    createCycle: async (input: CreateEvaluationCycleRequest, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/evaluation/cycles", { body: input, signal })),
    getCycle: async (cycleId: string, signal?: AbortSignal) =>
      requireData(
        await api.GET("/api/v1/evaluation/cycles/{cycle_id}", {
          params: { path: { cycle_id: cycleId } },
          signal,
        }),
      ),
    getPreflight: async (cycleId: string, signal?: AbortSignal) =>
      requireData(
        await api.GET("/api/v1/evaluation/cycles/{cycle_id}/preflight", {
          params: { path: { cycle_id: cycleId } },
          signal,
        }),
      ),
    openCycle: async (cycleId: string, signal?: AbortSignal) =>
      requireData(
        await api.POST("/api/v1/evaluation/cycles/{cycle_id}/open", {
          params: { path: { cycle_id: cycleId } },
          signal,
        }),
      ),
    startCalibration: async (cycleId: string, signal?: AbortSignal) =>
      requireData(
        await api.POST("/api/v1/evaluation/cycles/{cycle_id}/start-calibration", {
          params: { path: { cycle_id: cycleId } },
          signal,
        }),
      ),
    finalizeCycle: async (cycleId: string, signal?: AbortSignal) =>
      requireData(
        await api.POST("/api/v1/evaluation/cycles/{cycle_id}/finalize", {
          params: { path: { cycle_id: cycleId } },
          signal,
        }),
      ),
    archiveCycle: async (cycleId: string, signal?: AbortSignal) =>
      requireData(
        await api.POST("/api/v1/evaluation/cycles/{cycle_id}/archive", {
          params: { path: { cycle_id: cycleId } },
          signal,
        }),
      ),
    addSubject: async (input: AddEvaluationSubjectRequest, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/evaluation/subjects", { body: input, signal })),
    getSubject: async (subjectId: string, signal?: AbortSignal) =>
      requireData(
        await api.GET("/api/v1/evaluation/subjects/{subject_id}", {
          params: { path: { subject_id: subjectId } },
          signal,
        }),
      ),
    replaceGoals: async (
      subjectId: string,
      input: ReplaceEvaluationGoalsRequest,
      signal?: AbortSignal,
    ) =>
      requireData(
        await api.PUT("/api/v1/evaluation/subjects/{subject_id}/goals", {
          params: { path: { subject_id: subjectId } },
          body: input,
          signal,
        }),
      ),
    saveReview: async (
      subjectId: string,
      kind: EvaluationReviewKind,
      input: SaveEvaluationReviewRequest,
      signal?: AbortSignal,
    ) =>
      requireData(
        await api.PUT("/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}", {
          params: { path: { subject_id: subjectId, kind: reviewPathKind(kind) } },
          body: input,
          signal,
        }),
      ),
    submitReview: async (subjectId: string, kind: EvaluationReviewKind, signal?: AbortSignal) =>
      requireData(
        await api.POST("/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}/submit", {
          params: { path: { subject_id: subjectId, kind: reviewPathKind(kind) } },
          signal,
        }),
      ),
    calibrateSubject: async (
      subjectId: string,
      input: CalibrateEvaluationRequest,
      signal?: AbortSignal,
    ) =>
      requireData(
        await api.POST("/api/v1/evaluation/subjects/{subject_id}/calibrate", {
          params: { path: { subject_id: subjectId } },
          body: input,
          signal,
        }),
      ),
    myTasks: async (signal?: AbortSignal) =>
      requireData(await api.GET("/api/v1/evaluation/my-tasks", { signal })),
    employeeReviews: async (employeeId: string, signal?: AbortSignal) =>
      requireData(
        await api.GET("/api/v1/evaluation/employees/{employee_id}/reviews", {
          params: { path: { employee_id: employeeId } },
          signal,
        }),
      ),
  };
}

export type EvaluationApi = ReturnType<typeof createEvaluationApi>;
