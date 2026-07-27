import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { recruitingStrings as text } from "../../i18n/recruiting";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { RecruitingScreenBody } from "./RecruitingScreenBody";

const session: AuthSession = {
  access_token: "token",
  user_id: "actor-1",
  org_id: "org-1",
  client_session_incarnation: "session-a",
};

function client() {
  return {
    GET: vi.fn(() => Promise.resolve({ data: { items: [] }, response: new Response(null, { status: 200 }) })),
    POST: vi.fn(),
    PUT: vi.fn(),
  } as unknown as ConsoleApiClient;
}

function authzResponse(capabilities: unknown[]) {
  return new Response(JSON.stringify({
    roles: [],
    branch_scope: { kind: "all" },
    capabilities,
  }), { status: 200, headers: { "content-type": "application/json" } });
}

function mounted(api: ConsoleApiClient) {
  return (
    <MemoryRouter>
      <AuthTestProvider session={session} overrides={{ api }}>
        <RecruitingScreenBody />
      </AuthTestProvider>
    </MemoryRouter>
  );
}

describe("RecruitingScreenBody", () => {
  afterEach(() => { vi.unstubAllGlobals(); });

  it("mounts read-write from a parsed MeAuthzResponse recruiting_manage capability", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "recruiting_manage", permission: "allow", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByRole("button", { name: text.newPosting })).toBeVisible();
    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledWith("/api/v1/recruiting/postings", expect.anything());
    });
  });

  it("denies by omission when the projection carries no recruiting capability", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "employee_directory_manage", permission: "allow", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("fail-closes request_only capabilities from the parsed projection", async () => {
    const api = client();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse([
      { feature: "recruiting_read", permission: "request_only", branch_scope: { kind: "all" } },
    ])));
    render(mounted(api));
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });
});
