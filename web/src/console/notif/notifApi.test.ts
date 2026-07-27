import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { NotifApiError, createNotifApi, type NotificationObjectGroup, type NotificationSummary } from "./notifApi";

const summary = (over: Partial<NotificationSummary> = {}): NotificationSummary => ({
  id: "5a4b3c2d-1e0f-4a9b-8c7d-6e5f4a3b2c1d",
  recipient_user_id: "0a1b2c3d-4e5f-4a6b-8c7d-8e9f0a1b2c3d",
  category: "결재",
  kind: "info",
  text: "AP-3121 결재 대기",
  link: { type: "object", kind: "approval_run", id: "9a8b7c6d-5e4f-4a3b-8c1d-0e9f8a7b6c5d" },
  unread: true,
  created_at: "2026-07-24T09:00:00Z",
  read_at: null,
  resolved_at: null,
  ...over,
});

const group = (over: Partial<NotificationObjectGroup> = {}): NotificationObjectGroup => ({
  link: { type: "object", kind: "approval_run", id: "9a8b7c6d-5e4f-4a3b-8c1d-0e9f8a7b6c5d" },
  total: 3,
  unread: 2,
  categories: [{ category: "결재", unread: 2 }],
  latest: summary(),
  muted: false,
  ...over,
});

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function client(overrides: Record<string, unknown> = {}) {
  return { GET: vi.fn(), POST: vi.fn(), PUT: vi.fn(), DELETE: vi.fn(), ...overrides } as unknown as ConsoleApiClient;
}

type Mocked = ConsoleApiClient & {
  GET: ReturnType<typeof vi.fn>;
  POST: ReturnType<typeof vi.fn>;
  PUT: ReturnType<typeof vi.fn>;
  DELETE: ReturnType<typeof vi.fn>;
};

describe("createNotifApi", () => {
  it("pages the typed list route with no-store principal reads", async () => {
    const api = client() as Mocked;
    api.GET.mockResolvedValue(ok({ items: [summary()], next_cursor: null }));
    const page = await createNotifApi(api).list({ unread: true, before: "cursor-1", limit: 50 });
    expect(page.items).toHaveLength(1);
    expect(api.GET).toHaveBeenCalledWith("/api/v1/me/notifications", expect.objectContaining({
      params: { query: { unread: true, before: "cursor-1", limit: 50 } },
      headers: { "Cache-Control": "no-store, no-cache" },
    }));
  });

  it("calls the contract by-object route and validates the group page shape", async () => {
    const api = client() as Mocked;
    api.GET.mockResolvedValue(ok({ items: [group()], next_cursor: "opaque" }));
    const page = await createNotifApi(api).listByObject({ unread: true });
    expect(page.items[0].unread).toBe(2);
    expect(api.GET).toHaveBeenCalledWith("/api/v1/me/notifications/by-object", expect.objectContaining({
      params: { query: { unread: true } },
    }));
  });

  it("rejects a malformed by-object payload instead of rendering fabricated groups", async () => {
    const api = client() as Mocked;
    api.GET.mockResolvedValue(ok({ items: [{ link: { type: "object" }, total: 1 }], next_cursor: null }));
    await expect(createNotifApi(api).listByObject()).rejects.toBeInstanceOf(NotifApiError);
  });

  it("marks unread through the contract route and returns the authoritative row", async () => {
    const api = client() as Mocked;
    api.POST.mockResolvedValue(ok(summary({ unread: true, read_at: "2026-07-24T09:05:00Z" })));
    const row = await createNotifApi(api).markUnread("id/한");
    expect(row.read_at).toBe("2026-07-24T09:05:00Z");
    // openapi-fetch templates the path parameter and encodes it itself.
    expect(api.POST).toHaveBeenCalledWith("/api/v1/me/notifications/{id}/unread", expect.objectContaining({
      params: { path: { id: "id/한" } },
    }));
  });

  it("upserts and deletes mute policies over the contract routes", async () => {
    const api = client() as Mocked;
    api.PUT.mockResolvedValue(ok({ id: "p-1", scope: "object", link: group().link, action: "mute", created_at: "2026-07-24T09:00:00Z" }));
    api.DELETE.mockResolvedValue({ response: new Response(null, { status: 204 }) });
    const notifApi = createNotifApi(api);
    const policy = await notifApi.upsertPolicy({ scope: "object", link: group().link });
    expect(policy.action).toBe("mute");
    expect(api.PUT).toHaveBeenCalledWith("/api/v1/me/notification-policies", expect.objectContaining({
      body: { scope: "object", link: group().link },
    }));
    await notifApi.deletePolicy("p-1");
    expect(api.DELETE).toHaveBeenCalledWith("/api/v1/me/notification-policies/{id}", expect.objectContaining({
      params: { path: { id: "p-1" } },
    }));
  });

  it("surfaces the canonical error envelope message and status", async () => {
    const api = client() as Mocked;
    api.POST.mockResolvedValue({ error: { error: { code: "unauthorized", message: "denied" } }, response: new Response(null, { status: 401 }) });
    const failure = await createNotifApi(api).markAllRead().catch((cause: unknown) => cause);
    expect(failure).toBeInstanceOf(NotifApiError);
    expect((failure as NotifApiError).message).toBe("denied");
    expect((failure as NotifApiError).status).toBe(401);
  });

  it("resolves a source-object head and treats any denial as absence", async () => {
    const api = client() as Mocked;
    api.GET
      .mockResolvedValueOnce(ok({ kind: "approval_run", id: "x", code: "AP-3121", title: null, status: null, exists: true }))
      .mockResolvedValueOnce({ error: { error: { message: "not found" } }, response: new Response(null, { status: 404 }) });
    const notifApi = createNotifApi(api);
    const head = await notifApi.resolveObject("approval_run", "x");
    expect(head?.code).toBe("AP-3121");
    await expect(notifApi.resolveObject("approval_run", "y")).resolves.toBeUndefined();
  });
});
