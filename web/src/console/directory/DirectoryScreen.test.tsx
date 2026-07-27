import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { directoryStrings as text } from "../../i18n/directory";
import { DirectoryScreen } from "./DirectoryScreen";
import { deriveDirectoryCapabilities, type DirectoryCapabilities } from "./directoryCapabilities";

const BRANCH = "bbbbbbbb-0000-0000-0000-000000000001";
const ACTOR = "aaaaaaaa-0000-0000-0000-000000000001";
const PEER = "aaaaaaaa-0000-0000-0000-000000000002";
const EMP = "eeeeeeee-0000-0000-0000-000000000001";
const THREAD = "cccccccc-0000-0000-0000-000000000001";

const peer = { id: PEER, display_name: "김성호", team: "정비1팀" } as const;
const self = { id: ACTOR, display_name: "박지민", team: null } as const;

const employee = {
  id: EMP,
  company: "KNL",
  name: "이하나",
  employee_number: "10021",
  org_unit: "경영지원",
  worksite_name: "창원",
  worksite: null,
  job: "인사기획",
  position: "대리",
  hire_date: "2024-03-02",
  exit_date: null,
  status: "ACTIVE",
  leave_accrued: null,
  leave_used: null,
  leave_remaining: null,
  home_branch_id: BRANCH,
  home_branch_name: "창원지점",
  home_branch_review_required: false,
  identity_resolution_strategy: "employee_number",
  identity_resolution_confidence: "high",
  identity_review_required: false,
  identity_name_only_merge: false,
  created_at: "2026-07-01T00:00:00Z",
  updated_at: "2026-07-01T00:00:00Z",
} as const;

const lifecycleEvent = {
  id: "ffffffff-0000-0000-0000-000000000001",
  employee_id: EMP,
  event_type: "TRANSFER",
  from_status: "ACTIVE",
  to_status: "ACTIVE",
  from_company: "KNL",
  to_company: "KNL",
  from_org_unit: "영업",
  to_org_unit: "경영지원",
  from_position: null,
  to_position: null,
  effective_date: "2025-01-01",
  comment: "조직 개편",
  signoffs: {
    privacy_notice_ack: true,
    korean_labor_law_ack: true,
    payroll_cutoff_ack: true,
    retirement_settlement_ack: true,
  },
  created_by: ACTOR,
  created_at: "2025-01-01T00:00:00Z",
} as const;

const thread = { id: THREAD, kind: "dm" } as const;

const MEMBER_CAPS: DirectoryCapabilities = {
  canRead: true,
  canViewPerson: true,
  canReadHrDirectory: false,
  canMessage: true,
};
const FULL_CAPS: DirectoryCapabilities = {
  canRead: true,
  canViewPerson: true,
  canReadHrDirectory: true,
  canMessage: true,
};
const NO_CAPS: DirectoryCapabilities = {
  canRead: false,
  canViewPerson: false,
  canReadHrDirectory: false,
  canMessage: false,
};

function ok<T>(data: T) {
  return Promise.resolve({ response: new Response(), data });
}
function fail(status: number) {
  return Promise.resolve({ response: new Response(null, { status }), error: { message: "failed" } });
}

type Handler = (options: { params?: { path?: Record<string, string>; query?: Record<string, unknown> } }) => Promise<unknown>;

function makeApi(overrides: Partial<Record<string, Handler>> = {}) {
  const handlers: Partial<Record<string, Handler>> = {
    "/api/messenger/members": () => ok({ items: [peer, self] }),
    "/api/messenger/members/{userId}": (options) =>
      options.params?.path?.userId === PEER ? ok(peer) : ok(self),
    "/api/v1/employees": () => ok({ items: [employee], total: 1, limit: 100, offset: 0 }),
    "/api/v1/employees/{id}/lifecycle-events": () => ok({ items: [lifecycleEvent] }),
    ...overrides,
  };
  const GET = vi.fn((path: string, options: never) => {
    const handler = handlers[path];
    return handler ? handler(options) : fail(500);
  });
  const POST = vi.fn(() => Promise.resolve({ response: new Response(null, { status: 201 }), data: thread }));
  return { GET, POST } as unknown as ConsoleApiClient;
}

function renderScreen(api: ConsoleApiClient, capabilities: DirectoryCapabilities, extra: Partial<Parameters<typeof DirectoryScreen>[0]> = {}) {
  return render(
    <DirectoryScreen
      api={api}
      branchId={BRANCH}
      actorId={ACTOR}
      capabilities={capabilities}
      sessionKey="s1"
      {...extra}
    />,
  );
}

describe("deriveDirectoryCapabilities", () => {
  it("maps the messenger tier and HR feature grant onto disjoint capabilities", () => {
    expect(deriveDirectoryCapabilities({ roles: ["MEMBER"], featureGrants: [] }, BRANCH)).toEqual({
      canRead: true, canViewPerson: true, canReadHrDirectory: false, canMessage: true,
    });
    expect(deriveDirectoryCapabilities({ roles: ["MEMBER"], featureGrants: ["employee_directory_read"] }, BRANCH).canReadHrDirectory).toBe(true);
    expect(deriveDirectoryCapabilities({ roles: ["ADMIN"], featureGrants: [] }, undefined)).toEqual({
      canRead: true, canViewPerson: false, canReadHrDirectory: true, canMessage: false,
    });
    expect(deriveDirectoryCapabilities({ roles: [], featureGrants: [] }, BRANCH)).toEqual({
      canRead: false, canViewPerson: false, canReadHrDirectory: false, canMessage: false,
    });
  });
});

describe("DirectoryScreen", () => {
  it("keeps every visible literal in the module copy resource", () => {
    expect([text.title, text.statMembers, text.newConversation, text.viewLogged]).toEqual([
      "주소록", "구성원", "새 대화", "열람 기록됨",
    ]);
  });

  it("renders denied before any fetch when the caller has no capability", () => {
    const api = makeApi();
    renderScreen(api, NO_CAPS);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(vi.mocked(api.GET)).not.toHaveBeenCalled();
  });

  it("opens a coworker card only through the read-audited member endpoint", async () => {
    const api = makeApi();
    renderScreen(api, MEMBER_CAPS);
    fireEvent.click(await screen.findByRole("option", { name: /김성호/ }));
    await waitFor(() => {
      expect(vi.mocked(api.GET)).toHaveBeenCalledWith(
        "/api/messenger/members/{userId}",
        expect.objectContaining({ params: { path: { userId: PEER }, query: { branch_id: BRANCH } } }),
      );
    });
    expect(await screen.findByText(text.viewLogged)).toBeVisible();
    expect(screen.getByRole("button", { name: text.message })).toBeEnabled();
  });

  it("marks a self view without an audit chip or a message CTA", async () => {
    const api = makeApi();
    renderScreen(api, MEMBER_CAPS);
    fireEvent.click(await screen.findByRole("option", { name: /박지민/ }));
    expect(await screen.findByText(text.self)).toBeVisible();
    expect(screen.queryByText(text.viewLogged)).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: text.message })).not.toBeInTheDocument();
  });

  it("shows the no-leak blocked state for a 404 person view", async () => {
    const api = makeApi({ "/api/messenger/members/{userId}": () => fail(404) });
    renderScreen(api, MEMBER_CAPS);
    fireEvent.click(await screen.findByRole("option", { name: /김성호/ }));
    expect(await screen.findByText(text.detailBlocked)).toBeVisible();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("starts a DM through the real thread endpoint and hands off the thread id", async () => {
    const api = makeApi();
    const onOpenThread = vi.fn();
    renderScreen(api, MEMBER_CAPS, { onOpenThread });
    fireEvent.click(await screen.findByRole("option", { name: /김성호/ }));
    fireEvent.click(await screen.findByRole("button", { name: text.message }));
    await waitFor(() => { expect(onOpenThread).toHaveBeenCalledWith(THREAD); });
    expect(vi.mocked(api.POST)).toHaveBeenCalledWith(
      "/api/messenger/threads",
      expect.objectContaining({ body: { branch_id: BRANCH, kind: "dm", member_ids: [PEER] } }),
    );
  });

  it("surfaces a DM failure as a retryable alert", async () => {
    const api = makeApi();
    vi.mocked(api.POST).mockResolvedValueOnce({ response: new Response(null, { status: 409 }), error: { message: "conflict" } });
    const onOpenThread = vi.fn();
    renderScreen(api, MEMBER_CAPS, { onOpenThread });
    fireEvent.click(await screen.findByRole("option", { name: /김성호/ }));
    fireEvent.click(await screen.findByRole("button", { name: text.message }));
    expect(await screen.findByText(text.messageError)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: text.retry }));
    await waitFor(() => { expect(onOpenThread).toHaveBeenCalledWith(THREAD); });
  });

  it("recovers a failed roster load through retry", async () => {
    let calls = 0;
    const api = makeApi({
      "/api/messenger/members": () => (calls++ === 0 ? fail(500) : ok({ items: [peer, self] })),
    });
    renderScreen(api, MEMBER_CAPS);
    expect(await screen.findByText(text.membersLoadError)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await screen.findByRole("option", { name: /김성호/ })).toBeVisible();
  });

  it("renders a denied roster as denial, not as an error", async () => {
    const api = makeApi({ "/api/messenger/members": () => fail(403) });
    renderScreen(api, MEMBER_CAPS);
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows the filter-clearing empty state when a search matches nothing", async () => {
    const api = makeApi();
    renderScreen(api, MEMBER_CAPS);
    await screen.findByRole("option", { name: /김성호/ });
    fireEvent.change(screen.getByPlaceholderText(text.search), { target: { value: "없는사람" } });
    expect(await screen.findByText(text.empty)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: text.clearFilter }));
    expect(await screen.findByRole("option", { name: /김성호/ })).toBeVisible();
  });

  it("moves the selection with arrow and j/k keys", async () => {
    const api = makeApi();
    renderScreen(api, MEMBER_CAPS);
    const first = await screen.findByRole("option", { name: /김성호/ });
    fireEvent.click(first);
    fireEvent.keyDown(first, { key: "ArrowDown" });
    await waitFor(() => {
      expect(screen.getByRole("option", { name: /박지민/ })).toHaveAttribute("aria-selected", "true");
    });
    fireEvent.keyDown(screen.getByRole("option", { name: /박지민/ }), { key: "k" });
    await waitFor(() => {
      expect(screen.getByRole("option", { name: /김성호/ })).toHaveAttribute("aria-selected", "true");
    });
  });

  it("fences a stale person-card response behind a newer selection", async () => {
    let resolveSlow!: (value: unknown) => void;
    const slow = new Promise((resolve) => { resolveSlow = resolve; });
    const api = makeApi({
      "/api/messenger/members/{userId}": (options) =>
        options.params?.path?.userId === PEER ? slow : ok(self),
    });
    renderScreen(api, MEMBER_CAPS);
    fireEvent.click(await screen.findByRole("option", { name: /김성호/ }));
    fireEvent.click(screen.getByRole("option", { name: /박지민/ }));
    expect(await screen.findByText(text.self)).toBeVisible();
    resolveSlow({ response: new Response(), data: peer });
    await waitFor(() => {
      expect(screen.queryByText(text.viewLogged)).not.toBeInTheDocument();
    });
  });

  it("keeps HR-only surfaces absent for the member tier", async () => {
    const api = makeApi();
    renderScreen(api, MEMBER_CAPS);
    await screen.findByRole("option", { name: /김성호/ });
    expect(screen.queryByText(text.statEmployees)).not.toBeInTheDocument();
    expect(vi.mocked(api.GET)).not.toHaveBeenCalledWith("/api/v1/employees", expect.anything());
  });

  it("exposes the HR register with history and a server-reconciled company drill", async () => {
    const api = makeApi();
    renderScreen(api, FULL_CAPS);
    fireEvent.click(await screen.findByRole("button", { name: /임직원/ }));
    fireEvent.click(await screen.findByRole("option", { name: /이하나/ }));
    expect(await screen.findByText("조직 개편", { exact: false })).toBeVisible();
    expect(screen.getByText(text.event.TRANSFER)).toBeVisible();
    expect(screen.getByText(employee.hire_date)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: /KNL/ }));
    await waitFor(() => {
      expect(vi.mocked(api.GET)).toHaveBeenCalledWith(
        "/api/v1/employees",
        expect.objectContaining({ params: { query: expect.objectContaining({ company: "KNL" }) } }),
      );
    });
  });

  it("drops the HR enhancement when the server denies the register", async () => {
    const api = makeApi({ "/api/v1/employees": () => fail(403) });
    renderScreen(api, FULL_CAPS);
    await screen.findByRole("option", { name: /김성호/ });
    await waitFor(() => {
      expect(screen.queryByText(text.statEmployees)).not.toBeInTheDocument();
    });
  });

  it("restores a selection key and reports selection changes for URL persistence", async () => {
    const api = makeApi();
    const onPersonKeyChange = vi.fn();
    renderScreen(api, MEMBER_CAPS, { initialPersonKey: `m:${PEER}`, onPersonKeyChange });
    expect(await screen.findByText(text.viewLogged)).toBeVisible();
    expect(onPersonKeyChange).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("option", { name: /박지민/ }));
    await waitFor(() => {
      expect(onPersonKeyChange).toHaveBeenCalledWith(`m:${ACTOR}`);
    });
  });

  it("restores an employee selection through a truthful loading state into the card", async () => {
    let resolveEmployees!: (value: unknown) => void;
    const page = new Promise((resolve) => { resolveEmployees = resolve; });
    const api = makeApi({ "/api/v1/employees": () => page });
    renderScreen(api, FULL_CAPS, { initialPersonKey: `e:${EMP}` });
    expect(await screen.findByText(text.detailLoading)).toBeVisible();
    resolveEmployees({ response: new Response(), data: { items: [employee], total: 1, limit: 100, offset: 0 } });
    expect(await screen.findByText(employee.hire_date)).toBeVisible();
    expect(await screen.findByText(text.event.TRANSFER)).toBeVisible();
  });

  it("clears a restored employee selection when the server denies the register", async () => {
    const api = makeApi({ "/api/v1/employees": () => fail(403) });
    const onPersonKeyChange = vi.fn();
    renderScreen(api, FULL_CAPS, { initialPersonKey: `e:${EMP}`, onPersonKeyChange });
    expect(await screen.findByText(text.detailEmpty)).toBeVisible();
    await waitFor(() => { expect(onPersonKeyChange).toHaveBeenCalledWith(undefined); });
  });

  it("surfaces a register failure on the selected employee card and recovers through retry", async () => {
    let calls = 0;
    const api = makeApi({
      "/api/v1/employees": () =>
        (calls++ === 0 ? fail(500) : ok({ items: [employee], total: 1, limit: 100, offset: 0 })),
    });
    renderScreen(api, FULL_CAPS, { initialPersonKey: `e:${EMP}` });
    expect(await screen.findByText(text.detailError)).toBeVisible();
    const retries = screen.getAllByRole("button", { name: text.retry });
    fireEvent.click(retries[retries.length - 1]);
    expect(await screen.findByText(employee.hire_date)).toBeVisible();
  });

  it("attaches a draggable reference token to every row", async () => {
    const api = makeApi();
    renderScreen(api, FULL_CAPS);
    const row = await screen.findByRole("option", { name: /김성호/ });
    const setData = vi.fn();
    fireEvent.dragStart(row, { dataTransfer: { setData } });
    expect(setData).toHaveBeenCalledWith("text/plain", "@김성호");
  });
});
