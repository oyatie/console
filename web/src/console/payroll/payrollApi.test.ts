import { createConsoleApiClient, type ConsoleApiClient } from "../../api/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { createPayrollApi, PayrollApiError } from "./payrollApi";

describe("createPayrollApi", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("uses the authenticated console client bearer and typed run endpoint", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(
      JSON.stringify({ items: [], total: 0, limit: 50, offset: 0 }),
      { status: 200, headers: { "content-type": "application/json" } },
    ));
    vi.stubGlobal("fetch", fetchMock);
    await createPayrollApi(createConsoleApiClient("bearer-token")).listRuns();
    const request = fetchMock.mock.calls[0]?.[0] as Request;
    expect(request.url).toContain("/api/v1/payroll/runs");
    expect(request.headers.get("Authorization")).toBe("Bearer bearer-token");
    expect(request.headers.get("X-Auth-Transport")).toBe("cookie");
  });

  it("forwards contract-route params and bodies through the client instead of a module fetch", async () => {
    const api = {
      GET: vi.fn(),
      POST: vi.fn().mockResolvedValue({ data: { id: "ex-1" }, response: new Response(null, { status: 200 }) }),
    } as unknown as ConsoleApiClient;
    await createPayrollApi(api).resolveException("run-1", "ex-1", { action: "HOLD", reason: "계좌 재확인" });
    expect(api.POST).toHaveBeenCalledWith("/api/v1/payroll/runs/{id}/exceptions/{exceptionId}/resolve", expect.objectContaining({
      params: { path: { id: "run-1", exceptionId: "ex-1" } },
      body: { action: "HOLD", reason: "계좌 재확인" },
    }));
  });

  it("walks run-line pages to the server total instead of truncating the roster", async () => {
    const lineOf = (id: string) => ({
      id,
      employee_display_name: id,
      employee_company: "knl",
      gross_pay_source_present: true,
      net_pay_source_present: true,
      nts_tax_row_status: "VERIFIED_SOURCE_ROW",
      calculation_status: "READY_FOR_REVIEW",
      blockers: [],
    });
    const all = [lineOf("l-1"), lineOf("l-2"), lineOf("l-3")];
    const detailAt = (offset: number, lines: unknown[]) => ({
      run: { id: "run-1" },
      legal_basis: {},
      source_summary: {},
      lines,
      lines_total: all.length,
      lines_limit: 2,
      lines_offset: offset,
    });
    const api = {
      GET: vi.fn((_url: string, init?: { params?: { query?: { offset?: number } } }) => {
        const offset = init?.params?.query?.offset ?? 0;
        return Promise.resolve({
          data: detailAt(offset, offset === 0 ? all.slice(0, 2) : all.slice(offset)),
          response: new Response(null, { status: 200 }),
        });
      }),
      POST: vi.fn(),
    } as unknown as ConsoleApiClient;
    const detail = await createPayrollApi(api).getRun("run-1");
    expect(detail.lines.map((line) => line.id)).toEqual(["l-1", "l-2", "l-3"]);
    expect(api.GET).toHaveBeenCalledTimes(2);
  });

  it("surfaces the canonical error envelope code instead of synthesizing success", async () => {
    const api = {
      GET: vi.fn(),
      POST: vi.fn().mockResolvedValue({
        error: { error: { code: "sod_violation", message: "denied" } },
        response: new Response(null, { status: 409 }),
      }),
    } as unknown as ConsoleApiClient;
    const failure = await createPayrollApi(api).decide("run-1", { decision: "APPROVE" }).catch((cause: unknown) => cause);
    expect(failure).toBeInstanceOf(PayrollApiError);
    expect((failure as PayrollApiError).message).toBe("denied");
    expect((failure as PayrollApiError).code).toBe("sod_violation");
    expect((failure as PayrollApiError).status).toBe(409);
  });
});
