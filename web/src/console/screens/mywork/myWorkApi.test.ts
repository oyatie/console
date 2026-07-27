import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../../api/client";
import { createMyWorkApi } from "./myWorkApi";
import { kstLocalDateTimeToIso } from "./myWorkModel";

function clientWithGet(get: ReturnType<typeof vi.fn>): ConsoleApiClient {
  return { GET: get } as unknown as ConsoleApiClient;
}

describe("createMyWorkApi action-inbox pagination", () => {
  it("loads one bounded page and preserves its cursor and total semantics", async () => {
    const get = vi.fn().mockResolvedValue({
      data: {
        items: [{ id: "work:1" }],
        total: 501,
        total_is_exact: false,
        next_cursor: "cursor-1",
      },
    });

    const result = await createMyWorkApi(clientWithGet(get)).loadInbox();

    expect(result.items.map((item) => item.id)).toEqual(["work:1"]);
    expect(result).toMatchObject({
      total: 501,
      total_is_exact: false,
      next_cursor: "cursor-1",
    });
    expect(get).toHaveBeenCalledTimes(1);
    expect(get).toHaveBeenCalledWith("/api/v1/me/action-inbox", {
      params: { query: { limit: 200, cursor: undefined } },
    });
  });

  it("rejects a cursor that does not advance", async () => {
    const get = vi.fn().mockResolvedValue({
      data: { items: [{ id: "work:2" }], total: 2, next_cursor: "same" },
    });

    await expect(createMyWorkApi(clientWithGet(get)).loadInbox("same")).rejects.toThrow(
      "action-inbox cursor did not advance",
    );
    expect(get).toHaveBeenCalledTimes(1);
  });
});

describe("createMyWorkApi bounded calendar workbench", () => {
  it("reads the aggregate without recreating source reads and creates only a personal calendar event", async () => {
    const get = vi.fn().mockResolvedValue({ data: { calendar: { status: "ok", items: [] } } });
    const created = {
      id: "00000000-0000-0000-0000-000000000051",
      scope_type: "PERSONAL",
      title: "Focus time",
      description: "",
      starts_at: "2026-07-10T00:00:00.000Z",
      ends_at: "2026-07-10T01:00:00.000Z",
      all_day: false,
      status: "ACTIVE",
      created_at: "2026-07-08T09:00:00Z",
      updated_at: "2026-07-08T09:00:00Z",
      policy: { enforcement: "server", scope_type: "PERSONAL", visibility: "creator_only" },
    };
    const post = vi.fn().mockResolvedValue({ data: created, error: undefined });
    const api = createMyWorkApi({ GET: get, POST: post } as unknown as ConsoleApiClient);

    await api.loadWorkbench();
    await expect(api.createPersonalCalendarEvent({
      title: "Focus time",
      startsAt: "2026-07-10T00:00:00.000Z",
      endsAt: "2026-07-10T01:00:00.000Z",
    })).resolves.toEqual(created);

    expect(get).toHaveBeenCalledWith("/api/v1/me/workbench", { params: { query: {} } });
    expect(post).toHaveBeenCalledWith("/api/v1/collaboration/calendar/events", {
      body: {
        scope_type: "PERSONAL",
        title: "Focus time",
        starts_at: "2026-07-10T00:00:00.000Z",
        ends_at: "2026-07-10T01:00:00.000Z",
        all_day: false,
      },
    });
  });

  it("fails closed when a nominal calendar write lacks its created-event receipt", async () => {
    const post = vi.fn().mockResolvedValue({ error: undefined });
    const api = createMyWorkApi({ GET: vi.fn(), POST: post } as unknown as ConsoleApiClient);

    await expect(api.createPersonalCalendarEvent({
      title: "Focus time",
      startsAt: "2026-07-10T00:00:00.000Z",
      endsAt: "2026-07-10T01:00:00.000Z",
    })).rejects.toThrow("calendar event failed");
  });

  it("converts only complete KST datetime-local values to instants", () => {
    expect(kstLocalDateTimeToIso("2026-07-10T09:00")).toBe("2026-07-10T00:00:00.000Z");
    expect(kstLocalDateTimeToIso("2026-07-10")).toBeUndefined();
  });
});
