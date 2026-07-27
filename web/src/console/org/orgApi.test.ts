import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { createOrgApi, OrgApiError } from "./orgApi";

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function fail(status: number, message?: string) {
  return {
    error: message ? { error: { message } } : {},
    response: new Response(null, { status }),
  };
}

function client() {
  return { GET: vi.fn(), POST: vi.fn(), PATCH: vi.fn() } as unknown as ConsoleApiClient;
}

describe("createOrgApi", () => {
  it("unwraps generated reads and surfaces the canonical error envelope", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValueOnce(ok([]));
    await expect(createOrgApi(api).regions()).resolves.toEqual([]);

    vi.mocked(api.GET).mockResolvedValueOnce(fail(403, "권한이 없습니다"));
    const denied = await createOrgApi(api).branches().catch((cause: unknown) => cause);
    expect(denied).toBeInstanceOf(OrgApiError);
    expect(denied).toMatchObject({ status: 403, message: "권한이 없습니다" });
  });

  it("routes org-change calls through the authenticated client with contract paths", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(ok({ items: [], total: 0 }));
    vi.mocked(api.POST).mockResolvedValue(ok({ id: "oc-1" }));
    const orgApi = createOrgApi(api);

    await orgApi.listChanges({ status: "IN_APPROVAL", limit: 10 });
    expect(api.GET).toHaveBeenLastCalledWith("/api/v1/org-changes", expect.objectContaining({
      params: { query: { status: "IN_APPROVAL", limit: 10 } },
    }));

    await orgApi.decide("oc-1", "step-2", { decision: "APPROVED" });
    expect(api.POST).toHaveBeenLastCalledWith(
      "/api/v1/org-changes/{id}/approval-steps/{stepId}/decision",
      expect.objectContaining({
        params: { path: { id: "oc-1", stepId: "step-2" } },
        body: { decision: "APPROVED" },
      }),
    );

    await orgApi.cancel("oc-1", "중복 기안");
    expect(api.POST).toHaveBeenLastCalledWith("/api/v1/org-changes/{id}/cancel", expect.objectContaining({
      body: { reason: "중복 기안" },
    }));
  });

  it("keeps a status-coded fallback message when the envelope is absent", async () => {
    const api = client();
    vi.mocked(api.POST).mockResolvedValueOnce(fail(409));
    const conflict = createOrgApi(api).submit("oc-1").catch((cause: unknown) => cause);
    await expect(conflict).resolves.toMatchObject({ status: 409, message: "Org request failed (409)" });
  });
});
