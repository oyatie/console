import { render, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession, ViewAsState } from "../../context/auth";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import {
  resetObjectTypeRegistry,
} from "../ontology/typeRegistrySource";
import { ontologyWorkspaceAuthorityKey } from "../ontology/useOntologyRevisionCommitQueue";
import { ConsoleModuleRoute } from "./ConsoleModuleRoute";

const genericModuleScreen = vi.hoisted(() => vi.fn(() => null));

vi.mock("./GenericModuleScreen", () => ({
  GenericModuleScreen: genericModuleScreen,
}));

const api = { GET: vi.fn() } as unknown as ConsoleApiClient;

function session(overrides: Partial<AuthSession> = {}): AuthSession {
  return {
    access_token: "bearer-token-must-not-enter-authority-key",
    org_id: "org-a",
    user_id: "user-a",
    roles: ["ADMIN"],
    feature_grants: ["object.view"],
    branches: ["branch-a"],
    client_session_incarnation: "session-a",
    ...overrides,
  };
}

function viewAs(overrides: Partial<ViewAsState> = {}): ViewAsState {
  return {
    token: "view-as-token-must-not-enter-authority-key",
    client_session_incarnation: "view-as-effective-a",
    actingOrgId: "org-view-a",
    actingOrgName: "View org A",
    actingRole: "ADMIN",
    platformSession: session({
      org_id: "platform-org",
      user_id: "platform-user",
      client_session_incarnation: "platform-session-a",
    }),
    ...overrides,
  };
}

function mounted(currentSession: AuthSession | undefined, currentViewAs?: ViewAsState, screen = "widget") {
  return (
    <MemoryRouter initialEntries={[`/modules?screen=${screen}`]}>
      <AuthTestProvider session={currentSession} overrides={{ api, viewAs: currentViewAs }}>
        <ConsoleModuleRoute />
      </AuthTestProvider>
    </MemoryRouter>
  );
}

function latestModuleProps(): { config: { id: string; objectKind: string }; authorityKey?: string } {
  const call = genericModuleScreen.mock.calls.at(-1);
  if (!call) throw new Error("GenericModuleScreen did not mount");
  return call[0] as { config: { id: string; objectKind: string }; authorityKey?: string };
}

describe("ConsoleModuleRoute dynamic ontology authority wiring", () => {
  beforeEach(() => {
    genericModuleScreen.mockClear();
    resetObjectTypeRegistry();
    api.GET = vi.fn().mockResolvedValue({
      data: [{ kind: "widget", code_prefix: null, description: "dynamic widget", status: "active", active_count: 0 }],
    });
  });

  afterEach(() => {
    resetObjectTypeRegistry();
  });

  it("passes the exact fail-closed effective authority key for every dynamic scope change", async () => {
    const contexts = [
      { session: session(), viewAs: undefined },
      { session: session({ org_id: "org-b" }), viewAs: undefined },
      { session: session({ roles: ["MECHANIC"] }), viewAs: undefined },
      { session: session({ branches: ["branch-b"] }), viewAs: undefined },
      { session: session(), viewAs: viewAs() },
      { session: session({ client_session_incarnation: "session-b" }), viewAs: undefined },
    ] as const;
    const view = render(mounted(contexts[0].session, contexts[0].viewAs));
    const authorityKeys: Array<string | undefined> = [];

    for (const context of contexts) {
      view.rerender(mounted(context.session, context.viewAs));
      await waitFor(() => {
        const props = latestModuleProps();
        expect(props.config).toMatchObject({ id: "widget", objectKind: "widget" });
        expect(props.authorityKey).toBe(
          ontologyWorkspaceAuthorityKey(context.session, context.viewAs),
        );
      });
      authorityKeys.push(latestModuleProps().authorityKey);
    }

    expect(new Set(authorityKeys).size).toBe(contexts.length);
    expect(authorityKeys.every((key) => !key?.includes("bearer-token"))).toBe(true);
  });

  it("fails closed without an owned effective session incarnation", () => {
    render(mounted(session({ client_session_incarnation: undefined })));

    expect(genericModuleScreen).not.toHaveBeenCalled();
  });

  it("keeps a hand-authored module on its existing configuration", () => {
    render(mounted(session(), undefined, "asset"));

    expect(latestModuleProps().config).toMatchObject({ id: "asset", objectKind: "equipment" });
  });

  it("bootstraps a fresh registered unknown kind under the effective authority before rendering it", async () => {
    resetObjectTypeRegistry();
    api.GET = vi.fn().mockResolvedValue({
      data: [{ kind: "widget", code_prefix: null, description: "dynamic widget", status: "active", active_count: 0 }],
    });
    const currentSession = session();

    render(mounted(currentSession));

    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledWith("/api/v1/object-types", expect.objectContaining({ signal: expect.any(AbortSignal) }));
      expect(latestModuleProps().config).toMatchObject({ id: "widget", objectKind: "widget" });
    });
    expect(latestModuleProps().authorityKey).toBe(
      ontologyWorkspaceAuthorityKey(currentSession, undefined),
    );
  });

  it("fails closed instead of falling back to another module when the dynamic registry cannot load", async () => {
    resetObjectTypeRegistry();
    api.GET = vi.fn().mockRejectedValue(new Error("registry unavailable"));

    render(mounted(session()));

    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledWith("/api/v1/object-types", expect.objectContaining({ signal: expect.any(AbortSignal) }));
    });
    expect(genericModuleScreen).not.toHaveBeenCalled();
  });

  it("reloads the dynamic registry rather than reusing a prior authority's registry", async () => {
    resetObjectTypeRegistry();
    api.GET = vi.fn().mockResolvedValue({
      data: [{ kind: "widget", code_prefix: null, description: "dynamic widget", status: "active", active_count: 0 }],
    });
    const view = render(mounted(session()));
    await waitFor(() => {
      expect(latestModuleProps().config).toMatchObject({ id: "widget" });
    });

    view.rerender(mounted(session({ client_session_incarnation: "session-b" })));

    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledTimes(2);
      expect(latestModuleProps()).toMatchObject({
        config: { id: "widget", objectKind: "widget" },
        authorityKey: ontologyWorkspaceAuthorityKey(session({ client_session_incarnation: "session-b" }), undefined),
      });
    });
  });
});
