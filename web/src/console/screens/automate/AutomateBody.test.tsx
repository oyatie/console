// Composition smoke test — AutomateHub itself (rule list, canvas builder, run
// log, version-pending banner) is exhaustively covered by
// pages/AutomatePage.test.tsx; this file only proves AutomateBody mounts it
// correctly under its own BulkPolicyGateProvider (empty/error/loaded states).
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { afterAll, afterEach, beforeAll, describe, expect, it } from "vitest";
import { MemoryRouter, useLocation, useNavigate } from "react-router";

import { clearAuthorizeBulkCache } from "../../../api/authorizeBulk";
import { createConsoleApiClient } from "../../../api/client";
import { AuthContext } from "../../../context/auth";
import type { AuthContextValue, AuthSession } from "../../../context/auth";
import { ko } from "../../../i18n/ko";
import { allowAllBulkAuthorize } from "../../../test/policyGateMock";
import { AutomateBody } from "./AutomateBody";

const S = ko.console.automate;
const scheduledDefinition = {
  id: "44444444-4444-4444-8444-444444444444",
  workflow_key: "automate.schedule.kpi",
  display_name: "일일 KPI 스냅샷",
  object_type: "work_order",
  status: "ACTIVE",
  latest_version: 1,
  active_version: 1,
  pending_version: null,
  pending_staged_by: null,
  approval_line: [],
  payment_line: [],
  notification_rules: [],
  action_allowlist: [],
  required_approval_line: false,
  required_payment_line: false,
  created_at: "2026-07-08T09:00:00Z",
  updated_at: "2026-07-08T09:00:00Z",
  definition: {
    schema_version: "workflow.definition.v1",
    trigger: "automate.object_change",
    steps: [],
    automate: { scope: "org", doc: null, condition: null },
    schedule: {
      name: "일일 KPI 스냅샷",
      active: true,
      cron: "0 9 * * *",
      cron_label: "매일",
      next_run_at: "07-10 09:00",
      last_run_at: "07-09 09:00",
    },
  },
};
const durableSchedule = {
  id: "55555555-5555-4555-8555-555555555555",
  label: "일일 KPI 스냅샷",
  cron_expr: "0 9 * * *",
  timezone: "Asia/Seoul",
  definition_id: scheduledDefinition.id,
  enabled: true,
  next_run_at: null,
  last_run_at: null,
  last_status: null,
  created_at: "2026-07-08T09:00:00Z",
  updated_at: "2026-07-08T09:00:00Z",
};

const server = setupServer(allowAllBulkAuthorize());
beforeAll(() => {
  server.listen({ onUnhandledRequest: "bypass" });
});
afterEach(() => {
  server.resetHandlers();
  clearAuthorizeBulkCache();
});
afterAll(() => {
  server.close();
});

function sessionWith(roles: readonly string[]): AuthSession {
  return {
    access_token: "token",
    user_id: "00000000-0000-4000-8000-0000000000aa",
    display_name: "개발자",
    roles: [...roles],
    group_roles: [],
    feature_grants: [],
    org_id: "00000000-0000-0000-0000-0000000000a1",
    branches: ["00000000-0000-4000-8000-000000000001"],
    isPlatform: false,
  };
}

function authValue(roles: readonly string[]): AuthContextValue {
  const session = sessionWith(roles);
  return {
    session,
    restoring: false,
    login: () => Promise.resolve(),
    logout: () => Promise.resolve(),
    refresh: () => Promise.resolve(),
    acceptTokens: () => undefined,
    clearPasskeySetup: () => undefined,
    api: createConsoleApiClient(() => session.access_token),
    viewAs: undefined,
    enterViewAs: () => undefined,
    exitViewAs: () => undefined,
  };
}

// No injected policy provider — the body owns its own role gate (the R4 fix);
// mounting it bare is what proves SUPER_ADMIN gets tabs while others don't.
function RouterProbe() {
  const location = useLocation();
  const navigate = useNavigate();
  return (
    <>
      <output data-testid="location">{`${location.pathname}${location.search}${location.hash}`}</output>
      <button type="button" onClick={() => void navigate(-1)}>
        history back
      </button>
      <button type="button" onClick={() => void navigate(1)}>
        history forward
      </button>
    </>
  );
}

function renderBody(
  roles: readonly string[] = ["SUPER_ADMIN"],
  initialEntries: string[] = ["/console/workflow"],
) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <AuthContext.Provider value={authValue(roles)}>
        <AutomateBody />
      </AuthContext.Provider>
      <RouterProbe />
    </MemoryRouter>,
  );
}

function expectLocation(expected: string) {
  expect(screen.getByTestId("location").textContent).toBe(expected);
}

function installHandlers(items: unknown[] = [], scheduleItems: unknown[] = []) {
  server.use(
    http.get("*/api/v1/ontology/object-types", () => HttpResponse.json([])),
    http.get("*/api/v1/workflow-studio/definitions", () => HttpResponse.json({ items })),
    http.get("*/api/v1/workflow-studio/definitions/:id/run-log", () =>
      HttpResponse.json({ items: [] }),
    ),
    http.get("*/api/v1/workflow-studio/schedules", () =>
      HttpResponse.json({ items: scheduleItems }),
    ),
  );
}

describe("AutomateBody (console screen composition)", () => {
  it("mounts the real workflow-studio tabs and an empty rule list — no fabricated rows", async () => {
    installHandlers([]);
    renderBody();

    expect(await screen.findByRole("tab", { name: S.tabs.rules, selected: true })).toBeVisible();
    expect(screen.getByRole("tab", { name: S.tabs.schedules })).toBeVisible();
    expect(screen.getByRole("tab", { name: S.tabs.monitors })).toBeVisible();
    expect(screen.getByText(S.labels.noSelection)).toBeVisible();
  });

  it.each([
    ["/console/workflow", "rules"],
    ["/console/scheduled", "schedules"],
  ] as const)("direct load and reload of %s retain the route-authoritative %s surface", async (path, tab) => {
    installHandlers([]);

    const firstLoad = renderBody(["SUPER_ADMIN"], [path]);
    if (tab === "rules")
      expect(await screen.findByRole("tab", { name: S.tabs.rules, selected: true })).toBeVisible();
    else expect(await screen.findByRole("heading", { name: "예약 작업" })).toBeVisible();
    firstLoad.unmount();

    renderBody(["SUPER_ADMIN"], [path]);
    if (tab === "rules")
      expect(await screen.findByRole("tab", { name: S.tabs.rules, selected: true })).toBeVisible();
    else expect(await screen.findByRole("heading", { name: "예약 작업" })).toBeVisible();
    expectLocation(path);
  });

  it("keeps one durable schedule workspace with definition-backed create, edit, and run actions at /console/scheduled", async () => {
    installHandlers([scheduledDefinition], [durableSchedule]);
    renderBody(["SUPER_ADMIN"], ["/console/scheduled"]);

    const detail = await screen.findByRole("region", { name: "예약 상세" });
    expect(screen.getByRole("form", { name: "예약 작업 추가" })).toBeVisible();
    expect(screen.getByRole("option", { name: "일일 KPI 스냅샷" })).toBeVisible();
    expect(within(detail).getByRole("button", { name: "지금 실행" })).toBeVisible();
    expect(within(detail).getByRole("button", { name: "예약 편집" })).toBeVisible();
  });

  it("keeps the scheduled route to one main landmark", async () => {
    installHandlers([]);
    const view = render(
      <main aria-label="콘솔 콘텐츠">
        <MemoryRouter initialEntries={["/console/scheduled"]}>
          <AuthContext.Provider value={authValue(["SUPER_ADMIN"])}>
            <AutomateBody />
          </AuthContext.Provider>
        </MemoryRouter>
      </main>,
    );

    await screen.findByRole("heading", { name: "예약 작업" });
    const results = await axe.run(view.container, {
      runOnly: { type: "rule", values: ["landmark-one-main", "landmark-unique"] },
    });
    expect(results.violations).toEqual([]);
  });

  it("writes tab changes to history and follows browser back and forward", async () => {
    installHandlers([]);
    renderBody();

    expect(await screen.findByRole("tab", { name: S.tabs.rules, selected: true })).toBeVisible();
    await userEvent.click(screen.getByRole("tab", { name: S.tabs.schedules }));
    expectLocation("/console/scheduled");
    expect(await screen.findByRole("heading", { name: "예약 작업" })).toBeVisible();

    await userEvent.click(screen.getByRole("button", { name: "history back" }));
    await waitFor(() => {
      expectLocation("/console/workflow");
      expect(screen.getByRole("tab", { name: S.tabs.rules })).toHaveAttribute(
        "aria-selected",
        "true",
      );
    });

    await userEvent.click(screen.getByRole("button", { name: "history forward" }));
    await waitFor(() => {
      expectLocation("/console/scheduled");
      expect(screen.getByRole("heading", { name: "예약 작업" })).toBeVisible();
    });
  });

  it("gives the monitor tab a reloadable workflow sub-route", async () => {
    installHandlers([]);
    const initialView = renderBody();

    await screen.findByRole("tab", { name: S.tabs.rules, selected: true });
    await userEvent.click(screen.getByRole("tab", { name: S.tabs.monitors }));

    expectLocation("/console/workflow?tab=monitors");
    expect(screen.getByRole("tab", { name: S.tabs.monitors })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    initialView.unmount();
    renderBody(["SUPER_ADMIN"], ["/console/workflow?tab=monitors"]);
    expect(
      await screen.findByRole("tab", { name: S.tabs.monitors, selected: true }),
    ).toBeVisible();
  });

  it.each([
    [
      "/console/scheduled?keep=1&tab=monitors#anchor",
      "/console/scheduled?keep=1#anchor",
      S.tabs.schedules,
    ],
    [
      "/console/workflow?tab=unsupported&keep=1#anchor",
      "/console/workflow?keep=1#anchor",
      S.tabs.rules,
    ],
  ] as const)("replace-canonicalizes stale tab state in %s", async (input, expected, selectedTab) => {
    installHandlers([]);
    renderBody(["SUPER_ADMIN"], ["/sentinel", input]);

    if (selectedTab === S.tabs.schedules)
      expect(await screen.findByRole("heading", { name: "예약 작업" })).toBeVisible();
    else expect(await screen.findByRole("tab", { name: selectedTab, selected: true })).toBeVisible();
    await waitFor(() => {
      expectLocation(expected);
    });

    await userEvent.click(screen.getByRole("button", { name: "history back" }));
    await waitFor(() => {
      expectLocation("/sentinel");
    });
  });

  it("preserves unrelated query and hash while switching tabs", async () => {
    installHandlers([]);
    renderBody(["SUPER_ADMIN"], ["/console/workflow?keep=1#anchor"]);

    await screen.findByRole("tab", { name: S.tabs.rules, selected: true });
    await userEvent.click(screen.getByRole("tab", { name: S.tabs.monitors }));
    expectLocation("/console/workflow?keep=1&tab=monitors#anchor");

    await userEvent.click(screen.getByRole("tab", { name: S.tabs.schedules }));
    expectLocation("/console/scheduled?keep=1#anchor");
  });

  it("tracks monitor history with exact back and forward locations", async () => {
    installHandlers([]);
    renderBody();

    await screen.findByRole("tab", { name: S.tabs.rules, selected: true });
    await userEvent.click(screen.getByRole("tab", { name: S.tabs.monitors }));
    expectLocation("/console/workflow?tab=monitors");

    await userEvent.click(screen.getByRole("button", { name: "history back" }));
    await waitFor(() => {
      expectLocation("/console/workflow");
      expect(screen.getByRole("tab", { name: S.tabs.rules })).toHaveAttribute(
        "aria-selected",
        "true",
      );
    });

    await userEvent.click(screen.getByRole("button", { name: "history forward" }));
    await waitFor(() => {
      expectLocation("/console/workflow?tab=monitors");
      expect(screen.getByRole("tab", { name: S.tabs.monitors })).toHaveAttribute(
        "aria-selected",
        "true",
      );
    });
  });

  it.each([
    ["/console/workflow", S.tabs.rules],
    ["/console/workflow?keep=1&tab=monitors#anchor", S.tabs.monitors],
  ] as const)("does not add a history entry when the current tab at %s is selected", async (path, tabLabel) => {
    installHandlers([]);
    renderBody(["SUPER_ADMIN"], ["/sentinel", path]);

    const currentTab = await screen.findByRole("tab", { name: tabLabel, selected: true });
    await userEvent.click(currentTab);
    expectLocation(path);

    await userEvent.click(screen.getByRole("button", { name: "history back" }));
    await waitFor(() => {
      expectLocation("/sentinel");
    });
  });

  it("renders the error state (not a crash) when GET /definitions fails", async () => {
    server.use(
      http.get("*/api/v1/ontology/object-types", () => HttpResponse.json([])),
      http.get("*/api/v1/workflow-studio/definitions", () => HttpResponse.error()),
    );
    renderBody();

    expect(await screen.findByText(ko.console.workflows.errors.loadFailed)).toBeVisible();
  });

  it("shows no tabs (deny-by-omission) for a role without automate grants", async () => {
    installHandlers([]);
    renderBody(["MEMBER"]);

    // The hub loads, then finds zero viewable tabs → the honest empty chip, and
    // NOT the rule/schedule/monitor tablist.
    expect(await screen.findByText(S.labels.noAvailableTabs)).toBeVisible();
    expect(screen.queryByRole("tab", { name: S.tabs.rules })).toBeNull();
    expect(screen.queryByRole("tab", { name: S.tabs.schedules })).toBeNull();
    expect(screen.queryByRole("tab", { name: S.tabs.monitors })).toBeNull();
  });
});
