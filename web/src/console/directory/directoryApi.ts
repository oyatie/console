import type { components } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

export type DirectoryMember = components["schemas"]["MessengerMemberSummary"];
export type DirectoryEmployee = components["schemas"]["Employee"];
export type DirectoryEmployeePage = components["schemas"]["EmployeePage"];
export type DirectoryLifecycleEvent = components["schemas"]["EmployeeLifecycleEvent"];
export type DirectoryThread = components["schemas"]["MessengerThreadSummary"];

export class DirectoryApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "DirectoryApiError";
  }
}

/** 401/403 render as deny-by-omission, never as a technical failure. */
export function isDenied(error: unknown): boolean {
  return error instanceof DirectoryApiError && (error.status === 401 || error.status === 403);
}

/** 404 is the no-leak "not visible" answer for person reads. */
export function isBlocked(error: unknown): boolean {
  return (
    error instanceof DirectoryApiError &&
    (error.status === 401 || error.status === 403 || error.status === 404)
  );
}

function message(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Directory request failed (${String(status)})`;
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  throw new DirectoryApiError(message(response.error, response.response.status), response.response.status);
}

/** Directory transport bound to the authenticated ConsoleApiClient. */
export function createDirectoryApi(api: ConsoleApiClient) {
  return {
    /**
     * Branch member roster — the non-privileged people surface (id/name/team
     * only). Single page: the endpoint has no offset/total, so a branch beyond
     * `limit` truncates silently (same documented ceiling as
     * `composer/candidates.ts`; paging is a backend charter, GAP-DIR-5).
     */
    listMembers: async (branchId: string, signal?: AbortSignal) => {
      const response = await api.GET("/api/messenger/members", {
        params: { query: { branch_id: branchId, limit: 100 } },
        signal,
      });
      return requireData(response).items;
    },
    /**
     * Read-audited person view. The server records a `person.view` audit event
     * for a non-self open inside the read; an out-of-scope target is 404 with
     * no audit. Person cards MUST go through this call — never around it.
     */
    getMember: async (userId: string, branchId: string, signal?: AbortSignal) => {
      const response = await api.GET("/api/messenger/members/{userId}", {
        params: { path: { userId }, query: { branch_id: branchId } },
        signal,
      });
      return requireData(response);
    },
    /** HR register (feature `employee_directory_read`); server typeahead + paging. */
    listEmployees: async (
      query: { search?: string; company?: string; limit: number; offset: number },
      signal?: AbortSignal,
    ) => {
      const response = await api.GET("/api/v1/employees", {
        params: {
          query: {
            limit: query.limit,
            offset: query.offset,
            ...(query.search ? { search: query.search } : {}),
            ...(query.company ? { company: query.company } : {}),
          },
        },
        signal,
      });
      return requireData(response);
    },
    /** Audited lifecycle ledger, newest first (feature `employee_directory_read`). */
    listLifecycleEvents: async (employeeId: string, signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/employees/{id}/lifecycle-events", {
        params: { path: { id: employeeId } },
        signal,
      });
      return requireData(response).items;
    },
    /** 대화 개설 — a real audited DM thread (`message_thread.create`). */
    createDmThread: async (branchId: string, memberId: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/messenger/threads", {
        body: { branch_id: branchId, kind: "dm", member_ids: [memberId] },
        signal,
      });
      return requireData(response);
    },
  };
}

export type DirectoryApi = ReturnType<typeof createDirectoryApi>;
