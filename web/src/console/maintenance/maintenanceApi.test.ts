import { createConsoleApiClient, type ConsoleApiClient } from "../../api/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { createMaintenanceApi, MaintenanceApiError } from "./maintenanceApi";

describe("createMaintenanceApi", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("uses the authenticated console client bearer and typed work-order endpoint", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ items: [], limit: 50, offset: 0, total: 0 }), {
      status: 200, headers: { "content-type": "application/json" },
    }));
    vi.stubGlobal("fetch", fetchMock);
    await createMaintenanceApi(createConsoleApiClient("bearer-token")).list({ status: ["UNASSIGNED"], priority: ["P1"] });
    const request = fetchMock.mock.calls[0]?.[0] as Request;
    expect(request.url).toContain("/api/v1/work-orders?");
    expect(request.url).toContain("status=UNASSIGNED");
    expect(request.url).toContain("priority=P1");
    expect(request.headers.get("Authorization")).toBe("Bearer bearer-token");
  });

  it("forwards mutation params and bodies through the client instead of a module fetch", async () => {
    const api = { POST: vi.fn().mockResolvedValue({ data: { id: "wo-1" }, response: new Response(null, { status: 200 }) }) } as unknown as ConsoleApiClient;
    await createMaintenanceApi(api).reject("wo/a", "memo");
    expect(api.POST).toHaveBeenCalledWith("/api/v1/work-orders/{workOrderId}/reject", expect.objectContaining({
      params: { path: { workOrderId: "wo/a" } }, body: { memo: "memo" },
    }));
  });

  it("surfaces the canonical backend error envelope instead of synthesizing success", async () => {
    const api = { POST: vi.fn().mockResolvedValue({ error: { error: { message: "denied" } }, response: new Response(null, { status: 403 }) }) } as unknown as ConsoleApiClient;
    const call = createMaintenanceApi(api).start("wo-1");
    await expect(call).rejects.toThrow("denied");
    await expect(createMaintenanceApi(api).start("wo-1")).rejects.toBeInstanceOf(MaintenanceApiError);
  });

  it("treats a missing settlement (404) as absent, not as an error", async () => {
    const api = { GET: vi.fn().mockResolvedValue({ error: { error: { message: "not found" } }, response: new Response(null, { status: 404 }) }) } as unknown as ConsoleApiClient;
    await expect(createMaintenanceApi(api).settlement("wo-1")).resolves.toBeUndefined();
    expect(api.GET).toHaveBeenCalledWith("/api/v1/work-orders/{workOrderId}/settlement", expect.objectContaining({
      params: { path: { workOrderId: "wo-1" } },
    }));
  });

  it("posts settlement reviews to the settlement FSM routes", async () => {
    const api = { POST: vi.fn().mockResolvedValue({ data: { id: "st-1", work_order_id: "wo-1", status: "RETURNED", lines: [] }, response: new Response(null, { status: 200 }) }) } as unknown as ConsoleApiClient;
    await createMaintenanceApi(api).reviewSettlement("st-1", { decision: "RETURNED", comment: "insufficient" });
    expect(api.POST).toHaveBeenCalledWith("/api/v1/settlements/{settlementId}/review", expect.objectContaining({
      params: { path: { settlementId: "st-1" } }, body: { decision: "RETURNED", comment: "insufficient" },
    }));
  });
});
