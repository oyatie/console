import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { maintenanceStrings as text } from "../../i18n/maintenance";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { MaintenanceConsoleRoute } from "./MaintenanceConsoleRoute";

const session = (incarnation = "session-a"): AuthSession => ({
  access_token: "token", user_id: "user-1", org_id: "org-1", client_session_incarnation: incarnation,
});
const emptyPage = { items: [], limit: 50, offset: 0, total: 0 };
const ok = <T,>(data: T) => ({ data, response: new Response(null, { status: 200 }) });
const client = () => ({ GET: vi.fn(), POST: vi.fn(), PUT: vi.fn(), PATCH: vi.fn() } as unknown as ConsoleApiClient);

function mounted(api: ConsoleApiClient, currentSession = session(), branchId = "branch-a") {
  return (
    <AuthTestProvider session={currentSession} overrides={{ api }}>
      <MaintenanceConsoleRoute branchId={branchId} />
    </AuthTestProvider>
  );
}

function authzResponse(capabilities: unknown[]) {
  return new Response(JSON.stringify({
    roles: ["MECHANIC"],
    branch_scope: { kind: "branches", branches: ["branch-a"] },
    capabilities,
  }), { status: 200, headers: { "content-type": "application/json" } });
}

describe("MaintenanceConsoleRoute", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.sessionStorage.clear();
  });

  it("mounts from the parsed MeAuthzResponse capability: allow branch A, deny branch B", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(ok(emptyPage));
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "work_order_read_all", permission: "allow", branch_scope: { kind: "branches", branches: ["branch-a"] } },
    ])));
    const view = render(mounted(api));
    expect(await screen.findByText(text.empty)).toBeVisible();
    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledWith("/api/v1/work-orders", expect.anything());
    });

    view.rerender(mounted(api, session(), "branch-b"));
    expect(await screen.findByText(text.denied)).toBeVisible();
  });

  it("denies request_only capabilities from the parsed MeAuthzResponse", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "work_order_read_all", permission: "request_only", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("fences session and API switches before stale outgoing work can update the mounted body", async () => {
    let resolveFirst: ((value: ReturnType<typeof ok<typeof emptyPage>>) => void) | undefined;
    const firstList = new Promise((resolve) => { resolveFirst = resolve; });
    const apiA = client();
    const apiB = client();
    vi.mocked(apiA.GET).mockReturnValue(firstList);
    vi.mocked(apiB.GET).mockResolvedValue(ok(emptyPage));
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "work_order_read_all", permission: "allow", branch_scope: { kind: "all" } },
    ])));
    const view = render(mounted(apiA));
    await waitFor(() => {
      expect(apiA.GET).toHaveBeenCalled();
    });
    const oldSignal = (vi.mocked(apiA.GET).mock.calls[0]?.[1] as { signal?: AbortSignal } | undefined)?.signal;

    view.rerender(mounted(apiB, session("session-b")));
    expect(await screen.findByText(text.empty)).toBeVisible();
    expect(oldSignal?.aborted).toBe(true);
    resolveFirst?.(ok(emptyPage));
    expect(screen.getByText(text.empty)).toBeVisible();
  });
});
