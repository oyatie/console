import type { components, operations } from "@console/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

/** 고객·현장 transport, on the typed generated client throughout. */

export type TicketStatus = components["schemas"]["SupportTicketStatus"];
export type TicketPriority = components["schemas"]["SupportTicketPriority"];
export type TicketCategory = components["schemas"]["SupportTicketCategory"];
export type TicketComment = components["schemas"]["SupportTicketComment"];
export type CreateTicketRequest = components["schemas"]["CreateInternalTicketRequest"];
export type TicketSummary = components["schemas"]["SupportTicketSummary"];

export interface TicketDetail {
  ticket: TicketSummary;
  comments: TicketComment[];
}

export interface TicketPage {
  items: TicketSummary[];
  next_cursor: string | null;
  total: number;
}

export type FieldSlaState = components["schemas"]["FieldSlaState"];
export type FieldSiteRow = components["schemas"]["FieldSiteRow"];
export type FieldSitePage = components["schemas"]["FieldSitePage"];
export type FieldSiteSummary = components["schemas"]["FieldSiteSummary"];
export type FieldSlaSummary = components["schemas"]["FieldSlaSummary"];
export type FieldWorkOrderRef = components["schemas"]["FieldWorkOrderRef"];
export type FieldAttendanceEvent = components["schemas"]["FieldAttendanceEvent"];
export type FieldSiteDetail = components["schemas"]["FieldSiteDetail"];
export type AcceptanceKind = components["schemas"]["SupportTicketAcceptanceKind"];
export type AcceptanceChannel = components["schemas"]["SupportTicketAcceptanceChannel"];
export type TicketAcceptanceView = components["schemas"]["SupportTicketAcceptance"];
export type RecordAcceptanceRequest = components["schemas"]["RecordSupportTicketAcceptanceRequest"];
export type LinkTicketRequest = components["schemas"]["LinkSupportTicketRequest"];
export type ListFieldSitesQuery = NonNullable<operations["listFieldSites"]["parameters"]["query"]>;

export class FieldApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "FieldApiError";
  }
}

function errorMessage(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Field request failed (${String(status)})`;
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  throw new FieldApiError(errorMessage(response.error, response.response.status), response.response.status);
}


/** Field transport bound to the authenticated ConsoleApiClient. */
export function createFieldApi(api: ConsoleApiClient) {
  return {
    listSites: async (query: ListFieldSitesQuery = {}, signal?: AbortSignal): Promise<FieldSitePage> => {
      const response = await api.GET("/api/v1/field/sites", {
        params: { query: { ...query } },
        signal,
      });
      return requireData(response);
    },
    getSite: async (siteId: string, signal?: AbortSignal): Promise<FieldSiteDetail> => {
      const response = await api.GET("/api/v1/field/sites/{id}", {
        params: { path: { id: siteId } },
        signal,
      });
      return requireData(response);
    },
    listTickets: async (
      query: { include_untriaged?: boolean; limit?: number; cursor?: string } = {},
      signal?: AbortSignal,
    ): Promise<TicketPage> => {
      const response = await api.GET("/api/v1/support/tickets", {
        params: { query },
        signal,
      });
      return requireData(response);
    },
    getTicket: async (ticketId: string, signal?: AbortSignal): Promise<TicketDetail> => {
      const response = await api.GET("/api/v1/support/tickets/{id}", {
        params: { path: { id: ticketId } },
        signal,
      });
      return requireData(response);
    },
    createTicket: async (input: CreateTicketRequest, signal?: AbortSignal): Promise<TicketSummary> => {
      const response = await api.POST("/api/v1/support/tickets", { body: input, signal });
      return requireData(response);
    },
    assignTicket: async (
      ticketId: string,
      input: { assignee_user_id: string; branch_id?: string },
      signal?: AbortSignal,
    ): Promise<TicketSummary> => {
      const response = await api.POST("/api/v1/support/tickets/{id}/assign", {
        params: { path: { id: ticketId } },
        body: input,
        signal,
      });
      return requireData(response);
    },
    transitionTicket: async (
      ticketId: string,
      toStatus: TicketStatus,
      signal?: AbortSignal,
    ): Promise<TicketSummary> => {
      const response = await api.POST("/api/v1/support/tickets/{id}/transition", {
        params: { path: { id: ticketId } },
        body: { to_status: toStatus },
        signal,
      });
      return requireData(response);
    },
    addComment: async (
      ticketId: string,
      input: { body: string; is_internal_note?: boolean },
      signal?: AbortSignal,
    ): Promise<TicketComment> => {
      const response = await api.POST("/api/v1/support/tickets/{id}/comments", {
        params: { path: { id: ticketId } },
        body: input,
        signal,
      });
      return requireData(response);
    },
    linkTicket: async (ticketId: string, input: LinkTicketRequest, signal?: AbortSignal): Promise<TicketSummary> => {
      const response = await api.POST("/api/v1/support/tickets/{id}/link", {
        params: { path: { id: ticketId } },
        body: input,
        signal,
      });
      return requireData(response);
    },
    recordAcceptance: async (
      ticketId: string,
      input: RecordAcceptanceRequest,
      signal?: AbortSignal,
    ) => {
      const response = await api.POST("/api/v1/support/tickets/{id}/acceptance", {
        // Server dedupes replays on this key; an identical retry returns the
        // stored acceptance instead of recording a second one.
        params: { path: { id: ticketId }, header: { "Idempotency-Key": crypto.randomUUID() } },
        body: input,
        signal,
      });
      return requireData(response);
    },
  };
}

export type FieldApi = ReturnType<typeof createFieldApi>;
