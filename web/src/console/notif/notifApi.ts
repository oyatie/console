import type { components } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

type Wire = components["schemas"];

export type NotificationLink = Wire["NotificationLink"];
export type ObjectHead = Wire["ObjectHead"];

export type NotificationSummary = Wire["NotificationSummary"];
export type NotificationPage = Wire["NotificationPage"];
export type NotificationCategoryCount = Wire["NotificationCategoryCount"];
export type NotificationCountsSummary = Wire["NotificationCountsSummary"];
export type NotificationObjectGroup = Wire["NotificationObjectGroup"];
export type NotificationObjectGroupPage = Wire["NotificationObjectGroupPage"];
export type NotificationPolicySummary = Wire["NotificationPolicySummary"];
export type NotificationPolicyScope = NotificationPolicySummary["scope"];
export type NotificationPolicyList = Wire["NotificationPolicyList"];
export type UpsertNotificationPolicyRequest = Wire["UpsertNotificationPolicyRequest"];

export interface NotificationListQuery {
  unread?: boolean;
  before?: string;
  limit?: number;
}

export class NotifApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "NotifApiError";
  }
}

/** Canonical error envelope `{error:{code,message}}` message, or a status fallback. */
function envelopeMessage(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Notification request failed (${String(status)})`;
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  throw new NotifApiError(envelopeMessage(response.error, response.response.status), response.response.status);
}

// ─── boundary validation for hand-typed wire shapes ─────────────────────────

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function count(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

function isLink(value: unknown): value is NotificationLink {
  if (!record(value)) return false;
  if (value.type === "object") return typeof value.kind === "string" && typeof value.id === "string";
  return value.type === "screen" && typeof value.screen === "string";
}

function isSummary(value: unknown): value is NotificationSummary {
  return record(value) && typeof value.id === "string" && typeof value.category === "string" &&
    typeof value.kind === "string" && typeof value.text === "string" &&
    typeof value.unread === "boolean" && typeof value.created_at === "string" && isLink(value.link);
}

function isCategoryCount(value: unknown): value is NotificationCategoryCount {
  return record(value) && typeof value.category === "string" && count(value.unread);
}

function isGroup(value: unknown): value is NotificationObjectGroup {
  return record(value) && isLink(value.link) && count(value.total) && count(value.unread) &&
    Array.isArray(value.categories) && value.categories.every(isCategoryCount) &&
    isSummary(value.latest) && typeof value.muted === "boolean";
}

function isGroupPage(value: unknown): value is NotificationObjectGroupPage {
  return record(value) && Array.isArray(value.items) && value.items.every(isGroup) &&
    (value.next_cursor === null || typeof value.next_cursor === "string");
}

function isPolicy(value: unknown): value is NotificationPolicySummary {
  return record(value) && typeof value.id === "string" &&
    (value.scope === "all" || value.scope === "category" || value.scope === "object") &&
    typeof value.action === "string" && typeof value.created_at === "string" &&
    (value.category === undefined || value.category === null || typeof value.category === "string") &&
    (value.link === undefined || value.link === null || isLink(value.link));
}

function isPolicyList(value: unknown): value is NotificationPolicyList {
  return record(value) && Array.isArray(value.items) && value.items.every(isPolicy);
}

function decode<T>(
  result: { data?: unknown; error?: unknown; response: Response },
  guard: (value: unknown) => value is T,
): T {
  if (result.data !== undefined && guard(result.data)) return result.data;
  throw new NotifApiError(envelopeMessage(result.error, result.response.status), result.response.status);
}

// ─── transport ──────────────────────────────────────────────────────────────

/** Principal-scoped reads opt out of browser/client caching (comms-rail rule). */
const NO_STORE = { "Cache-Control": "no-store, no-cache" } as const;

function pageQuery(query: NotificationListQuery): Record<string, unknown> {
  return {
    ...(query.unread !== undefined ? { unread: query.unread } : {}),
    ...(query.before !== undefined ? { before: query.before } : {}),
    ...(query.limit !== undefined ? { limit: query.limit } : {}),
  };
}

/** 알림 transport bound to the authenticated ConsoleApiClient. */
export function createNotifApi(api: ConsoleApiClient) {
  return {
    list: async (query: NotificationListQuery = {}, signal?: AbortSignal): Promise<NotificationPage> => {
      const response = await api.GET("/api/v1/me/notifications", {
        params: { query: pageQuery(query) },
        headers: NO_STORE,
        signal,
      });
      return requireData(response);
    },
    summary: async (signal?: AbortSignal): Promise<NotificationCountsSummary> => {
      const response = await api.GET("/api/v1/me/notifications/summary", { headers: NO_STORE, signal });
      return requireData(response);
    },
    markRead: async (id: string, signal?: AbortSignal): Promise<NotificationSummary> => {
      const response = await api.POST("/api/v1/me/notifications/{id}/read", {
        params: { path: { id } },
        signal,
      });
      return requireData(response);
    },
    markAllRead: async (signal?: AbortSignal): Promise<{ marked: number }> => {
      const response = await api.POST("/api/v1/me/notifications/read-all", { signal });
      return requireData(response);
    },
    markUnread: async (id: string, signal?: AbortSignal): Promise<NotificationSummary> => {
      const response = await api.POST("/api/v1/me/notifications/{id}/unread", { params: { path: { id } }, signal });
      return decode(response, isSummary);
    },
    listByObject: async (query: NotificationListQuery = {}, signal?: AbortSignal): Promise<NotificationObjectGroupPage> => {
      const response = await api.GET("/api/v1/me/notifications/by-object", {
        params: { query: pageQuery(query) },
        headers: NO_STORE,
        signal,
      });
      return decode(response, isGroupPage);
    },
    listPolicies: async (signal?: AbortSignal): Promise<NotificationPolicyList> => {
      const response = await api.GET("/api/v1/me/notification-policies", { headers: NO_STORE, signal });
      return decode(response, isPolicyList);
    },
    upsertPolicy: async (body: UpsertNotificationPolicyRequest, signal?: AbortSignal): Promise<NotificationPolicySummary> => {
      const response = await api.PUT("/api/v1/me/notification-policies", { body, signal });
      return decode(response, isPolicy);
    },
    deletePolicy: async (id: string, signal?: AbortSignal): Promise<void> => {
      const response = await api.DELETE("/api/v1/me/notification-policies/{id}", { params: { path: { id } }, signal });
      if (!response.response.ok) {
        throw new NotifApiError(envelopeMessage(response.error, response.response.status), response.response.status);
      }
    },
    /**
     * Source-object head for a `link.type === "object"` deep link. Any
     * failure (404 = cross-tenant/absent — indistinguishable by design) yields
     * `undefined`: the caller renders plain text, never a dead link
     * (deny-by-omission, DESIGN §4.5).
     */
    resolveObject: async (kind: string, id: string, signal?: AbortSignal): Promise<ObjectHead | undefined> => {
      try {
        const response = await api.GET("/api/objects/{kind}/{id}", {
          params: { path: { kind, id } },
          signal,
        });
        const head = requireData(response);
        return head.exists ? head : undefined;
      } catch (cause) {
        if (cause instanceof DOMException && cause.name === "AbortError") throw cause;
        return undefined;
      }
    },
  };
}

export type NotifApi = ReturnType<typeof createNotifApi>;
