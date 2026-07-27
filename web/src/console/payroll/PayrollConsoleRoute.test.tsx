import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { payrollStrings as text } from "../../i18n/payroll";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { PayrollConsoleRoute } from "./PayrollConsoleRoute";

vi.mock("react-router", () => ({
  useNavigate: () => vi.fn(),
}));

const session: AuthSession = {
  access_token: "token",
  user_id: "admin-1",
  org_id: "org-1",
  client_session_incarnation: "session-a",
};

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function client() {
  return { GET: vi.fn(), POST: vi.fn() } as unknown as ConsoleApiClient;
}

function wireReads(api: ConsoleApiClient) {
  vi.mocked(api.GET).mockImplementation((url: string) => {
    if (url === "/api/v1/payroll/runs") {
      return Promise.resolve(ok({
        items: [{
          id: "run-1",
          period_start: "2026-06-01",
          period_end: "2026-06-30",
          source_label: "7월 정기",
          status: "STAGED",
          calculation_enabled: true,
          created_at: "2026-07-01T00:00:00Z",
          updated_at: "2026-07-01T00:00:00Z",
        }],
        total: 1,
        limit: 50,
        offset: 0,
      })) as never;
    }
    if (url.endsWith("/exceptions")) {
      return Promise.resolve(ok({ items: [], total: 0, limit: 50, offset: 0 })) as never;
    }
    return Promise.resolve(ok({
      run: {
        id: "run-1",
        period_start: "2026-06-01",
        period_end: "2026-06-30",
        source_label: "7월 정기",
        status: "STAGED",
        calculation_enabled: true,
        created_at: "2026-07-01T00:00:00Z",
        updated_at: "2026-07-01T00:00:00Z",
      },
      legal_basis: {},
      source_summary: {},
      lines: [],
      lines_total: 0,
      lines_limit: 50,
      lines_offset: 0,
      exceptions_open: 0,
      exceptions_total: 0,
      calculation: null,
      disbursement: null,
      payslip_delivery: null,
    })) as never;
  });
}

function authzResponse(capabilities: unknown[]) {
  return new Response(JSON.stringify({
    roles: ["ADMIN"],
    branch_scope: { kind: "all" },
    capabilities,
  }), { status: 200, headers: { "content-type": "application/json" } });
}

function mounted(api: ConsoleApiClient) {
  return (
    <AuthTestProvider session={session} overrides={{ api }}>
      <PayrollConsoleRoute branchId="branch-a" />
    </AuthTestProvider>
  );
}

describe("PayrollConsoleRoute", () => {
  beforeEach(() => { window.localStorage.clear(); });
  afterEach(() => vi.unstubAllGlobals());

  it("denies by omission when the parsed MeAuthzResponse carries no payroll capability", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "daily_plan_request", permission: "allow", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("mounts read-only from payroll_run_read without offering the run CTA", async () => {
    const api = client();
    wireReads(api);
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "payroll_run_read", permission: "allow", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByText(text.rosterGateStaged)).toBeVisible();
    await waitFor(() => { expect(api.GET).toHaveBeenCalledWith("/api/v1/payroll/runs", expect.anything()); });
    expect(screen.queryByRole("button", { name: text.ctaClose })).toBeNull();
  });

  it("unlocks run management from payroll_run_manage", async () => {
    const api = client();
    wireReads(api);
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "payroll_run_manage", permission: "allow", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect((await screen.findAllByRole("button", { name: text.ctaClose })).length).toBeGreaterThan(0);
  });

  it("denies request_only capabilities from the parsed MeAuthzResponse", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "payroll_run_read", permission: "request_only", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });
});
