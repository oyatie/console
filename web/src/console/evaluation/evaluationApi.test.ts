import { afterEach, describe, expect, it, vi } from "vitest";

import { createConsoleApiClient, type ConsoleApiClient } from "../../api/client";
import { createEvaluationApi } from "./evaluationApi";

function client() {
  const impl = { GET: vi.fn(), POST: vi.fn(), PUT: vi.fn() };
  return { impl, api: impl as unknown as ConsoleApiClient };
}

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

describe("createEvaluationApi", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("uses the authenticated console client bearer and the evaluation endpoint", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [], limit: 50, offset: 0, total: 0 }), {
        status: 200,
        headers: { "content-type": "application/json" },
      }),
    );
    vi.stubGlobal("fetch", fetchMock);
    await createEvaluationApi(createConsoleApiClient("bearer-token")).listCycles({
      stage: "OPEN",
    });
    const request = fetchMock.mock.calls[0]?.[0] as Request;
    expect(request.url).toContain("/api/v1/evaluation/cycles?stage=OPEN");
    expect(request.headers.get("Authorization")).toBe("Bearer bearer-token");
    expect(request.headers.get("X-Auth-Transport")).toBe("cookie");
  });

  it("forwards generated path params, lowercased review kind, and contract bodies", async () => {
    const { impl, api } = client();
    impl.PUT.mockResolvedValue(ok({ id: "review-1" }));
    await createEvaluationApi(api).saveReview("subject/1", "MANAGER", {
      grade: "A",
      note: "확인",
      evidence_links: [
        { object_kind: "KPI", object_ref: "KPI-SLA", label: "고객 응답" },
      ],
    });
    expect(impl.PUT).toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}",
      expect.objectContaining({
        params: { path: { subject_id: "subject/1", kind: "manager" } },
        body: expect.objectContaining({ grade: "A", note: "확인" }),
      }),
    );
  });

  it("targets the audited employee ledger endpoint", async () => {
    const { impl, api } = client();
    impl.GET.mockResolvedValue(ok({ items: [] }));
    await createEvaluationApi(api).employeeReviews("employee-1");
    expect(impl.GET).toHaveBeenCalledWith(
      "/api/v1/evaluation/employees/{employee_id}/reviews",
      expect.objectContaining({ params: { path: { employee_id: "employee-1" } } }),
    );
  });

  it("surfaces the backend error envelope instead of synthesizing success", async () => {
    const { impl, api } = client();
    impl.POST.mockResolvedValue({
      error: { error: { message: "denied" } },
      response: new Response(null, { status: 403 }),
    });
    await expect(
      createEvaluationApi(api).submitReview("subject-1", "SELF"),
    ).rejects.toThrow("denied");
  });
});
