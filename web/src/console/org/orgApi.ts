import type { components, operations } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

export type RegionSummary = components["schemas"]["RegionSummary"];
export type BranchSummary = components["schemas"]["BranchSummary"];
export type HrOrgChartResponse = components["schemas"]["HrOrgChartResponse"];
export type HrOrgChartCompany = components["schemas"]["HrOrgChartCompany"];
export type HrOrgChartUnit = components["schemas"]["HrOrgChartUnit"];
export type SelfProfile = components["schemas"]["UserSummary"];

/**
 * Org-change governance contract (CAP-ORG-CONSOLE scout digest). The backend
 * lane builds `backend/crates/orgchange/rest` against the same digest; until
 * the OpenAPI schema lands via the integrator these DTOs are the sync point —
 * any deviation from the digest is a defect on whichever side deviated.
 */
export type OrgChangeKind = components["schemas"]["OrgChangeKind"];
export type OrgChangeStatus = components["schemas"]["OrgChangeStatus"];
export type OrgChangeTargetKind = components["schemas"]["OrgChangeTargetKind"];
export type OrgChangeTarget = components["schemas"]["OrgChangeTarget"];
export type OrgProposalOp = components["schemas"]["OrgProposalOp"];
export type OrgApprovalRoleKey = components["schemas"]["OrgChangeApprovalRoleKey"];
export type OrgApprovalDecision = components["schemas"]["OrgChangeStepDecision"];
export type OrgChangeApprovalStep = components["schemas"]["OrgChangeApprovalStep"];
export type OrgSettlementItemKey = components["schemas"]["OrgChangeSettlementKey"];
export type OrgChangeSettlementItem = components["schemas"]["OrgChangeSettlementItem"];
export type OrgChangeEvent = components["schemas"]["OrgChangeEvent"];
export type OrgChangePreflightBlocker = components["schemas"]["OrgChangePreflightBlocker"];
export type OrgChangePreflightWarning = components["schemas"]["OrgChangePreflightWarning"];
export type OrgChangePreflightReport = components["schemas"]["OrgChangePreflightReport"];
export type OrgChangeSummary = components["schemas"]["OrgChangeSummary"];
export type OrgChangeDetail = components["schemas"]["OrgChangeDetail"];
export type OrgChangePage = components["schemas"]["OrgChangePage"];
export type CreateOrgChangeRequest = components["schemas"]["CreateOrgChangeRequest"];
export type UpdateOrgChangeDraftRequest = components["schemas"]["UpdateOrgChangeDraftRequest"];
export type OrgEntitySummary = components["schemas"]["OrgEntitySummary"];
export type OrgChangeDecisionRequest = components["schemas"]["OrgChangeDecisionRequest"];
export type ListOrgChangesQuery = NonNullable<operations["listOrgChanges"]["parameters"]["query"]>;

export class OrgApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "OrgApiError";
  }
}

function message(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Org request failed (${String(status)})`;
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  throw new OrgApiError(message(response.error, response.response.status), response.response.status);
}

/** Org module transport bound to the authenticated ConsoleApiClient. */
export function createOrgApi(api: ConsoleApiClient) {
  return {
    regions: async (signal?: AbortSignal): Promise<RegionSummary[]> => {
      const response = await api.GET("/api/v1/regions", { signal });
      return requireData(response);
    },
    branches: async (signal?: AbortSignal): Promise<BranchSummary[]> => {
      const response = await api.GET("/api/v1/branches", { signal });
      return requireData(response);
    },
    orgChart: async (signal?: AbortSignal): Promise<HrOrgChartResponse> => {
      const response = await api.GET("/api/v1/hr/org-chart", { signal });
      return requireData(response);
    },
    me: async (signal?: AbortSignal): Promise<SelfProfile> => {
      const response = await api.GET("/api/v1/users/me", { signal });
      return requireData(response);
    },
    entities: async (signal?: AbortSignal) =>
      requireData(await api.GET("/api/v1/org-entities", { signal })),
    listChanges: async (query?: ListOrgChangesQuery, signal?: AbortSignal) =>
      requireData(await api.GET("/api/v1/org-changes", { params: { query: query ?? {} }, signal })),
    getChange: async (id: string, signal?: AbortSignal) =>
      requireData(await api.GET("/api/v1/org-changes/{id}", { params: { path: { id } }, signal })),
    createChange: async (input: CreateOrgChangeRequest, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes", {
        // The handler dedupes a replayed create on this key.
        params: { header: { "Idempotency-Key": crypto.randomUUID() } },
        body: input,
        signal,
      })),
    updateDraft: async (id: string, input: UpdateOrgChangeDraftRequest, signal?: AbortSignal) =>
      requireData(await api.PATCH("/api/v1/org-changes/{id}", { params: { path: { id } }, body: input, signal })),
    preflight: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/preflight", { params: { path: { id } }, signal })),
    submit: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/submit", { params: { path: { id } }, signal })),
    decide: async (id: string, stepId: string, input: OrgChangeDecisionRequest, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/approval-steps/{stepId}/decision", {
        params: { path: { id, stepId } },
        body: input,
        signal,
      })),
    effectuate: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/effectuate", { params: { path: { id } }, signal })),
    completeSettlement: async (id: string, itemId: string, memo?: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/settlement-items/{itemId}/complete", {
        params: { path: { id, itemId } },
        body: memo === undefined ? {} : { memo },
        signal,
      })),
    archive: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/archive", { params: { path: { id } }, signal })),
    cancel: async (id: string, reason: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/org-changes/{id}/cancel", { params: { path: { id } }, body: { reason }, signal })),
  };
}

export type OrgApi = ReturnType<typeof createOrgApi>;
