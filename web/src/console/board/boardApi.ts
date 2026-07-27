import type { components } from "@console/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

export type NoticeProgress = components["schemas"]["NoticeProgress"];
export type BranchSummary = components["schemas"]["BranchSummary"];

export type NoticeCategory = components["schemas"]["NoticeCategory"];
export type NoticeAudienceScope = BoardNotice["audience_scope"];
export type NoticeAudienceBranch = components["schemas"]["NamedEntity"];
export type NoticeMyReceipt = components["schemas"]["NoticeMyReceipt"];
export type BoardNotice = components["schemas"]["NoticeSummary"];
export type NoticeAudienceInput = components["schemas"]["NoticeAudienceInput"];
export type CreateNoticeDraftInput = components["schemas"]["CreateNoticeDraftRequest"];
export type UpdateNoticeDraftInput = components["schemas"]["UpdateNoticeDraftRequest"];
export type NoticeReceipt = components["schemas"]["NoticeReceipt"];
export type NoticeReceiptPage = components["schemas"]["NoticeReceiptPage"];

export class BoardApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "BoardApiError";
  }
}

function message(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Board request failed (${String(status)})`;
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  throw new BoardApiError(message(response.error, response.response.status), response.response.status);
}

/** Notice transport bound to the authenticated ConsoleApiClient. */
export function createBoardApi(api: ConsoleApiClient) {
  return {
    list: async (limit?: number, signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/notices", {
        params: { query: limit === undefined ? {} : { limit } },
        signal,
      });
      return requireData(response);
    },
    get: async (id: string, signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/notices/{id}", {
        params: { path: { id } },
        signal,
      });
      return requireData(response);
    },
    createDraft: async (input: CreateNoticeDraftInput, signal?: AbortSignal) => {
      const response = await api.POST("/api/v1/notices", { body: input, signal });
      return requireData(response);
    },
    updateDraft: async (id: string, input: UpdateNoticeDraftInput, signal?: AbortSignal) => {
      const response = await api.PATCH("/api/v1/notices/{id}", {
        params: { path: { id } },
        body: input,
        signal,
      });
      return requireData(response);
    },
    publish: async (id: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/v1/notices/{id}/publish", {
        params: { path: { id } },
        signal,
      });
      return requireData(response);
    },
    ack: async (id: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/v1/notices/{id}/ack", {
        params: { path: { id } },
        signal,
      });
      if (!response.response.ok) {
        throw new BoardApiError(message(response.error, response.response.status), response.response.status);
      }
    },
    progress: async (id: string, signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/notices/{id}/progress", {
        params: { path: { id } },
        signal,
      });
      return requireData(response);
    },
    receipts: async (
      id: string,
      query: { acknowledged?: boolean; limit: number; offset: number },
      signal?: AbortSignal,
    ) => {
      const response = await api.GET("/api/v1/notices/{id}/receipts", {
        params: {
          path: { id },
          query: {
            ...(query.acknowledged === undefined ? {} : { acknowledged: query.acknowledged }),
            limit: query.limit,
            offset: query.offset,
          },
        },
        signal,
      });
      return requireData(response);
    },
    listBranches: async (signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/branches", { signal });
      return requireData(response);
    },
  };
}

export type BoardApi = ReturnType<typeof createBoardApi>;
