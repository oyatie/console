import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { evaluationStrings as text } from "../../i18n/evaluation";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { EvaluationConsoleRoute } from "./EvaluationConsoleRoute";

const session = (incarnation = "session-a"): AuthSession => ({
  access_token: "token",
  user_id: "user-mgr",
  org_id: "org-1",
  branches: ["branch-a"],
  client_session_incarnation: incarnation,
});

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function client() {
  const impl = { GET: vi.fn(), POST: vi.fn(), PUT: vi.fn() };
  return { impl, api: impl as unknown as ConsoleApiClient };
}

function authzResponse(capabilities: unknown[]) {
  return new Response(
    JSON.stringify({
      roles: ["ADMIN"],
      branch_scope: { kind: "all" },
      capabilities,
    }),
    { status: 200, headers: { "content-type": "application/json" } },
  );
}

function mounted(api: ConsoleApiClient, currentSession = session()) {
  return (
    <MemoryRouter initialEntries={["/console/evaluation"]}>
      <AuthTestProvider session={currentSession} overrides={{ api }}>
        <EvaluationConsoleRoute />
      </AuthTestProvider>
    </MemoryRouter>
  );
}

describe("EvaluationConsoleRoute", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    sessionStorage.clear();
  });

  it("mounts from the parsed MeAuthzResponse evaluation capabilities", async () => {
    const { impl, api } = client();
    impl.GET.mockImplementation((path: string) =>
      Promise.resolve(
        path === "/api/v1/evaluation/my-tasks"
          ? ok({ items: [], limit: 50, offset: 0, total: 0 })
          : ok({ items: [], limit: 50, offset: 0, total: 0 }),
      ),
    );
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        authzResponse([
          { feature: "evaluation_read", permission: "allow", branch_scope: { kind: "all" } },
          { feature: "evaluation_submit", permission: "allow", branch_scope: { kind: "all" } },
        ]),
      ),
    );
    render(mounted(api));
    expect(await screen.findByText(text.tasksEmpty)).toBeVisible();
    await waitFor(() => {
      expect(impl.GET).toHaveBeenCalledWith("/api/v1/evaluation/cycles", expect.anything());
    });
  });

  it("denies request_only capabilities from the parsed MeAuthzResponse", async () => {
    const { impl, api } = client();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        authzResponse([
          { feature: "evaluation_read", permission: "request_only", branch_scope: { kind: "all" } },
        ]),
      ),
    );
    render(mounted(api));
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(impl.GET).not.toHaveBeenCalled();
  });

  it("fences a session switch before stale outgoing work can update the mounted body", async () => {
    let resolveFirst!: (value: unknown) => void;
    const first = new Promise((resolve) => {
      resolveFirst = resolve;
    });
    const staleImpl = { GET: vi.fn().mockReturnValue(first), POST: vi.fn(), PUT: vi.fn() };
    const staleApi = staleImpl as unknown as ConsoleApiClient;
    const { impl, api } = client();
    impl.GET.mockImplementation((path: string) =>
      Promise.resolve(
        path === "/api/v1/evaluation/cycles"
          ? ok({
              items: [
                {
                  id: "cycle-1",
                  name: "2026 상반기 정기평가",
                  kind: "REGULAR",
                  period_label: "2026 H1",
                  due_date: "2099-07-18",
                  stage: "CALIBRATION",
                  subjects_total: 1,
                  manager_submitted: 1,
                  self_submitted: 1,
                  calibrated: 0,
                  finalized: 0,
                  created_at: "2026-07-01T00:00:00Z",
                },
              ],
              limit: 50,
              offset: 0,
              total: 1,
            })
          : ok({ items: [], limit: 50, offset: 0, total: 0 }),
      ),
    );
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        authzResponse([
          { feature: "evaluation_read", permission: "allow", branch_scope: { kind: "all" } },
        ]),
      ),
    );
    const view = render(mounted(staleApi));
    await waitFor(() => {
      expect(staleImpl.GET).toHaveBeenCalledTimes(1);
    });
    const staleCall = staleImpl.GET.mock.calls[0] as unknown[] | undefined;
    const staleSignal = (staleCall?.[1] as { signal?: AbortSignal } | undefined)?.signal;

    view.rerender(mounted(api, session("session-b")));
    expect(
      await screen.findByRole("button", { name: /2026 상반기 정기평가/ }),
    ).toHaveTextContent(text.stage.CALIBRATION);
    expect(staleSignal?.aborted).toBe(true);
    resolveFirst(ok({ items: [], limit: 50, offset: 0, total: 0 }));
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /2026 상반기 정기평가/ }),
      ).toHaveTextContent(text.stage.CALIBRATION);
    });
  });
});
