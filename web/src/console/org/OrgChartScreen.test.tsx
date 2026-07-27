import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { orgStrings as text } from "../../i18n/org";
import type { OrgChangeDetail, OrgChangeSummary } from "./orgApi";
import type { OrgCapabilities } from "./orgCapabilities";
import { OrgChartScreen } from "./OrgChartScreen";

const full: OrgCapabilities = { canReadTree: true, canReadChanges: true, canDraft: true, canApprove: true, canApply: true };
const reader: OrgCapabilities = { ...full, canDraft: false, canApprove: false, canApply: false };
const denied: OrgCapabilities = { canReadTree: false, canReadChanges: false, canDraft: false, canApprove: false, canApply: false };

const chart = {
  companies: [{
    company: "코스",
    total: 12,
    active: 10,
    units: [{
      name: "운영팀",
      total: 6,
      positions: [{ title: "팀장", total: 1, employees: [{ id: "e1", name: "김하나", status: "재직" }] }],
    }],
  }],
};

const regions = [{ id: "r1", name: "경남", deactivated_at: null, created_at: "2026-01-01T00:00:00Z" }];
const branches = [{ id: "b1", regionId: "r1", name: "창원지점", deactivated_at: null, created_at: "2026-01-01T00:00:00Z" }];
const me = { employee_company: "코스" };
const entities = [{ orgId: "org-1", slug: "coss", name: "코스", status: "ACTIVE" }];

const changeSummary: OrgChangeSummary = {
  id: "oc-1",
  code: "OC-2026-0001",
  kind: "REORG",
  status: "IN_APPROVAL",
  target: { kind: "ENTITY", ref: "org-1", label: "코스" },
  effectiveDate: "2026-09-01",
  reason: "조직 개편",
  headcount: 10,
  siteCount: 1,
  teamCount: 1,
  draftedBy: "actor-2",
  created_at: "2026-07-01T00:00:00Z",
  updated_at: "2026-07-02T00:00:00Z",
};

const changeDetail: OrgChangeDetail = {
  ...changeSummary,
  proposal: [{ op: "RENAME_BRANCH", branchId: "b1", name: "창원제1지점" }],
  preflight: {
    computedAt: "2026-07-02T00:00:00Z",
    stale: false,
    blockers: [],
    warnings: [{ code: "OPEN_DOCS", label: "진행 중 공고·결재 종결 필요" }],
    headcount: 10,
    dependentsTotal: 2,
  },
  approvalSteps: [
    { id: "s1", stepOrder: 1, roleKey: "hr", decision: "APPROVED", decidedBy: "hr-1", decidedAt: "2026-07-02T00:00:00Z" },
    { id: "s2", stepOrder: 2, roleKey: "finance", decision: "PENDING" },
    { id: "s3", stepOrder: 3, roleKey: "legal", decision: "PENDING" },
    { id: "s4", stepOrder: 4, roleKey: "executive", decision: "PENDING" },
  ],
  settlementItems: [],
  events: [{ at: "2026-07-01T09:00:00Z", actor: "actor-2", action: "조직 변경 상신", reason: "사전점검 통과" }],
};

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function fail(status: number) {
  return { error: {}, response: new Response(null, { status }) };
}

type Routes = Partial<Record<string, () => unknown>>;

function apiWith(routes: Routes) {
  const api = { GET: vi.fn(), POST: vi.fn(), PATCH: vi.fn() } as unknown as ConsoleApiClient;
  const dispatch = (path: string) => {
    const handler = routes[path];
    return Promise.resolve(handler ? handler() : fail(404));
  };
  vi.mocked(api.GET).mockImplementation(dispatch as never);
  vi.mocked(api.POST).mockImplementation(dispatch as never);
  vi.mocked(api.PATCH).mockImplementation(dispatch as never);
  return api;
}

const happyRoutes: Routes = {
  "/api/v1/hr/org-chart": () => ok(chart),
  "/api/v1/regions": () => ok(regions),
  "/api/v1/branches": () => ok(branches),
  "/api/v1/users/me": () => ok(me),
  "/api/v1/org-entities": () => ok(entities),
  "/api/v1/org-changes": () => ok({ items: [changeSummary], total: 1 }),
  "/api/v1/org-changes/{id}": () => ok(changeDetail),
};

/** Entity-card button accessible name = name + headcount spans ("코스" + "10"). */
const entityCardName = (name: string) => name.replace(/\s/g, "") === "코스10";

function renderScreen(capabilities: OrgCapabilities = full, api = apiWith(happyRoutes), onNavigate = vi.fn()) {
  const view = render(
    <OrgChartScreen api={api} actorId="actor-1" capabilities={capabilities} sessionKey="session-a" onNavigate={onNavigate} />,
  );
  return { view, api, onNavigate };
}

beforeEach(() => {
  window.sessionStorage.clear();
});

describe("OrgChartScreen", () => {
  it("denies an unauthorized user before fetching anything", () => {
    const api = apiWith(happyRoutes);
    renderScreen(denied, api);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("renders the tree, sites, and derived stat bar from the backend responses", async () => {
    renderScreen(reader);
    expect(await screen.findByRole("button", { name: entityCardName })).toBeVisible();
    expect(screen.getByRole("button", { name: /창원지점/ })).toBeVisible();
    expect(screen.getByText(text.rootMeta)).toBeVisible();
    const root = document.querySelector(".org-root-card");
    expect(root?.textContent).toContain("10");
  });

  it("shows the truthful empty state when every structure source is empty", async () => {
    renderScreen(reader, apiWith({
      ...happyRoutes,
      "/api/v1/hr/org-chart": () => ok({ companies: [] }),
      "/api/v1/org-entities": () => ok([]),
      "/api/v1/branches": () => ok([]),
    }));
    expect(await screen.findByText(text.empty)).toBeVisible();
  });

  it("surfaces a tree load error with a working retry", async () => {
    let calls = 0;
    renderScreen(reader, apiWith({
      ...happyRoutes,
      "/api/v1/hr/org-chart": () => (calls++ === 0 ? fail(500) : ok(chart)),
    }));
    expect(await screen.findByRole("alert")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await screen.findByRole("button", { name: /창원지점/ })).toBeVisible();
  });

  it("treats denied side reads as absence, not error (deny-by-omission)", async () => {
    renderScreen(reader, apiWith({
      ...happyRoutes,
      "/api/v1/regions": () => fail(403),
      "/api/v1/branches": () => fail(403),
      "/api/v1/org-entities": () => fail(403),
    }));
    expect(await screen.findByRole("button", { name: entityCardName })).toBeVisible();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByText(/창원지점/)).toBeNull();
  });

  it("reports the org-change API as unavailable while the backend lane has not landed", async () => {
    renderScreen(reader, apiWith({ ...happyRoutes, "/api/v1/org-changes": () => fail(404) }));
    expect(await screen.findByText(text.changesUnavailable)).toBeVisible();
  });

  it("does not restore a persisted change modal after the read capability is revoked", async () => {
    window.sessionStorage.setItem(
      "console.org.sandbox",
      JSON.stringify({ actorId: "actor-1", ops: [], openChangeId: "oc-1" }),
    );
    const api = apiWith(happyRoutes);
    renderScreen({ ...reader, canReadChanges: false }, api);
    await screen.findByRole("button", { name: entityCardName });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(vi.mocked(api.GET)).not.toHaveBeenCalledWith("/api/v1/org-changes/{id}", expect.anything());
    expect(vi.mocked(api.GET)).not.toHaveBeenCalledWith("/api/v1/org-changes", expect.anything());
  });

  it("opens a team card via keyboard, drills to messenger/people, and closes with Escape", async () => {
    const { onNavigate } = renderScreen(reader);
    const entity = await screen.findByRole("button", { name: entityCardName });
    await userEvent.click(entity);
    const team = await screen.findByRole("button", { name: /운영팀/ });
    team.focus();
    await userEvent.keyboard("{Enter}");
    const dialog = await screen.findByRole("dialog", { name: text.teamInfo });
    expect(dialog).toBeVisible();
    // 김하나 appears twice by design: 책임자 link + position roster entry.
    expect(screen.getAllByText("김하나")).toHaveLength(2);
    expect(screen.getByText(text.headDerived)).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.teamChannel }));
    expect(onNavigate).toHaveBeenLastCalledWith("messenger");
    await userEvent.click(screen.getByRole("button", { name: text.teamRoster }));
    expect(onNavigate).toHaveBeenLastCalledWith("people");
    await userEvent.keyboard("{Escape}");
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: text.teamInfo })).toBeNull();
    });
  });

  it("collects sandbox edits into a proposal that survives a remount and drafts a real org change", async () => {
    const detailAfterCreate: OrgChangeDetail = { ...changeDetail, id: "oc-2", code: "OC-2026-0002", status: "DRAFT", approvalSteps: [], events: [] };
    const createBody = vi.fn();
    const api = apiWith(happyRoutes);
    vi.mocked(api.POST).mockImplementation(((path: string, init: { body?: unknown }) => {
      if (path === "/api/v1/org-changes") {
        createBody(init.body);
        return Promise.resolve(ok(detailAfterCreate));
      }
      return Promise.resolve(fail(404));
    }) as never);
    const { view } = renderScreen(full, api);

    await userEvent.click(await screen.findByRole("button", { name: text.edit }));
    const rename = screen.getByLabelText(`${text.ocOps.RENAME_BRANCH} · 창원지점`);
    await userEvent.clear(rename);
    await userEvent.type(rename, "창원제1지점");
    await userEvent.tab();
    expect(await screen.findByText(`${text.dirtyBanner} 1${text.dirtyBannerUnit}`)).toBeVisible();

    view.rerender(
      <OrgChartScreen api={api} actorId="actor-1" capabilities={full} sessionKey="session-b" onNavigate={vi.fn()} />,
    );
    expect(await screen.findByText(`${text.dirtyBanner} 1${text.dirtyBannerUnit}`)).toBeVisible();

    await userEvent.click(screen.getByRole("button", { name: text.dirtyCta }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toBeVisible();
    expect(screen.getByText(`${text.ocOps.RENAME_BRANCH} · 창원제1지점`)).toBeVisible();

    await userEvent.type(screen.getByLabelText(text.ocReason), "지점 명칭 정비");
    await userEvent.click(screen.getByRole("button", { name: text.ocCreate }));
    await waitFor(() => {
      expect(createBody).toHaveBeenCalledWith(expect.objectContaining({
        kind: "REORG",
        target: expect.objectContaining({ label: "코스" }),
        proposal: [{ op: "RENAME_BRANCH", branchId: "b1", name: "창원제1지점" }],
      }));
    });
    await waitFor(() => {
      expect(screen.queryByText(`${text.dirtyBanner} 1${text.dirtyBannerUnit}`)).toBeNull();
    });
  });

  it("lets an approver decide only the next pending SoD step through the real decision route", async () => {
    const decide = vi.fn();
    const api = apiWith(happyRoutes);
    vi.mocked(api.POST).mockImplementation(((path: string, init: { params?: { path?: Record<string, string> }; body?: unknown }) => {
      if (path === "/api/v1/org-changes/{id}/approval-steps/{stepId}/decision") {
        decide(init.params?.path, init.body);
        return Promise.resolve(ok({
          ...changeDetail,
          approvalSteps: changeDetail.approvalSteps.map((step) =>
            step.id === "s2" ? { ...step, decision: "APPROVED" as const, decidedBy: "actor-1" } : step),
        }));
      }
      return Promise.resolve(fail(404));
    }) as never);
    renderScreen(full, api);

    await userEvent.click(await screen.findByRole("button", { name: /OC-2026-0001/ }));
    await screen.findByRole("dialog");
    await waitFor(() => {
      expect(screen.getAllByRole("button", { name: text.ocApprove })).toHaveLength(1);
    });
    await userEvent.click(screen.getByRole("button", { name: text.ocApprove }));
    await waitFor(() => {
      expect(decide).toHaveBeenCalledWith({ id: "oc-1", stepId: "s2" }, { decision: "APPROVED" });
    });
    expect(screen.getByRole("button", { name: `${text.ocWaiting} (2/4)` })).toBeDisabled();
  });

  it("gates 폐지 보관 behind the settlement checklist through the real completion route", async () => {
    const settling: OrgChangeDetail = {
      ...changeDetail,
      kind: "DISSOLVE",
      status: "SETTLING",
      approvalSteps: changeDetail.approvalSteps.map((step) => ({ ...step, decision: "APPROVED" as const, decidedBy: "hr-1" })),
      settlementItems: [
        { id: "st1", itemKey: "ASSETS", label: "자산 이관·반납", done: true },
        { id: "st2", itemKey: "PAYROLL_SOCIAL_FINAL", label: "급여·4대보험·퇴직 정산", done: false },
      ],
    };
    const complete = vi.fn();
    const api = apiWith({ ...happyRoutes, "/api/v1/org-changes/{id}": () => ok(settling) });
    vi.mocked(api.POST).mockImplementation(((path: string, init: { params?: { path?: Record<string, string> } }) => {
      if (path === "/api/v1/org-changes/{id}/settlement-items/{itemId}/complete") {
        complete(init.params?.path);
        return Promise.resolve(ok({
          ...settling,
          settlementItems: settling.settlementItems.map((item) => ({ ...item, done: true })),
        }));
      }
      return Promise.resolve(fail(404));
    }) as never);
    renderScreen(full, api);

    await userEvent.click(await screen.findByRole("button", { name: /OC-2026-0001/ }));
    await screen.findByRole("dialog");
    const archive = await screen.findByRole("button", { name: text.ocArchive });
    expect(archive).toBeDisabled();
    await userEvent.click(screen.getByRole("button", { name: text.ocSettleAction }));
    await waitFor(() => {
      expect(complete).toHaveBeenCalledWith({ id: "oc-1", itemId: "st2" });
    });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: text.ocArchive })).toBeEnabled();
    });
  });

  it("hides every approval and draft affordance from a read-only capability set", async () => {
    const { onNavigate } = renderScreen(reader);
    await userEvent.click(await screen.findByRole("button", { name: /OC-2026-0001/ }));
    await screen.findByRole("dialog");
    await waitFor(() => {
      expect(screen.getByText(text.ocEvents)).toBeVisible();
    });
    expect(screen.queryByRole("button", { name: text.ocApprove })).toBeNull();
    expect(screen.queryByRole("button", { name: text.edit })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: text.ocEventsAudit }));
    expect(onNavigate).toHaveBeenLastCalledWith("audit");
  });

  it("keeps the tree inside a horizontally scrollable canvas for narrow viewports", async () => {
    renderScreen(reader);
    await screen.findByRole("button", { name: entityCardName });
    const canvas = document.querySelector(".org-canvas");
    const inner = document.querySelector(".org-canvas-inner");
    expect(canvas).not.toBeNull();
    expect(inner).not.toBeNull();
  });
});
