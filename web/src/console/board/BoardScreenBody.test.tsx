import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { ko } from "../../i18n/ko";
import { boardStrings as text } from "../../i18n/board";
import type { BoardNotice } from "./boardApi";
import { BoardScreenBody } from "./BoardScreenBody";

const { useAuth } = vi.hoisted(() => ({ useAuth: vi.fn() }));
vi.mock("../../context/auth", () => ({ useAuth }));

type Call = { data?: unknown; error?: unknown; response: Response };

function ok(data: unknown, status = 200): Call {
  return { data, response: new Response(null, { status }) };
}

function fail(status: number, code: string, message: string): Call {
  return { error: { error: { code, message } }, response: new Response(null, { status }) };
}

function notice(over: Partial<BoardNotice> = {}): BoardNotice {
  return {
    id: "n1",
    code: "NT-0707",
    author_user_id: "author-1",
    title: "취업규칙 개정 통지",
    body: "근로기준법 §94 개별 수령확인 대상입니다.",
    status: "published",
    published_at: "2026-07-22T09:00:00+09:00",
    created_at: "2026-07-20T09:00:00+09:00",
    category: "legal",
    audience_scope: "org",
    audience_branches: [],
    my_receipt: null,
    progress: null,
    ...over,
  };
}

const draftRow = notice({
  id: "n2",
  code: null,
  title: "7월 안전교육 안내",
  body: "교육 대상 안내 초안",
  status: "draft",
  published_at: null,
  category: "training",
  audience_scope: "branches",
  audience_branches: [{ id: "b1", name: "안산공장" }],
});

/** Authz endpoint served at the real fetch boundary (jwt floor stays empty). */
function stubAuthzFetch(features: string[]) {
  vi.stubGlobal("fetch", vi.fn(() => Promise.resolve(new Response(JSON.stringify({
    roles: [],
    branch_scope: { kind: "all" },
    capabilities: features.map((feature) => ({
      feature,
      permission: "allow",
      branch_scope: { kind: "all" },
    })),
  }), { status: 200, headers: { "Content-Type": "application/json" } }))));
}

interface Api {
  GET: ReturnType<typeof vi.fn>;
  POST: ReturnType<typeof vi.fn>;
  PATCH: ReturnType<typeof vi.fn>;
}

function makeApi(routes: Partial<Record<string, Call | Call[] | (() => Promise<Call>)>>): Api {
  const queues = new Map<string, Call[]>();
  const call = (path: string): Promise<Call> => {
    const route = routes[path];
    if (route === undefined) return Promise.resolve(fail(404, "not_found", path));
    if (typeof route === "function") return route();
    if (Array.isArray(route)) {
      const queue = queues.get(path) ?? [...route];
      queues.set(path, queue);
      return Promise.resolve(queue.length > 1 ? (queue.shift() as Call) : queue[0]);
    }
    return Promise.resolve(route);
  };
  return {
    GET: vi.fn((path: string) => call(path)),
    POST: vi.fn((path: string) => call(path)),
    PATCH: vi.fn((path: string) => call(path)),
  };
}

function renderBody(api: Api, features: string[]) {
  stubAuthzFetch(features);
  useAuth.mockReturnValue({
    api,
    session: {
      org_id: "org-a",
      user_id: "user-a",
      client_session_incarnation: "inc-1",
      access_token: "token-a",
      roles: [],
      feature_grants: features,
    },
  });
  return render(<BoardScreenBody />);
}

beforeEach(() => {
  window.sessionStorage.clear();
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

describe("BoardScreenBody", () => {
  it("renders the manager list: codes, status chips, stats incl. drafts, and the compose affordance", async () => {
    const api = makeApi({
      "/api/v1/notices": ok([
        notice({ progress: { total: 1284, acknowledged: 1192 }, my_receipt: { acknowledged_at: "2026-07-22T10:00:00+09:00" } }),
        draftRow,
      ]),
    });
    renderBody(api, ["notice_manage"]);
    expect(await screen.findByText("NT-0707")).toBeVisible();
    expect(screen.getByText(text.status.ackInProgress)).toBeVisible();
    const grid = screen.getByRole("grid", { name: `${text.title} ${ko.console.module.list.label}` });
    expect(within(grid).getByText(text.status.draft)).toBeVisible();
    const statbar = document.querySelector('[data-fidelity="module-statbar"]');
    expect(within(statbar as HTMLElement).getByText(text.stats.drafts)).toBeVisible();
    expect(screen.getByTestId("module-primary-action")).toHaveTextContent(text.compose);
  });

  it("deny-by-omission for members: no compose, no draft stat, no manager actions — only the pending ack", async () => {
    const api = makeApi({
      "/api/v1/notices": ok([notice({ my_receipt: { acknowledged_at: null } })]),
    });
    renderBody(api, []);
    await userEvent.click(await screen.findByText("취업규칙 개정 통지"));
    const detail = await screen.findByRole("complementary", { name: "취업규칙 개정 통지" });
    expect(within(detail).getByRole("button", { name: text.actions.ack })).toBeVisible();
    expect(within(detail).queryByRole("button", { name: text.actions.receipts })).toBeNull();
    expect(within(detail).queryByRole("button", { name: text.actions.publish })).toBeNull();
    expect(screen.queryByTestId("module-primary-action")).toBeNull();
    expect(screen.queryByText(text.stats.drafts)).toBeNull();
  });

  it("shows the truthful empty state", async () => {
    const api = makeApi({ "/api/v1/notices": ok([]) });
    renderBody(api, []);
    expect(await screen.findByText(ko.console.module.list.empty)).toBeVisible();
  });

  it("surfaces a load error and recovers through retry", async () => {
    const api = makeApi({
      "/api/v1/notices": [fail(500, "internal", "server down"), ok([notice()])],
    });
    renderBody(api, []);
    expect(await screen.findByText(ko.console.module.list.error)).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: ko.console.module.list.retry }));
    expect(await screen.findByText("NT-0707")).toBeVisible();
  });

  it("navigates the list with J/K/Enter", async () => {
    const api = makeApi({
      "/api/v1/notices": ok([notice(), notice({ id: "n9", code: "NT-0701", title: "정기인사 명령" })]),
    });
    renderBody(api, []);
    await screen.findByText("NT-0707");
    const grid = screen.getByRole("grid", { name: `${text.title} ${ko.console.module.list.label}` });
    grid.focus();
    fireEvent.keyDown(grid, { key: "j" });
    fireEvent.keyDown(grid, { key: "j" });
    fireEvent.keyDown(grid, { key: "Enter" });
    expect(await screen.findByRole("complementary", { name: "정기인사 명령" })).toBeVisible();
  });

  it("acknowledges a pending receipt and reconciles from the reloaded backend list", async () => {
    const acked = notice({ my_receipt: { acknowledged_at: "2026-07-24T09:00:00+09:00" } });
    const api = makeApi({
      "/api/v1/notices": [ok([notice({ my_receipt: { acknowledged_at: null } })]), ok([acked])],
      "/api/v1/notices/{id}/ack": ok(undefined, 204),
    });
    renderBody(api, []);
    await userEvent.click(await screen.findByText("취업규칙 개정 통지"));
    await userEvent.click(await screen.findByRole("button", { name: text.actions.ack }));
    expect(await screen.findByRole("status")).toHaveTextContent(text.toasts.acked);
    expect(api.POST).toHaveBeenCalledWith("/api/v1/notices/{id}/ack", expect.objectContaining({
      params: { path: { id: "n1" } },
    }));
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: text.actions.ack })).toBeNull();
    });
    expect(screen.getByText(text.detail.myAck)).toBeVisible();
  });

  it("surfaces a publish conflict from the server envelope", async () => {
    const api = makeApi({
      "/api/v1/notices": ok([draftRow]),
      "/api/v1/notices/{id}/publish": fail(409, "conflict", "이미 게시된 공지입니다"),
    });
    renderBody(api, ["notice_manage"]);
    await userEvent.click(await screen.findByText("7월 안전교육 안내"));
    await userEvent.click(await screen.findByRole("button", { name: text.actions.publish }));
    expect(await screen.findByRole("status")).toHaveTextContent("이미 게시된 공지입니다");
  });

  it("creates a scoped draft through the composer with client-side audience validation", async () => {
    const created = notice({ id: "n3", code: null, status: "draft", title: "새 공지" });
    const api = makeApi({
      "/api/v1/notices": [ok([]), ok([created])],
      "/api/v1/branches": ok([
        { id: "b1", region_id: "r1", name: "안산공장", deactivated_at: null, created_at: "2026-01-01T00:00:00Z" },
        { id: "b2", region_id: "r1", name: "폐쇄지점", deactivated_at: "2026-06-01T00:00:00Z", created_at: "2026-01-01T00:00:00Z" },
      ]),
    });
    renderBody(api, ["notice_manage"]);
    await userEvent.click(await screen.findByTestId("module-primary-action"));
    const dialog = await screen.findByRole("dialog", { name: text.composer.createTitle });
    await userEvent.type(within(dialog).getByLabelText(text.composer.titleLabel), "새 공지");
    await userEvent.type(within(dialog).getByLabelText(text.composer.bodyLabel), "본문입니다");
    await userEvent.click(within(dialog).getByRole("radio", { name: text.composer.scopeBranches }));
    expect(await within(dialog).findByRole("checkbox", { name: "안산공장" })).toBeVisible();
    expect(within(dialog).queryByRole("checkbox", { name: "폐쇄지점" })).toBeNull();

    // Empty branch selection is rejected client-side before any request.
    await userEvent.click(within(dialog).getByRole("button", { name: text.composer.save }));
    expect(await within(dialog).findByRole("alert")).toHaveTextContent(text.composer.branchesRequired);
    expect(api.POST).not.toHaveBeenCalled();

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "안산공장" }));
    await userEvent.click(within(dialog).getByRole("button", { name: text.composer.save }));
    await waitFor(() => {
      expect(api.POST).toHaveBeenCalledWith("/api/v1/notices", expect.objectContaining({
        body: {
          title: "새 공지",
          body: "본문입니다",
          category: "general",
          audience: { scope: "branches", branch_ids: ["b1"] },
        },
      }));
    });
    expect(await screen.findByRole("status")).toHaveTextContent(text.toasts.draftSaved);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("keeps unsaved composer fields across a remount (refresh survival)", async () => {
    const api = makeApi({ "/api/v1/notices": ok([]), "/api/v1/branches": ok([]) });
    const view = renderBody(api, ["notice_manage"]);
    await userEvent.click(await screen.findByTestId("module-primary-action"));
    await userEvent.type(screen.getByLabelText(text.composer.titleLabel), "임시 제목");
    view.unmount();

    const api2 = makeApi({ "/api/v1/notices": ok([]), "/api/v1/branches": ok([]) });
    renderBody(api2, ["notice_manage"]);
    await userEvent.click(await screen.findByTestId("module-primary-action"));
    expect(screen.getByLabelText(text.composer.titleLabel)).toHaveValue("임시 제목");
  });

  it("drills receipts with progress bar, filter, and paging query", async () => {
    const managerRow = notice({ progress: { total: 3, acknowledged: 1 } });
    const api = makeApi({
      "/api/v1/notices": ok([managerRow]),
      "/api/v1/notices/{id}/progress": ok({ total: 3, acknowledged: 1 }),
      "/api/v1/notices/{id}/receipts": ok({
        items: [
          { recipient_user_id: "u1", display_name: "김성아", acknowledged_at: "2026-07-23T10:00:00+09:00" },
          { recipient_user_id: "u2", display_name: "이종호", acknowledged_at: null },
        ],
        total: 2,
      }),
    });
    renderBody(api, ["notice_manage"]);
    await userEvent.click(await screen.findByText("취업규칙 개정 통지"));
    await userEvent.click(await screen.findByRole("button", { name: text.actions.receipts }));
    const dialog = await screen.findByRole("dialog");
    expect(await within(dialog).findByText("김성아")).toBeVisible();
    expect(within(dialog).getByText("이종호")).toBeVisible();
    expect(within(dialog).getByText("1 / 3 (33%)")).toBeVisible();
    expect(api.GET).toHaveBeenCalledWith("/api/v1/notices/{id}/receipts", expect.objectContaining({
      params: { path: { id: "n1" }, query: { limit: 50, offset: 0 } },
    }));
    await userEvent.click(within(dialog).getByRole("button", { name: text.receipts.filterPending }));
    await waitFor(() => {
      expect(api.GET).toHaveBeenCalledWith("/api/v1/notices/{id}/receipts", expect.objectContaining({
        params: { path: { id: "n1" }, query: { acknowledged: false, limit: 50, offset: 0 } },
      }));
    });
  });

  it("closes the receipts dialog with Escape straight after opening and hands focus back to the opener", async () => {
    const api = makeApi({
      "/api/v1/notices": ok([notice({ progress: { total: 3, acknowledged: 1 } })]),
      "/api/v1/notices/{id}/progress": ok({ total: 3, acknowledged: 1 }),
      "/api/v1/notices/{id}/receipts": ok({ items: [], total: 0 }),
    });
    renderBody(api, ["notice_manage"]);
    await userEvent.click(await screen.findByText("취업규칙 개정 통지"));
    const opener = await screen.findByRole("button", { name: text.actions.receipts });
    await userEvent.click(opener);
    const dialog = await screen.findByRole("dialog");
    // Focus moved INTO the dialog on open, so Escape reaches the overlay handler
    // without any prior click inside.
    expect(dialog.contains(document.activeElement)).toBe(true);
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it("opens the composer with the title focused, closes on Escape, and restores the opener's focus", async () => {
    const api = makeApi({ "/api/v1/notices": ok([]), "/api/v1/branches": ok([]) });
    renderBody(api, ["notice_manage"]);
    const opener = await screen.findByTestId("module-primary-action");
    await userEvent.click(opener);
    const dialog = await screen.findByRole("dialog", { name: text.composer.createTitle });
    expect(document.activeElement).toBe(within(dialog).getByLabelText(text.composer.titleLabel));
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it("filters the list by a status-chip label typed into search", async () => {
    const api = makeApi({ "/api/v1/notices": ok([notice(), draftRow]) });
    renderBody(api, ["notice_manage"]);
    await screen.findByText("NT-0707");
    await userEvent.type(screen.getByRole("searchbox"), text.status.draft);
    await waitFor(() => {
      expect(screen.queryByText("NT-0707")).toBeNull();
    });
    expect(screen.getByText("7월 안전교육 안내")).toBeVisible();
  });

  it("drills the list to an audience branch through its link chip", async () => {
    const branchRow = notice({
      id: "n5",
      code: "NT-0628",
      title: "경비팀 안전교육",
      audience_scope: "branches",
      audience_branches: [{ id: "b1", name: "안산공장" }],
    });
    const api = makeApi({ "/api/v1/notices": ok([notice(), branchRow]) });
    renderBody(api, []);
    await userEvent.click(await screen.findByText("경비팀 안전교육"));
    const detail = await screen.findByRole("complementary", { name: "경비팀 안전교육" });
    await userEvent.click(within(detail).getByRole("button", { name: /안산공장/ }));
    await waitFor(() => {
      expect(screen.queryByText("NT-0707")).toBeNull();
    });
    expect(screen.getByText("NT-0628")).toBeVisible();
    // Clearing the drill chip restores the full list.
    await userEvent.click(screen.getByRole("button", { name: `안산공장 ${text.filterClear}` }));
    expect(await screen.findByText("NT-0707")).toBeVisible();
  });

  it("fences stale responses when the auth session is replaced", async () => {
    let resolveFirst!: (value: Call) => void;
    const first = new Promise<Call>((resolve) => { resolveFirst = resolve; });
    const api = makeApi({ "/api/v1/notices": () => first });
    const view = renderBody(api, []);
    const api2 = makeApi({ "/api/v1/notices": ok([]) });
    useAuth.mockReturnValue({
      api: api2,
      session: {
        org_id: "org-b",
        user_id: "user-b",
        client_session_incarnation: "inc-2",
        access_token: "token-b",
        roles: [],
        feature_grants: [],
      },
    });
    view.rerender(<BoardScreenBody />);
    resolveFirst(ok([notice()]));
    expect(await screen.findByText(ko.console.module.list.empty)).toBeVisible();
    expect(screen.queryByText("NT-0707")).toBeNull();
  });

  it("keeps wide content scrollable inside its own containers (responsive affordances)", async () => {
    const api = makeApi({ "/api/v1/notices": ok([notice()]) });
    renderBody(api, []);
    await screen.findByText("NT-0707");
    const statbar = document.querySelector('[data-fidelity="module-statbar"]');
    expect(statbar).not.toBeNull();
    expect((statbar as HTMLElement).style.overflowX).toBe("auto");
    const grid = screen.getByRole("grid", { name: `${text.title} ${ko.console.module.list.label}` });
    expect(grid.style.overscrollBehavior).toBe("contain");
  });
});
