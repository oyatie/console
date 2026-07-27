import type { components } from "@console/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

/**
 * Recruiting transport bound to the authenticated ConsoleApiClient.
 *
 * CONTRACT: `/api/v1/recruiting` per the CAP-RECRUITING scout digest — the
 * backend lane builds the same contract in parallel, so every path, field, and
 * error shape here is the sync point (deviation is a defect, not a preference).
 *
 */

export type RecruitPostingStatus = components["schemas"]["RecruitPostingStatus"];
export type RecruitEmploymentType = components["schemas"]["RecruitEmploymentType"];
export type RecruitPostingScope = components["schemas"]["RecruitPostingScope"];
export type RecruitApplicantStage = components["schemas"]["RecruitApplicantStage"];
export type RecruitAssessmentScore = components["schemas"]["RecruitAssessmentScore"];
export type RecruitRejectReason = components["schemas"]["RecruitRejectReason"];
export type RecruitOfferStatus = components["schemas"]["RecruitOfferStatus"];
export type RecruitOfferPeriod = components["schemas"]["RecruitAmountPeriod"];
export type RecruitStageCounts = components["schemas"]["RecruitStageCounts"];
export type RecruitPostingView = components["schemas"]["RecruitPosting"];
export type RecruitPostingRow = components["schemas"]["RecruitPostingSummary"];
export type RecruitAssessmentView = components["schemas"]["RecruitAssessment"];
export type RecruitApplicantView = components["schemas"]["RecruitApplicant"];
export type RecruitApplicantRow = components["schemas"]["RecruitApplicantSummary"];
/**
 * The pipeline-routing fields both applicant projections carry. The list
 * projection is non-PII (no profile/assessment) and the detail projection has
 * no `assessed` flag, so neither is a subtype of the other; handlers that only
 * route take this intersection.
 */
export type RecruitApplicantRouting = Pick<
  RecruitApplicantRow,
  "id" | "posting_id" | "applicant_no" | "name" | "stage" | "hold" | "doc_requested" | "rejected_at" | "reject_reason" | "hired_employee_id" | "created_at" | "updated_at"
>;
export type RecruitOfferView = components["schemas"]["RecruitOffer"];
export type RecruitStageEventView = components["schemas"]["RecruitStageEvent"];
export type RecruitPreflightCheck = components["schemas"]["RecruitPreflightCheck"];
export type RecruitPreflightResponse = components["schemas"]["RecruitPostingPreflightResponse"];
export type RecruitPostingListResponse = components["schemas"]["RecruitPostingListResponse"];
export type RecruitPostingDetailResponse = components["schemas"]["RecruitPostingDetailResponse"];
export type RecruitApplicantDetailResponse = components["schemas"]["RecruitApplicantDetailResponse"];
export type RecruitTalentPoolItem = components["schemas"]["RecruitTalentPoolEntry"];
export type RecruitTalentPoolResponse = components["schemas"]["RecruitTalentPoolListResponse"];
export type CreateRecruitPostingRequest = components["schemas"]["CreateRecruitPostingRequest"];
export type HireRecruitApplicantRequest = components["schemas"]["HireRecruitApplicantRequest"];
export type HireRecruitApplicantResponse = components["schemas"]["HireRecruitApplicantResponse"];
export type RecruitOfferDecision = components["schemas"]["RecordRecruitOfferReplyRequest"]["decision"];

export class RecruitingApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly checks?: RecruitPreflightCheck[],
  ) {
    super(message);
    this.name = "RecruitingApiError";
  }
}

export function isDenied(error: unknown): boolean {
  return error instanceof RecruitingApiError && (error.status === 401 || error.status === 403);
}

export function isConflict(error: unknown): boolean {
  return error instanceof RecruitingApiError && error.status === 409;
}

function envelopeMessage(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Recruiting request failed (${String(status)})`;
}

const PREFLIGHT_KEYS: readonly RecruitPreflightCheck["key"][] = [
  "role_defined",
  "quota_defined",
  "no_duplicate_open",
  "exposure_attested",
];

function parseChecks(error: unknown): RecruitPreflightCheck[] | undefined {
  if (!error || typeof error !== "object") return undefined;
  const raw = (error as { checks?: unknown }).checks;
  if (!Array.isArray(raw)) return undefined;
  const checks: RecruitPreflightCheck[] = [];
  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const check = item as Record<string, unknown>;
    const key = PREFLIGHT_KEYS.find((known) => known === check.key);
    if (key === undefined || typeof check.ok !== "boolean") continue;
    checks.push({ key, ok: check.ok, note: typeof check.note === "string" ? check.note : "" });
  }
  return checks.length > 0 ? checks : undefined;
}

interface TransportResult {
  data?: unknown;
  error?: unknown;
  response: Response;
}

function requireData(result: TransportResult): unknown {
  if (result.response.ok && result.data !== undefined) return result.data;
  throw new RecruitingApiError(
    envelopeMessage(result.error, result.response.status),
    result.response.status,
    parseChecks(result.error),
  );
}

function requireOk(result: TransportResult): void {
  if (result.response.ok) return;
  throw new RecruitingApiError(
    envelopeMessage(result.error, result.response.status),
    result.response.status,
    parseChecks(result.error),
  );
}

export interface RecruitPostingListQuery {
  status?: RecruitPostingStatus;
  scope?: RecruitPostingScope;
}

/** Recruiting transport bound to the authenticated ConsoleApiClient. */
export function createRecruitingApi(api: ConsoleApiClient) {
  return {
    listPostings: async (query?: RecruitPostingListQuery, signal?: AbortSignal) => {
      const result = await api.GET("/api/v1/recruiting/postings", {
        params: { query: { status: query?.status, scope: query?.scope } },
        signal,
      });
      return requireData(result) as RecruitPostingListResponse;
    },
    getPosting: async (id: string, signal?: AbortSignal) => {
      const result = await api.GET("/api/v1/recruiting/postings/{postingId}", {
        params: { path: { postingId: id } },
        signal,
      });
      return requireData(result) as RecruitPostingDetailResponse;
    },
    createPosting: async (input: CreateRecruitPostingRequest, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/postings", { body: input, signal });
      return requireData(result) as RecruitPostingView;
    },
    updatePosting: async (
      id: string,
      input: CreateRecruitPostingRequest & { expected_updated_at: string },
      signal?: AbortSignal,
    ) => {
      const result = await api.PUT("/api/v1/recruiting/postings/{postingId}", {
        params: { path: { postingId: id } },
        body: input,
        signal,
      });
      return requireData(result) as RecruitPostingView;
    },
    preflightPosting: async (id: string, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/postings/{postingId}/preflight", {
        params: { path: { postingId: id } },
        signal,
      });
      return requireData(result) as RecruitPreflightResponse;
    },
    publishPosting: async (
      id: string,
      input: { attest_exposure_scope: boolean; expected_updated_at: string },
      signal?: AbortSignal,
    ) => {
      const result = await api.POST("/api/v1/recruiting/postings/{postingId}/publish", {
        params: { path: { postingId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    closePosting: async (id: string, input: { expected_updated_at: string }, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/postings/{postingId}/close", {
        params: { path: { postingId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    createApplicant: async (
      postingId: string,
      input: { name: string; profile_lines: string[]; source_document?: string },
      signal?: AbortSignal,
    ) => {
      const result = await api.POST("/api/v1/recruiting/postings/{postingId}/applicants", {
        params: { path: { postingId } },
        body: input,
        signal,
      });
      return requireData(result) as RecruitApplicantView;
    },
    getApplicant: async (id: string, signal?: AbortSignal) => {
      const result = await api.GET("/api/v1/recruiting/applicants/{applicantId}", {
        params: { path: { applicantId: id } },
        signal,
      });
      return requireData(result) as RecruitApplicantDetailResponse;
    },
    advanceApplicant: async (id: string, input: { expected_updated_at: string }, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/advance", {
        params: { path: { applicantId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    assessApplicant: async (id: string, input: { score: RecruitAssessmentScore }, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/assess", {
        params: { path: { applicantId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    holdApplicant: async (id: string, input: { hold: boolean }, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/hold", {
        params: { path: { applicantId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    requestDocuments: async (id: string, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/request-documents", {
        params: { path: { applicantId: id } },
        signal,
      });
      requireOk(result);
    },
    rejectApplicant: async (
      id: string,
      input: { reason: RecruitRejectReason; note?: string },
      signal?: AbortSignal,
    ) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/reject", {
        params: { path: { applicantId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    reinstateApplicant: async (id: string, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/reinstate", {
        params: { path: { applicantId: id } },
        signal,
      });
      requireOk(result);
    },
    extendOffer: async (
      id: string,
      input: { amount: string; amount_period: RecruitOfferPeriod; reply_deadline: string },
      signal?: AbortSignal,
    ) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/offer", {
        params: { path: { applicantId: id } },
        body: input,
        signal,
      });
      return requireData(result) as RecruitOfferView;
    },
    adjustOffer: async (
      id: string,
      input: { amount: string; reply_deadline?: string },
      signal?: AbortSignal,
    ) => {
      const result = await api.POST("/api/v1/recruiting/offers/{offerId}/adjust", {
        params: { path: { offerId: id } },
        body: input,
        signal,
      });
      return requireData(result) as RecruitOfferView;
    },
    withdrawOffer: async (id: string, input: { reason: string }, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/offers/{offerId}/withdraw", {
        params: { path: { offerId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    recordOfferReply: async (
      id: string,
      input: { decision: RecruitOfferDecision },
      signal?: AbortSignal,
    ) => {
      const result = await api.POST("/api/v1/recruiting/offers/{offerId}/record-reply", {
        params: { path: { offerId: id } },
        body: input,
        signal,
      });
      requireOk(result);
    },
    hireApplicant: async (id: string, input: HireRecruitApplicantRequest, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/recruiting/applicants/{applicantId}/hire", {
        params: { path: { applicantId: id } },
        body: input,
        signal,
      });
      return requireData(result) as HireRecruitApplicantResponse;
    },
    listTalentPool: async (signal?: AbortSignal) => {
      const result = await api.GET("/api/v1/recruiting/talent-pool", { signal });
      return requireData(result) as RecruitTalentPoolResponse;
    },
    listBranches: async (signal?: AbortSignal) => {
      // Branch reference data for the hire handshake — already in the generated client.
      const result = await api.GET("/api/v1/branches", { signal });
      if (result.data !== undefined) return result.data;
      throw new RecruitingApiError(
        envelopeMessage(result.error, result.response.status),
        result.response.status,
      );
    },
  };
}

export type RecruitingApi = ReturnType<typeof createRecruitingApi>;
