import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { fieldStrings as text } from "../../i18n/field";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { FieldConsoleRoute, FieldScreenBody } from "./FieldConsoleRoute";

const session = (): AuthSession => ({
  access_token: "token",
  user_id: "user-1",
  org_id: "org-1",
  client_session_incarnation: "session-a",
  branches: ["branch-a"],
});

const page = {
  items: [
    {
      site_id: "site-1",
      site_name: "대원강업 상주",
      branch_id: "branch-a",
      customer_id: "customer-1",
      customer_name: "대원강업",
      address: null,
      latitude: null,
      longitude: null,
      open_ticket_count: 0,
      breached_ticket_count: 0,
      next_due_at: null,
      active_work_order_count: 0,
      last_arrival_at: null,
      sla: "OK",
    },
  ],
  next_cursor: null,
  total: 1,
};

const ok = <T,>(data: T) => ({ data, response: new Response(null, { status: 200 }) });
const client = () => ({ GET: vi.fn(), POST: vi.fn() }) as unknown as ConsoleApiClient;

function authzResponse(capabilities: unknown[]) {
  return new Response(
    JSON.stringify({
      roles: ["OPERATOR"],
      branch_scope: { kind: "branches", branches: ["branch-a"] },
      capabilities,
    }),
    { status: 200, headers: { "content-type": "application/json" } },
  );
}

describe("FieldConsoleRoute", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.localStorage.clear();
  });

  it("mounts the screen from the parsed MeAuthzResponse login capability", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(ok(page));
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        authzResponse([
          { feature: "login", permission: "allow", branch_scope: { kind: "all" } },
        ]),
      ),
    );
    render(
      <AuthTestProvider session={session()} overrides={{ api }}>
        <FieldConsoleRoute branchId="branch-a" />
      </AuthTestProvider>,
    );
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledWith("/api/v1/field/sites", expect.anything());
    });
  });

  it("denies by omission when the projection carries no capabilities", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([])));
    render(
      <AuthTestProvider session={session()} overrides={{ api }}>
        <FieldConsoleRoute branchId="branch-a" />
      </AuthTestProvider>,
    );
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("denies request_only capabilities from the parsed MeAuthzResponse", async () => {
    const api = client();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        authzResponse([
          { feature: "login", permission: "request_only", branch_scope: { kind: "all" } },
        ]),
      ),
    );
    render(
      <AuthTestProvider session={session()} overrides={{ api }}>
        <FieldConsoleRoute branchId="branch-a" />
      </AuthTestProvider>,
    );
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("resolves the active branch for the registry zero-prop body", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(ok(page));
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        authzResponse([
          { feature: "login", permission: "allow", branch_scope: { kind: "branches", branches: ["branch-a"] } },
        ]),
      ),
    );
    render(
      <AuthTestProvider session={session()} overrides={{ api }}>
        <FieldScreenBody />
      </AuthTestProvider>,
    );
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
    // Branch-scoped login on the active branch keeps the intake affordance on.
    expect(screen.getByRole("button", { name: text.intake })).toBeVisible();
  });
});
