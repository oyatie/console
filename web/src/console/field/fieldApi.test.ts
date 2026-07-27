import { afterEach, describe, expect, it, vi } from "vitest";

import { createConsoleApiClient, type ConsoleApiClient } from "../../api/client";
import { createFieldApi, FieldApiError } from "./fieldApi";

function jsonResponse(data: unknown, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "content-type": "application/json" },
  });
}

describe("createFieldApi", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("lists sites through the authenticated client with contract query params", async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ items: [], next_cursor: null, total: 0 }));
    vi.stubGlobal("fetch", fetchMock);
    const page = await createFieldApi(createConsoleApiClient("bearer-token")).listSites({
      q: "안산",
      sla: "BREACHED",
      limit: 100,
    });
    expect(page).toEqual({ items: [], next_cursor: null, total: 0 });
    const request = fetchMock.mock.calls[0]?.[0] as Request;
    const url = new URL(request.url);
    expect(url.pathname).toBe("/api/v1/field/sites");
    expect(url.searchParams.get("q")).toBe("안산");
    expect(url.searchParams.get("sla")).toBe("BREACHED");
    expect(url.searchParams.get("limit")).toBe("100");
    expect(request.headers.get("Authorization")).toBe("Bearer bearer-token");
    expect(request.headers.get("X-Auth-Transport")).toBe("cookie");
  });

  it("templates the site-detail path through the client", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        site: {},
        sla: {},
        tickets: [],
        work_orders: [],
        attendance: [],
        acceptances: [],
      }),
    );
    vi.stubGlobal("fetch", fetchMock);
    await createFieldApi(createConsoleApiClient("bearer-token")).getSite("site-9");
    const request = fetchMock.mock.calls[0]?.[0] as Request;
    expect(new URL(request.url).pathname).toBe("/api/v1/field/sites/site-9");
  });

  it("forwards acceptance body and path through the client instead of a module fetch", async () => {
    const api = {
      GET: vi.fn(),
      POST: vi.fn().mockResolvedValue({
        data: { id: "acc-1" },
        response: new Response(null, { status: 201 }),
      }),
    } as unknown as ConsoleApiClient;
    await createFieldApi(api).recordAcceptance("ticket-1", {
      kind: "CUSTOMER_ACCEPTED",
      channel: "PHONE",
      accepted_by: "김확인",
    });
    expect(vi.mocked(api.POST)).toHaveBeenCalledWith(
      "/api/v1/support/tickets/{id}/acceptance",
      expect.objectContaining({
        params: {
          path: { id: "ticket-1" },
          // The handler requires an Idempotency-Key so a retried acceptance
          // returns the stored one instead of recording a second.
          header: { "Idempotency-Key": expect.any(String) as unknown as string },
        },
        body: { kind: "CUSTOMER_ACCEPTED", channel: "PHONE", accepted_by: "김확인" },
      }),
    );
  });

  it("routes ticket reads through the typed support endpoint", async () => {
    const api = {
      GET: vi.fn().mockResolvedValue({
        data: { items: [], next_cursor: null, total: 0 },
        response: new Response(null, { status: 200 }),
      }),
      POST: vi.fn(),
    } as unknown as ConsoleApiClient;
    await createFieldApi(api).listTickets({ include_untriaged: true, limit: 50 });
    expect(vi.mocked(api.GET)).toHaveBeenCalledWith(
      "/api/v1/support/tickets",
      expect.objectContaining({ params: { query: { include_untriaged: true, limit: 50 } } }),
    );
  });

  it("surfaces the canonical error envelope instead of synthesizing success", async () => {
    const api = {
      GET: vi.fn(),
      POST: vi.fn().mockResolvedValue({
        error: { error: { code: "conflict", message: "work order site mismatch" } },
        response: new Response(null, { status: 409 }),
      }),
    } as unknown as ConsoleApiClient;
    const failure = createFieldApi(api).linkTicket("ticket-1", { site_id: "site-1" });
    await expect(failure).rejects.toThrow("work order site mismatch");
    await expect(
      createFieldApi(api).linkTicket("ticket-1", { site_id: "site-1" }),
    ).rejects.toMatchObject({ name: "FieldApiError", status: 409 });
  });

  it("keeps the typed error class distinguishable for denied-versus-error rendering", () => {
    const error = new FieldApiError("missing", 404);
    expect(error.status).toBe(404);
    expect(error).toBeInstanceOf(Error);
  });
});
