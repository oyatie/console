import { readFileSync } from "node:fs";
import { join } from "node:path";

import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { maintenanceStrings as text } from "../../i18n/maintenance";
import type { WorkOrderDetail, WorkOrderLens, WorkOrderRow, WorkOrderSettlement } from "./maintenanceApi";
import type { MaintenanceCapabilities } from "./maintenanceCapabilities";
import { MaintenanceScreen } from "./MaintenanceScreen";

const none: MaintenanceCapabilities = {
  canRead: false, canCreate: false, canEditIntake: false, canAssign: false, canStart: false,
  canSubmitReport: false, canReview: false, canManagePriority: false, canManageTarget: false,
  canAttachEvidence: false, canSettle: false, canReviewSettlement: false, canTriage: false,
};
const viewer: MaintenanceCapabilities = { ...none, canRead: true };
const requester: MaintenanceCapabilities = { ...viewer, canCreate: true };
const reviewer: MaintenanceCapabilities = { ...viewer, canReview: true };
const settler: MaintenanceCapabilities = { ...viewer, canSettle: true };

const FUTURE = "2999-01-01T00:00:00Z";

function row(over: Partial<WorkOrderRow> = {}): WorkOrderRow {
  return {
    id: "wo-1",
    request_no: "20260724-001",
    branch_id: "branch-1",
    status: "UNASSIGNED",
    priority: "P3",
    result_type: "UNKNOWN",
    target_due_at: FUTURE,
    created_at: "2026-07-24T00:00:00Z",
    updated_at: "2026-07-24T00:00:00Z",
    equipment: {
      id: "eq-1", equipment_no: "FL-2643", management_no: "M-2643", model: "D30S-7",
      status: "ACTIVE", specification: "3T", ton_text: "3.0t",
    },
    customer: { id: "cu-1", name: "Alpha Logistics" },
    site: { id: "site-1", name: "Busan Yard" },
    site_contact: null,
    assignments: [],
    ...over,
  };
}

function detailOf(over: Partial<WorkOrderDetail> = {}): WorkOrderDetail {
  return {
    ...row(),
    symptom: "lift chain noise",
    customer_request: null,
    delay_reason: null,
    delay_note: null,
    diagnosis: null,
    action_taken: null,
    report_submitted_by: null,
    report_submitted_at: null,
    kpi_excluded: false,
    evidence_verified: false,
    approval_line: [],
    status_history: [{
      id: "hist-1", actor: "user-1", action: "CREATE", from_status: null,
      to_status: "RECEIVED", occurred_at: "2026-07-24T00:00:00Z",
    }],
    evidence: [],
    ...over,
  };
}

function lensOf(over: Partial<WorkOrderLens["aggregates"]> = {}): WorkOrderLens {
  return {
    object_type: "work_order",
    aggregates: { total_count: 2, p1_count: 1, overdue_open_count: 1, unassigned_count: 2, ...over },
    facets: {
      status: [{ value: "UNASSIGNED", count: 2, filters: { status: "UNASSIGNED" } }],
      priority: [{ value: "P1", count: 1, filters: { priority: "P1" } }],
    },
    histograms: { target_due_date: [] },
    listograms: { customers: [], sites: [] },
  };
}

type Result = { data?: unknown; error?: unknown; response: Response };
const ok = (data: unknown): Result => ({ data, response: new Response(null, { status: 200 }) });
const fail = (status: number, message: string): Result => ({
  error: { error: { message } }, response: new Response(null, { status }),
});

/** URL-keyed transport stub; an array value is consumed one result per call. */
function router(map: Record<string, Result | Result[] | undefined>) {
  return vi.fn((url: string) => {
    const entry = map[url];
    if (Array.isArray(entry)) return Promise.resolve(entry.length > 1 ? (entry.shift() as Result) : entry[0]);
    return Promise.resolve(entry ?? fail(404, "unrouted"));
  });
}

type Router = ReturnType<typeof router>;

function client(get: Router, post: Router = router({})) {
  return {
    api: { GET: get, POST: post, PUT: router({}), PATCH: router({}) } as unknown as ConsoleApiClient,
    get,
    post,
  };
}

function page(items: WorkOrderRow[], lens: WorkOrderLens = lensOf()) {
  return ok({ items, limit: 50, offset: 0, total: items.length, lens });
}

function renderScreen(capabilities: MaintenanceCapabilities, api: ConsoleApiClient) {
  return render(
    <MaintenanceScreen api={api} branchId="branch-1" actorId="user-1" capabilities={capabilities} sessionKey="session-a" />,
  );
}

/** The shared-track list rows; lane cards share names, so queries must scope here. */
async function findListRow(name: RegExp) {
  const list = await screen.findByRole("list", { name: text.list });
  return within(list).findByRole("button", { name });
}

afterEach(() => {
  window.sessionStorage.clear();
});

describe("MaintenanceScreen", () => {
  it("denies an unauthorized user before fetching or exposing actions", () => {
    const { api, get } = client(router({}));
    renderScreen(none, api);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.create })).toBeNull();
    expect(get).not.toHaveBeenCalled();
  });

  it("retries a failed list load, then renders lens stats and drills the P1 stat into a filter", async () => {
    const { api, get } = client(router({
      "/api/v1/work-orders": [fail(500, "backend unavailable"), page([row()])],
    }));
    renderScreen(viewer, api);
    expect(await screen.findByRole("alert")).toHaveTextContent("backend unavailable");
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await findListRow(/20260724-001/)).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: new RegExp(`^${text.stats.p1}`) }));
    await waitFor(() => {
      const lastList = get.mock.calls.filter(([url]) => url === "/api/v1/work-orders").at(-1);
      expect(lastList?.[1]).toMatchObject({ params: { query: { priority: ["P1"] } } });
    });
  });

  it("renders a truthful empty state from an empty backend queue", async () => {
    const { api } = client(router({
      "/api/v1/work-orders": page([], lensOf({ total_count: 0, p1_count: 0, overdue_open_count: 0, unassigned_count: 0 })),
    }));
    renderScreen(viewer, api);
    expect(await screen.findByText(text.empty)).toBeVisible();
  });

  it("triages the board lanes: SLA-risk unassigned, planned unassigned, assigned active", async () => {
    const slaRisk = row({ id: "wo-a", request_no: "20260724-001", priority: "P1" });
    const planned = row({ id: "wo-b", request_no: "20260724-002" });
    const active = row({
      id: "wo-c", request_no: "20260724-003", status: "IN_PROGRESS",
      assignments: [{ id: "as-1", mechanic_id: "mech-1", mechanic_name: "Kim", role: "PRIMARY", assigned_at: "2026-07-24T01:00:00Z" }],
    });
    const { api } = client(router({ "/api/v1/work-orders": page([slaRisk, planned, active]) }));
    renderScreen(viewer, api);
    const dueSoon = await screen.findByRole("region", { name: text.lanes.dueSoonUnassigned });
    expect(dueSoon).toHaveTextContent("20260724-001");
    expect(screen.getByRole("region", { name: text.lanes.plannedUnassigned })).toHaveTextContent("20260724-002");
    expect(screen.getByRole("region", { name: text.lanes.assignedActive })).toHaveTextContent("20260724-003");
  });

  it("narrows the queue and the board with the header search", async () => {
    const alpha = row({ id: "wo-a", request_no: "20260724-001" });
    const beta = row({
      id: "wo-b", request_no: "20260724-002",
      site: { id: "site-2", name: "Ulsan Depot" },
    });
    const { api } = client(router({ "/api/v1/work-orders": page([alpha, beta]) }));
    renderScreen(viewer, api);
    expect(await findListRow(/20260724-002/)).toBeVisible();
    await userEvent.type(screen.getByRole("searchbox", { name: text.search }), "Ulsan");
    const list = await screen.findByRole("list", { name: text.list });
    expect(within(list).getByRole("button", { name: /20260724-002/ })).toBeVisible();
    expect(within(list).queryByRole("button", { name: /20260724-001/ })).toBeNull();
    const planned = screen.getByRole("region", { name: text.lanes.plannedUnassigned });
    expect(planned).not.toHaveTextContent("20260724-001");
  });

  it("hides rows from another branch even if the transport returns them", async () => {
    const { api } = client(router({
      "/api/v1/work-orders": page([row(), row({ id: "wo-x", request_no: "20260724-009", branch_id: "branch-2" })]),
    }));
    renderScreen(viewer, api);
    expect(await findListRow(/20260724-001/)).toBeVisible();
    expect(screen.queryByRole("button", { name: /20260724-009/ })).toBeNull();
  });

  it("moves row focus with j/k and opens the focused object with Enter", async () => {
    const first = row({ id: "wo-a", request_no: "20260724-001" });
    const second = row({ id: "wo-b", request_no: "20260724-002" });
    const { api, get } = client(router({
      "/api/v1/work-orders": page([first, second]),
      "/api/v1/work-orders/{workOrderId}": ok(detailOf({ id: "wo-b", request_no: "20260724-002" })),
    }));
    const view = renderScreen(viewer, api);
    const list = await screen.findByRole("list", { name: text.list });
    const rows = within(list).getAllByRole("button", { name: /2026072/ });
    rows[0]?.focus();
    await userEvent.keyboard("j");
    expect(document.activeElement).toBe(rows[1]);
    await userEvent.keyboard("{Enter}");
    expect(await screen.findByRole("heading", { name: "20260724-002" })).toBeVisible();
    expect(get).toHaveBeenCalledWith("/api/v1/work-orders/{workOrderId}", expect.objectContaining({
      params: { path: { workOrderId: "wo-b" } },
    }));
    const currentStep = view.container.querySelector('.maintenance__flow li[data-step="cur"]');
    expect(currentStep).toHaveTextContent(text.flow.intake);
  });

  it("keeps the selection and the composer draft across a remount (refresh)", async () => {
    const routes = () => ({
      "/api/v1/work-orders": page([row()]),
      "/api/v1/work-orders/{workOrderId}": ok(detailOf()),
    });
    const one = client(router(routes()));
    const view = renderScreen(requester, one.api);
    await userEvent.click(await findListRow(/20260724-001/));
    expect(await screen.findByRole("heading", { name: "20260724-001" })).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.create }));
    await userEvent.type(screen.getByLabelText(text.form.managementNo), "M-2643");
    view.unmount();

    const two = client(router(routes()));
    renderScreen(requester, two.api);
    expect(await screen.findByRole("heading", { name: "20260724-001" })).toBeVisible();
    expect(screen.getByLabelText(text.form.managementNo)).toHaveValue("M-2643");
  });

  it("presents a backend detail denial as denied, not as a retryable error", async () => {
    const { api } = client(router({
      "/api/v1/work-orders": page([row()]),
      "/api/v1/work-orders/{workOrderId}": fail(403, "forbidden"),
    }));
    renderScreen(viewer, api);
    await userEvent.click(await findListRow(/20260724-001/));
    expect(await screen.findByText(text.detailDenied)).toBeVisible();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByRole("button", { name: text.retry })).toBeNull();
  });

  it("fails closed on reject until a memo is entered, then posts the review", async () => {
    const submitted = row({ status: "REPORT_SUBMITTED" });
    const { api, post } = client(
      router({
        "/api/v1/work-orders": page([submitted]),
        "/api/v1/work-orders/{workOrderId}": ok(detailOf({ status: "REPORT_SUBMITTED" })),
        "/api/v1/work-orders/{workOrderId}/settlement": fail(404, "no settlement"),
      }),
      router({ "/api/v1/work-orders/{workOrderId}/reject": ok(detailOf({ status: "REJECTED" })) }),
    );
    renderScreen(reviewer, api);
    await userEvent.click(await findListRow(/20260724-001/));
    const reject = await screen.findByRole("button", { name: text.actions.reject });
    await userEvent.click(reject);
    expect(screen.getByText(text.actions.rejectMemoRequired)).toBeVisible();
    expect(post).not.toHaveBeenCalled();
    await userEvent.type(screen.getByLabelText(text.actions.reviewComment), "insufficient evidence");
    await userEvent.click(reject);
    await waitFor(() => {
      expect(post).toHaveBeenCalledWith("/api/v1/work-orders/{workOrderId}/reject", expect.objectContaining({
        body: { memo: "insufficient evidence" },
      }));
    });
  });

  it("does not carry a review memo typed for one order into the next selected order", async () => {
    const first = row({ id: "wo-a", request_no: "20260724-001", status: "REPORT_SUBMITTED" });
    const second = row({ id: "wo-b", request_no: "20260724-002", status: "REPORT_SUBMITTED" });
    const details: Record<string, WorkOrderDetail> = {
      "wo-a": detailOf({ id: "wo-a", request_no: "20260724-001", status: "REPORT_SUBMITTED" }),
      "wo-b": detailOf({ id: "wo-b", request_no: "20260724-002", status: "REPORT_SUBMITTED" }),
    };
    const get = vi.fn((url: string, init?: { params?: { path?: { workOrderId?: string } } }) => {
      if (url === "/api/v1/work-orders") return Promise.resolve(page([first, second]));
      if (url === "/api/v1/work-orders/{workOrderId}") {
        return Promise.resolve(ok(details[init?.params?.path?.workOrderId ?? ""]));
      }
      return Promise.resolve(fail(404, "no settlement"));
    });
    const { api } = client(get);
    renderScreen(reviewer, api);
    await userEvent.click(await findListRow(/20260724-001/));
    await userEvent.type(await screen.findByLabelText(text.actions.reviewComment), "memo for A");
    await userEvent.click(await findListRow(/20260724-002/));
    await screen.findByRole("heading", { name: "20260724-002" });
    expect(screen.getByLabelText(text.actions.reviewComment)).toHaveValue("");
  });

  it("keeps an authorized detail readable when only the settlement read is denied", async () => {
    const { api } = client(router({
      "/api/v1/work-orders": page([row({ status: "REPORT_SUBMITTED" })]),
      "/api/v1/work-orders/{workOrderId}": ok(detailOf({ status: "REPORT_SUBMITTED" })),
      "/api/v1/work-orders/{workOrderId}/settlement": fail(403, "settlement denied"),
    }));
    renderScreen(viewer, api);
    await userEvent.click(await findListRow(/20260724-001/));
    expect(await screen.findByRole("heading", { name: "20260724-001" })).toBeVisible();
    expect(screen.queryByText(text.detailDenied)).toBeNull();
    expect(screen.queryByRole("region", { name: text.settlement.heading })).toBeNull();
    expect(screen.queryByText(text.settlement.none)).toBeNull();
  });

  it("drafts a settlement after report submission and reconciles the backend table", async () => {
    const submitted = row({ status: "REPORT_SUBMITTED" });
    const draft: WorkOrderSettlement = {
      id: "st-1", work_order_id: "wo-1", branch_id: "br-1", status: "DRAFT",
      total_amount_krw: 120000, voucher_ref: null, note: null,
      lines: [{ id: "sl-1", kind: "LABOR", label: "정비 인건", amount_krw: 120000, source_ref: null, sort_order: 0 }],
      created_by: "u-1", submitted_by: null, submitted_at: null, approved_by: null, approved_at: null,
      created_at: "2026-07-24T11:00:00Z", updated_at: "2026-07-24T11:00:00Z",
    };
    const { api, post } = client(
      router({
        "/api/v1/work-orders": page([submitted]),
        "/api/v1/work-orders/{workOrderId}": ok(detailOf({ status: "REPORT_SUBMITTED" })),
        "/api/v1/work-orders/{workOrderId}/settlement": [fail(404, "no settlement"), ok(draft)],
      }),
      router({ "/api/v1/work-orders/{workOrderId}/settlement": ok(draft) }),
    );
    renderScreen(settler, api);
    await userEvent.click(await findListRow(/20260724-001/));
    await userEvent.type(await screen.findByLabelText(text.settlement.amount), "120000");
    await userEvent.type(await screen.findByLabelText(text.settlement.lineLabel), "정비 인건");
    await userEvent.click(screen.getByRole("button", { name: text.settlement.create }));
    await waitFor(() => {
      expect(post).toHaveBeenCalledWith("/api/v1/work-orders/{workOrderId}/settlement", expect.objectContaining({
        body: { lines: [{ kind: "LABOR", label: "정비 인건", amount_krw: 120000, sort_order: 0 }] },
      }));
    });
    expect(await screen.findByText(text.settlement.status.DRAFT)).toBeVisible();
    expect(screen.getByRole("button", { name: text.settlement.submit })).toBeVisible();
    expect(screen.getAllByText("120,000").length).toBeGreaterThan(0);
  });

  it("collapses the row grid and lanes for narrow viewports in the stylesheet", () => {
    const css = readFileSync(join(process.cwd(), "src/console/maintenance/maintenance.css"), "utf8");
    const narrow = css.split("@media (max-width: 900px)")[1] ?? "";
    expect(narrow).toContain(".maintenance__lanes { grid-template-columns: 1fr; }");
    expect(narrow).toContain("display: none");
    expect(narrow).toMatch(/\.maintenance__row \{ grid-template-columns: 7\.5rem minmax\(0, 1fr\) 6\.5rem/);
  });
});
