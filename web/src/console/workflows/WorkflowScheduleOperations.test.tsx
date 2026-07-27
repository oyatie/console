import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { PolicyGateProvider } from "../policy";
import { WorkflowScheduleOperations } from "./WorkflowScheduleOperations";
import type { WorkflowScheduleRun } from "./scheduleApi";

const mockAssertPasskeyStepUp = vi.hoisted(() => vi.fn());
vi.mock("../../auth/webauthn", () => ({
  assertPasskeyStepUp: mockAssertPasskeyStepUp,
}));

const schedule = {
  id: "11111111-1111-4111-8111-111111111111",
  label: "평일 KPI 스냅샷",
  cron_expr: "0 9 * * 1-5",
  timezone: "Asia/Seoul",
  definition_id: "22222222-2222-4222-8222-222222222222",
  enabled: false,
  next_run_at: null,
  last_run_at: "2026-07-23T00:00:00Z",
  last_status: "FAILED" as const,
  created_at: "2026-07-01T00:00:00Z",
  updated_at: "2026-07-23T00:00:00Z",
};

function api(): ConsoleApiClient {
  const GET = vi.fn((path: string) => {
    if (path === "/api/v1/workflow-studio/schedules")
      return { data: { items: [schedule] }, response: new Response() };
    if (path === "/api/v1/workflow-studio/definitions")
      return {
        data: { items: [{ id: schedule.definition_id, display_name: "KPI 정의" }] },
        response: new Response(),
      };
    if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
      return {
        data: {
          items: [
            {
              run_id: "33333333-3333-4333-8333-333333333333",
              status: "FAILED",
              definition_id: schedule.definition_id,
              definition_version: 3,
              started_at: "2026-07-23T00:00:00Z",
              failed_at: "2026-07-23T00:01:00Z",
            },
          ],
        },
        response: new Response(),
      };
    throw new Error(`unexpected GET ${path}`);
  });
  const POST = vi.fn(() => ({
    data: {
      cron_expr: schedule.cron_expr,
      timezone: schedule.timezone,
      fire_times: ["2026-07-27T00:00:00Z"],
    },
    response: new Response(),
  }));
  const PATCH = vi.fn(() => ({
    data: { ...schedule, enabled: true, next_run_at: "2026-07-27T00:00:00Z" },
    response: new Response(),
  }));
  return { GET, POST, PATCH } as unknown as ConsoleApiClient;
}

function renderPanel(client = api(), can: (action: string) => boolean = () => true) {
  const result = render(
    <AuthTestProvider
      session={{
        access_token: "test",
        org_id: "org-1",
        user_id: "u-1",
        roles: ["SUPER_ADMIN"],
        feature_grants: [],
      }}
      overrides={{ api: client }}
    >
      <PolicyGateProvider gate={{ can }}>
        <WorkflowScheduleOperations />
      </PolicyGateProvider>
    </AuthTestProvider>,
  );
  return { ...result, client };
}

describe("WorkflowScheduleOperations", () => {
  it("loads tenant-authorized schedule, future occurrences, and immutable schedule run history", async () => {
    renderPanel();
    expect(
      await screen.findByRole("button", { name: "평일 KPI 스냅샷 선택" }),
    ).toBeVisible();
    expect(screen.getByText("다음 실행 없음")).toBeVisible();
    expect(await screen.findByText("2026-07-27T00:00:00Z")).toBeVisible();
    expect(screen.getByText("FAILED")).toBeVisible();
  });

  it("enables a durable schedule once, then reloads authoritative server state", async () => {
    const { client } = renderPanel();
    await userEvent.click(
      await screen.findByRole("button", { name: "예약 활성화" }),
    );
    await waitFor(() => { expect(client.PATCH).toHaveBeenCalledTimes(1); });
    expect(client.PATCH).toHaveBeenCalledWith(
      "/api/v1/workflow-studio/schedules/{id}",
      {
        params: { path: { id: schedule.id } },
        body: { enabled: true },
      },
    );
    await waitFor(() => { expect(client.GET).toHaveBeenCalledTimes(5); });
    expect(screen.getByRole("button", { name: "예약 활성화" })).toBeVisible();
  });

  it("renders a denied mutation as a truthful error without changing the schedule locally", async () => {
    const client = api();
    vi.mocked(client.PATCH).mockResolvedValue({
      data: undefined,
      error: { error: { message: "denied", code: "forbidden" } },
      response: new Response(undefined, { status: 403 }),
    });
    renderPanel(client);
    await userEvent.click(
      await screen.findByRole("button", { name: "예약 활성화" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "예약 변경이 거부되었습니다",
    );
    expect(screen.getByRole("button", { name: "예약 활성화" })).toBeVisible();
  });

  it("keeps a 409 write conflict visible and reloads the authoritative schedule", async () => {
    const client = api();
    vi.mocked(client.PATCH).mockResolvedValue({
      data: undefined,
      error: { error: { message: "conflict", code: "conflict" } },
      response: new Response(undefined, { status: 409 }),
    });
    renderPanel(client);
    await userEvent.click(
      await screen.findByRole("button", { name: "예약 활성화" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "예약 상태가 변경되었습니다",
    );
    await waitFor(() => { expect(client.GET).toHaveBeenCalledTimes(5); });
    expect(screen.getByRole("button", { name: "예약 활성화" })).toBeVisible();
  });

  it("creates a durable schedule from a selected workflow definition", async () => {
    const client = api();
    vi.mocked(client.POST).mockImplementation((path) => {
      if (path === "/api/v1/workflow-studio/schedules")
        return {
          data: { ...schedule, label: "월말 KPI", enabled: true },
          response: new Response(),
        } as never;
      return {
        data: {
          cron_expr: schedule.cron_expr,
          timezone: schedule.timezone,
          fire_times: [],
        },
        response: new Response(),
      } as never;
    });
    const user = userEvent.setup();
    renderPanel(client);

    await user.type(await screen.findByLabelText("이름"), "월말 KPI");
    await user.selectOptions(
      screen.getByLabelText("연결된 워크플로 정의"),
      schedule.definition_id,
    );
    await user.click(screen.getByRole("button", { name: "예약 작업 추가" }));

    await waitFor(() =>
      { expect(client.POST).toHaveBeenCalledWith("/api/v1/workflow-studio/schedules", {
        body: {
          label: "월말 KPI",
          cron_expr: "0 9 * * 1-5",
          timezone: "Asia/Seoul",
          definition_id: schedule.definition_id,
          enabled: true,
        },
      }); },
    );
  });

  it("omits schedule creation controls for a view-only policy", async () => {
    renderPanel(api(), (action) =>
      action !== "console.automate.schedule.create" &&
      action !== "console.automate.schedule.toggle" &&
      action !== "console.automate.schedule.edit",
    );
    await screen.findByRole("button", { name: "평일 KPI 스냅샷 선택" });
    expect(screen.queryByRole("form", { name: "예약 작업 추가" })).toBeNull();
    expect(screen.queryByRole("button", { name: "예약 작업 추가" })).toBeNull();
    expect(screen.queryByRole("button", { name: "예약 편집" })).toBeNull();
  });

  it("allows update-only users to edit, save, and cancel without a create affordance", async () => {
    renderPanel(
      api(),
      (action) =>
        action !== "console.automate.schedule.create" &&
        action !== "console.automate.schedule.toggle",
    );
    await userEvent.click(await screen.findByRole("button", { name: "예약 편집" }));
    expect(screen.getByRole("form", { name: "예약 작업 편집" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "예약 작업 추가" })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "취소" }));
    expect(screen.queryByRole("form", { name: "예약 작업 추가" })).toBeNull();
  });

  it("allows toggle-only users to change lifecycle without edit or create controls", async () => {
    renderPanel(
      api(),
      (action) =>
        action !== "console.automate.schedule.create" &&
        action !== "console.automate.schedule.edit",
    );
    expect(
      await screen.findByRole("button", { name: "예약 활성화" }),
    ).toBeVisible();
    expect(screen.queryByRole("button", { name: "예약 편집" })).toBeNull();
    expect(screen.queryByRole("form", { name: "예약 작업 추가" })).toBeNull();
  });

  it("uses generated path params and a passkey proof for revision approval", async () => {
    const proof = {
      ceremony_id: "ceremony",
      credential: { id: "credential", type: "public-key" },
    };
    mockAssertPasskeyStepUp.mockResolvedValueOnce(proof);
    const GET = vi.fn((path: string) => {
      if (path === "/api/v1/workflow-studio/schedules")
        return { data: { items: [schedule] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/definitions")
        return {
          data: {
            items: [
              {
                id: schedule.definition_id,
                display_name: "KPI 정의",
                pending_version: 4,
                pending_staged_by: "another-user",
              },
            ],
          },
          response: new Response(),
        };
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
        return { data: { items: [] }, response: new Response() };
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn(() => ({
      data: { id: schedule.definition_id },
      response: new Response(),
    }));
    renderPanel({ GET, POST, PATCH: vi.fn() } as unknown as ConsoleApiClient);

    await userEvent.click(
      await screen.findByRole("button", { name: "개정 승인" }),
    );

    await waitFor(() => {
      expect(POST).toHaveBeenCalledWith(
        "/api/v1/workflow-studio/definitions/{id}/revisions/{rev}/approve",
        {
          params: { path: { id: schedule.definition_id, rev: 4 } },
          body: { step_up: proof },
        },
      );
    });
  });

  it("fails closed without sending an approval when the passkey proof is absent", async () => {
    mockAssertPasskeyStepUp.mockResolvedValueOnce(undefined);
    const GET = vi.fn((path: string) => {
      if (path === "/api/v1/workflow-studio/schedules")
        return { data: { items: [schedule] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/definitions")
        return {
          data: {
            items: [
              {
                id: schedule.definition_id,
                display_name: "KPI 정의",
                pending_version: 4,
                pending_staged_by: "another-user",
              },
            ],
          },
          response: new Response(),
        };
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
        return { data: { items: [] }, response: new Response() };
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn(() => ({
      data: { id: schedule.definition_id },
      response: new Response(),
    }));
    renderPanel({ GET, POST, PATCH: vi.fn() } as unknown as ConsoleApiClient);

    await userEvent.click(
      await screen.findByRole("button", { name: "개정 승인" }),
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "예약 정보를 처리하지 못했습니다",
    );
    expect(
      POST.mock.calls.some(([path]) => String(path).includes("/revisions/")),
    ).toBe(false);
  });

  it("blocks self-approval of a pending definition revision before a request", async () => {
    const GET = vi.fn((path: string) => {
      if (path === "/api/v1/workflow-studio/schedules")
        return { data: { items: [schedule] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/definitions")
        return {
          data: {
            items: [
              {
                id: schedule.definition_id,
                display_name: "KPI 정의",
                pending_version: 4,
                pending_staged_by: "u-1",
              },
            ],
          },
          response: new Response(),
        };
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
        return { data: { items: [] }, response: new Response() };
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn(() => ({
      data: { cron_expr: schedule.cron_expr, timezone: schedule.timezone, fire_times: [] },
      response: new Response(),
    }));
    renderPanel({ GET, POST, PATCH: vi.fn() } as unknown as ConsoleApiClient);

    await userEvent.click(await screen.findByRole("button", { name: "개정 승인" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "자신이 올린 개정은 승인할 수 없습니다.",
    );
    expect(
      POST.mock.calls.some(([path]) => String(path).includes("/revisions/")),
    ).toBe(false);
  });
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((nextResolve) => {
    resolve = nextResolve;
  });
  return { promise, resolve };
}

describe("WorkflowScheduleOperations scope fences", () => {
  it("does not post a revision after an A passkey step-up resolves in B", async () => {
    const stepUp = deferred<{
      ceremony_id: string;
      credential: { id: string; type: string };
    }>();
    mockAssertPasskeyStepUp.mockReturnValueOnce(stepUp.promise);
    const GET = vi.fn((path: string) => {
      if (path === "/api/v1/workflow-studio/schedules")
        return { data: { items: [schedule] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/definitions")
        return {
          data: {
            items: [
              {
                id: schedule.definition_id,
                display_name: "KPI 정의",
                pending_version: 4,
                pending_staged_by: "another-user",
              },
            ],
          },
          response: new Response(),
        };
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
        return { data: { items: [] }, response: new Response() };
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn(() => ({
      data: { cron_expr: schedule.cron_expr, timezone: schedule.timezone, fire_times: [] },
      response: new Response(),
    }));
    const client = { GET, POST, PATCH: vi.fn() } as unknown as ConsoleApiClient;
    const sessionA = { access_token: "token-a", org_id: "org-a", user_id: "a", roles: ["SUPER_ADMIN"], feature_grants: [] };
    const sessionB = { ...sessionA, access_token: "token-b", org_id: "org-b", user_id: "b" };
    const view = render(
      <AuthTestProvider session={sessionA} overrides={{ api: client }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );
    await userEvent.click(await screen.findByRole("button", { name: "개정 승인" }));
    view.rerender(
      <AuthTestProvider session={sessionB} overrides={{ api: client }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );
    stepUp.resolve({ ceremony_id: "ceremony", credential: { id: "credential", type: "public-key" } });
    await Promise.resolve();
    await Promise.resolve();

    expect(
      POST.mock.calls.some(([path]) => String(path).includes("/revisions/")),
    ).toBe(false);
  });

  it("does not reload or mutate B after an A toggle resolves late", async () => {
    const write = deferred<{ data: typeof schedule; response: Response }>();
    let scheduleLists = 0;
    const GET = vi.fn((path: string) => {
      if (path === "/api/v1/workflow-studio/schedules") {
        scheduleLists += 1;
        return {
          data: {
            items: [
              {
                ...schedule,
                label: scheduleLists === 1 ? "A 예약" : "B 예약",
              },
            ],
          },
          response: new Response(),
        };
      }
      if (path === "/api/v1/workflow-studio/definitions")
        return { data: { items: [] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
        return { data: { items: [] }, response: new Response() };
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn(() => ({
      data: { cron_expr: schedule.cron_expr, timezone: schedule.timezone, fire_times: [] },
      response: new Response(),
    }));
    const client = { GET, POST, PATCH: vi.fn(() => write.promise) } as unknown as ConsoleApiClient;
    const sessionA = { access_token: "token-a", org_id: "org-a", user_id: "a", roles: ["SUPER_ADMIN"], feature_grants: [] };
    const sessionB = { ...sessionA, access_token: "token-b", org_id: "org-b", user_id: "b" };
    const view = render(
      <AuthTestProvider session={sessionA} overrides={{ api: client }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );
    await userEvent.click(await screen.findByRole("button", { name: "예약 활성화" }));
    view.rerender(
      <AuthTestProvider session={sessionB} overrides={{ api: client }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );
    expect(await screen.findByText("B 예약")).toBeVisible();
    write.resolve({ data: { ...schedule, enabled: true, label: "A 예약" }, response: new Response() });
    await Promise.resolve();
    await Promise.resolve();

    expect(scheduleLists).toBe(2);
    expect(screen.queryByText("A 예약")).toBeNull();
    expect(screen.getByRole("button", { name: "예약 활성화" })).toBeVisible();
  });

  it("clears an A-session edit form before B can inherit it", async () => {
    const client = api();
    const sessionA = { access_token: "token-a", org_id: "org-a", user_id: "a", roles: ["SUPER_ADMIN"], feature_grants: [] };
    const sessionB = { ...sessionA, access_token: "token-b", org_id: "org-b", user_id: "b" };
    const view = render(
      <AuthTestProvider session={sessionA} overrides={{ api: client }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );
    await userEvent.click(await screen.findByRole("button", { name: "예약 편집" }));
    expect(screen.getByRole("form", { name: "예약 작업 편집" })).toBeVisible();
    expect(screen.getByLabelText("이름")).toHaveValue("평일 KPI 스냅샷");

    view.rerender(
      <AuthTestProvider session={sessionB} overrides={{ api: client }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );
    await waitFor(() =>
      { expect(screen.getByRole("form", { name: "예약 작업 추가" })).toBeVisible(); },
    );
    expect(screen.getByLabelText("이름")).toHaveValue("");
    expect(screen.getByLabelText("연결된 워크플로 정의")).toHaveValue("");
  });

  it("does not start an already-unmounted A bootstrap after an immediate A→B switch", async () => {
    const clientA = api();
    const clientB = api();
    const sessionA = { access_token: "token-a", org_id: "org-a", user_id: "a", roles: ["SUPER_ADMIN"], feature_grants: [] };
    const sessionB = { ...sessionA, access_token: "token-b", org_id: "org-b", user_id: "b" };
    const view = render(
      <AuthTestProvider session={sessionA} overrides={{ api: clientA }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );

    // Rerender before React drains the startup microtask scheduled for A.
    view.rerender(
      <AuthTestProvider session={sessionB} overrides={{ api: clientB }}>
        <PolicyGateProvider gate={{ can: () => true }}><WorkflowScheduleOperations /></PolicyGateProvider>
      </AuthTestProvider>,
    );

    await waitFor(() => { expect(clientB.GET).toHaveBeenCalledTimes(2); });
    expect(clientA.GET).not.toHaveBeenCalled();
    expect(await screen.findByRole("button", { name: "평일 KPI 스냅샷 선택" })).toBeVisible();
  });

  it("clears preview and history together when the selected schedule detail request fails", async () => {
    const GET = vi.fn((path: string) => {
      if (path === "/api/v1/workflow-studio/schedules") return { data: { items: [schedule] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/definitions") return { data: { items: [] }, response: new Response() };
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs") return { data: undefined, error: { error: { message: "conflict", code: "conflict" } }, response: new Response(undefined, { status: 409 }) };
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn(() => ({ data: { cron_expr: schedule.cron_expr, timezone: schedule.timezone, fire_times: ["2026-07-30T00:00:00Z"] }, response: new Response() }));
    const client = { GET, POST, PATCH: vi.fn() } as unknown as ConsoleApiClient;
    renderPanel(client);

    expect(await screen.findByRole("alert")).toHaveTextContent("예약 상태가 변경되었습니다");
    expect(screen.queryByText("2026-07-30T00:00:00Z")).toBeNull();
    expect(screen.getByText("예정된 실행이 없습니다.")).toBeVisible();
    expect(screen.getByText("실행 기록 없음")).toBeVisible();
  });

  it("rejects late preview and history results after selecting another schedule", async () => {
    const scheduleB = {
      ...schedule,
      id: "44444444-4444-4444-8444-444444444444",
      label: "월말 정산",
      cron_expr: "0 18 28-31 * *",
    };
    const firstPreview = deferred<{ data: { cron_expr: string; timezone: string; fire_times: string[] }; response: Response }>();
    const firstRuns = deferred<{ data: { items: WorkflowScheduleRun[] }; response: Response }>();
    const secondPreview = deferred<{ data: { cron_expr: string; timezone: string; fire_times: string[] }; response: Response }>();
    const secondRuns = deferred<{ data: { items: WorkflowScheduleRun[] }; response: Response }>();
    const GET = vi.fn((path: string, options?: { params?: { path?: { id?: string } } }) => {
      if (path === "/api/v1/workflow-studio/schedules") return Promise.resolve({ data: { items: [schedule, scheduleB] }, response: new Response() });
      if (path === "/api/v1/workflow-studio/definitions") return Promise.resolve({ data: { items: [] }, response: new Response() });
      if (path === "/api/v1/workflow-studio/schedules/{id}/runs")
        return options?.params?.path?.id === schedule.id ? firstRuns.promise : secondRuns.promise;
      throw new Error(`unexpected GET ${path}`);
    });
    const POST = vi.fn((_: string, options: { body: { cron_expr: string } }) =>
      options.body.cron_expr === schedule.cron_expr ? firstPreview.promise : secondPreview.promise,
    );
    const client = { GET, POST, PATCH: vi.fn() } as unknown as ConsoleApiClient;
    renderPanel(client);

    await screen.findByRole("button", { name: "월말 정산 선택" });
    await waitFor(() => { expect(POST).toHaveBeenCalledTimes(1); });
    const firstPreviewSignal = POST.mock.calls[0]?.[1]?.signal as AbortSignal;
    const firstHistorySignal = GET.mock.calls.find(
      ([path, options]) =>
        path === "/api/v1/workflow-studio/schedules/{id}/runs" &&
        options?.params?.path?.id === schedule.id,
    )?.[1]?.signal as AbortSignal;
    await userEvent.click(screen.getByRole("button", { name: "월말 정산 선택" }));
    await waitFor(() => { expect(POST).toHaveBeenCalledTimes(2); });
    expect(firstPreviewSignal.aborted).toBe(true);
    expect(firstHistorySignal.aborted).toBe(true);

    secondPreview.resolve({ data: { cron_expr: scheduleB.cron_expr, timezone: scheduleB.timezone, fire_times: ["2026-07-31T09:00:00Z"] }, response: new Response() });
    secondRuns.resolve({ data: { items: [] }, response: new Response() });
    expect(await screen.findByText("2026-07-31T09:00:00Z")).toBeVisible();

    firstPreview.resolve({ data: { cron_expr: schedule.cron_expr, timezone: schedule.timezone, fire_times: ["2026-07-27T00:00:00Z"] }, response: new Response() });
    firstRuns.resolve({ data: { items: [{ run_id: "late-a", status: "FAILED", definition_id: schedule.definition_id, definition_version: 3, started_at: "2026-07-27T00:00:00Z", failed_at: "2026-07-27T00:01:00Z" }] }, response: new Response() });
    await Promise.resolve();

    expect(screen.queryByText("2026-07-27T00:00:00Z")).toBeNull();
    expect(screen.queryByText(/late-a/)).toBeNull();
  });
});
