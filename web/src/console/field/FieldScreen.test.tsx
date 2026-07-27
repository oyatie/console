import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { createConsoleApiClient } from "../../api/client";
import { fieldStrings as text } from "../../i18n/field";
import { ko } from "../../i18n/ko";
import type {
  FieldSiteDetail,
  FieldSiteRow,
  TicketDetail,
  TicketSummary,
} from "./fieldApi";
import type { FieldCapabilities } from "./fieldCapabilities";
import { FieldScreen } from "./FieldScreen";

const operator: FieldCapabilities = {
  canRead: true,
  canIntake: true,
  canTriage: true,
  canAccept: true,
  canComment: true,
};
const viewer: FieldCapabilities = {
  canRead: true,
  canIntake: false,
  canTriage: false,
  canAccept: false,
  canComment: false,
};
const denied: FieldCapabilities = {
  canRead: false,
  canIntake: false,
  canTriage: false,
  canAccept: false,
  canComment: false,
};

function siteRow(over: Partial<FieldSiteRow> = {}): FieldSiteRow {
  return {
    site_id: "site-1",
    site_name: "대원강업 상주",
    branch_id: "branch-a",
    customer_id: "customer-1",
    customer_name: "대원강업",
    address: "창원시 성산구",
    latitude: null,
    longitude: null,
    open_ticket_count: 1,
    breached_ticket_count: 0,
    next_due_at: null,
    active_work_order_count: 1,
    last_arrival_at: null,
    sla: "OK",
    ...over,
  };
}

function ticketSummary(over: Partial<TicketSummary> = {}): TicketSummary {
  return {
    id: "ticket-1",
    branch_id: "branch-a",
    origin: "CUSTOMER",
    category: "OPERATIONAL",
    priority: "HIGH",
    status: "RESOLVED",
    title: "경비 결원 대근 요청",
    requester_user_id: "user-9",
    requester_name: "이종호",
    assignee_user_id: "user-1",
    assignee_name: "정하늘",
    due_at: null,
    created_at: "2026-07-20T01:00:00Z",
    updated_at: "2026-07-20T01:00:00Z",
    resolved_at: "2026-07-21T01:00:00Z",
    closed_at: null,
    site_id: "site-1",
    site_name: "대원강업 상주",
    customer_id: "customer-1",
    customer_name: "대원강업",
    work_order_id: null,
    ...over,
  };
}

function siteDetail(over: Partial<FieldSiteDetail> = {}): FieldSiteDetail {
  return {
    site: {
      id: "site-1",
      name: "대원강업 상주",
      branch_id: "branch-a",
      customer_id: "customer-1",
      customer_name: "대원강업",
      address: "창원시 성산구",
      province: "경남",
      city: "창원",
      postal_code: null,
      latitude: null,
      longitude: null,
      geofence_radius_m: 150,
      contact_name: "이종호",
      contact_phone: null,
    },
    sla: {
      state: "OK",
      open: 1,
      breached: 0,
      next_due_at: "2026-07-25T09:00:00Z",
      resolved_within_sla_90d: 12,
      resolved_breached_90d: 1,
    },
    tickets: [ticketSummary()],
    work_orders: [
      {
        id: "wo-1",
        request_no: "WO-2638",
        status: "IN_PROGRESS",
        priority: "P2",
        target_due_at: "2026-07-26T09:00:00Z",
        report_submitted_at: null,
        result_type: null,
        created_at: "2026-07-19T01:00:00Z",
      },
    ],
    attendance: [
      {
        user_id: "user-2",
        user_name: "김성호",
        work_order_id: "wo-1",
        kind: "ARRIVAL",
        occurred_at: "2026-07-23T00:00:00Z",
      },
    ],
    acceptances: [],
    ...over,
  };
}

function ticketDetail(over: Partial<TicketDetail> = {}): TicketDetail {
  return { ticket: ticketSummary(), comments: [], ...over };
}

function jsonResponse(data: unknown, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "content-type": "application/json" },
  });
}

type Routes = Partial<Record<string, (request: Request) => Response>>;

/** Fetch-boundary mock: requests route by pathname; unrouted paths 404 with the
 * canonical error envelope. */
function stubRoutes(routes: Routes) {
  const fetchMock = vi.fn((input: Request | string | URL) => {
    const request = input instanceof Request ? input : new Request(input);
    const route = routes[new URL(request.url).pathname];
    return Promise.resolve(
      route
        ? route(request)
        : jsonResponse({ error: { code: "not_found", message: "not found" } }, 404),
    );
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

function renderScreen(capabilities: FieldCapabilities = operator, sessionKey = "session-a") {
  const api = createConsoleApiClient("bearer-token");
  return render(
    <FieldScreen
      api={api}
      branchId="branch-a"
      actorId="user-1"
      capabilities={capabilities}
      sessionKey={sessionKey}
    />,
  );
}

describe("FieldScreen", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    window.localStorage.clear();
  });

  it("denies an unauthorized user before fetching anything", () => {
    const fetchMock = stubRoutes({});
    renderScreen(denied);
    expect(screen.getByRole("status")).toHaveTextContent(text.denied);
    expect(screen.queryByRole("button", { name: text.intake })).toBeNull();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("renders the backend list with drillable stats derived from the same rows", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({
          items: [
            siteRow(),
            siteRow({
              site_id: "site-2",
              site_name: "평택항 물류센터",
              customer_name: "대한제강",
              sla: "BREACHED",
              open_ticket_count: 0,
              active_work_order_count: 0,
            }),
          ],
          next_cursor: null,
          total: 2,
        }),
    });
    renderScreen(viewer);
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
    const breachedStat = screen.getByRole("button", { name: new RegExp(text.stats.breached) });
    expect(breachedStat).toHaveTextContent("1");
    const totalStat = screen.getByRole("button", { name: new RegExp(text.stats.total) });
    expect(totalStat).toHaveTextContent("2");
    // Viewer capability: intake affordance is absent, not disabled.
    expect(screen.queryByRole("button", { name: text.intake })).toBeNull();
  });

  it("shows the empty state with a clear-filters action only when filters caused it", async () => {
    stubRoutes({
      "/api/v1/field/sites": (request) => {
        const sla = new URL(request.url).searchParams.get("sla");
        return jsonResponse({
          items: sla === "BREACHED" ? [] : [siteRow()],
          next_cursor: null,
          total: sla === "BREACHED" ? 0 : 1,
        });
      },
    });
    renderScreen(viewer);
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.sla.BREACHED }));
    expect(await screen.findByText(text.emptyFiltered)).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.clearFilters }));
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
  });

  it("surfaces a list failure as an alert and recovers through retry", async () => {
    let failures = 1;
    stubRoutes({
      "/api/v1/field/sites": () => {
        if (failures > 0) {
          failures -= 1;
          return jsonResponse({ error: { code: "internal", message: "server down" } }, 500);
        }
        return jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 });
      },
    });
    renderScreen(viewer);
    expect(await screen.findByRole("alert")).toHaveTextContent("server down");
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
  });

  it("navigates rows with J/K and opens the pinned detail with Enter", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({
          items: [siteRow(), siteRow({ site_id: "site-2", site_name: "평택항 물류센터" })],
          next_cursor: null,
          total: 2,
        }),
      "/api/v1/field/sites/site-2": () =>
        jsonResponse(
          siteDetail({
            site: { ...siteDetail().site, id: "site-2", name: "평택항 물류센터" },
          }),
        ),
    });
    renderScreen(viewer);
    expect(await screen.findByText("대원강업 상주")).toBeVisible();
    const grid = screen.getByRole("grid", { name: text.listLabel });
    grid.focus();
    await userEvent.keyboard("jj");
    await userEvent.keyboard("{Enter}");
    const detail = await screen.findByRole("complementary", { name: text.detail.label });
    expect(await within(detail).findByRole("heading", { name: "평택항 물류센터" })).toBeVisible();
  });

  it("renders site detail with upstream links and downstream history sections", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
      "/api/v1/field/sites/site-1": () => jsonResponse(siteDetail()),
    });
    renderScreen(viewer);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    const detail = await screen.findByRole("complementary", { name: text.detail.label });
    // Upstream traversals: customer drill + contact-person search.
    expect(
      await within(detail).findByRole("button", { name: text.detail.filterByCustomer("대원강업") }),
    ).toBeVisible();
    expect(
      within(detail).getByRole("button", { name: text.detail.searchContact("이종호") }),
    ).toBeVisible();
    // Downstream: ticket, work order (dispatch drill), attendance, acceptances.
    expect(
      within(detail).getByRole("button", { name: text.ticket.open("경비 결원 대근 요청") }),
    ).toBeVisible();
    const workOrderLink = within(detail).getByRole("link", {
      name: text.workOrder.open("WO-2638"),
    });
    // The dispatch deep-link param the dispatch page actually parses.
    expect(workOrderLink).toHaveAttribute("href", "/dispatch?around_work_order_id=wo-1");
    expect(within(detail).getByText("김성호")).toBeVisible();
    const acceptSection = within(detail).getByRole("region", { name: text.detail.acceptances });
    expect(within(acceptSection).getByText(text.detail.sectionEmpty)).toBeVisible();
  });

  it("treats an out-of-scope site as absence, not an error", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
      // site-1 detail route intentionally missing → canonical 404 envelope.
    });
    renderScreen(viewer);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    const detail = await screen.findByRole("complementary", { name: text.detail.label });
    expect(await within(detail).findByRole("status")).toHaveTextContent(text.detail.absent);
    expect(within(detail).queryByRole("alert")).toBeNull();
  });

  it("records customer acceptance on a resolved ticket through the contract route", async () => {
    const acceptanceCalls: unknown[] = [];
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
      "/api/v1/field/sites/site-1": () => jsonResponse(siteDetail()),
      "/api/v1/support/tickets/ticket-1": () => jsonResponse(ticketDetail()),
      "/api/v1/support/tickets/ticket-1/acceptance": (request) => {
        acceptanceCalls.push(request);
        return jsonResponse(
          {
            id: "acc-1",
            ticket_id: "ticket-1",
            kind: "CUSTOMER_ACCEPTED",
            channel: "IN_PERSON",
            accepted_by: "이종호",
            note: null,
            recorded_by_user_id: "user-1",
            recorded_by_name: "정하늘",
            occurred_at: "2026-07-23T02:00:00Z",
          },
          201,
        );
      },
    });
    renderScreen(operator);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    await userEvent.click(
      await screen.findByRole("button", { name: text.ticket.open("경비 결원 대근 요청") }),
    );
    const form = await screen.findByRole("form", { name: text.detail.acceptances });
    await userEvent.type(
      within(form).getByLabelText(text.acceptance.acceptedBy),
      "이종호",
    );
    await userEvent.click(within(form).getByRole("button", { name: text.acceptance.record }));
    await waitFor(() => {
      expect(acceptanceCalls).toHaveLength(1);
    });
    const request = acceptanceCalls[0] as Request;
    expect(request.method).toBe("POST");
    // Business outcome: the form's collected answers reach the wire verbatim.
    expect(await request.json()).toMatchObject({
      kind: "CUSTOMER_ACCEPTED",
      channel: "IN_PERSON",
      accepted_by: "이종호",
    });
  });

  it("reconciles the list from the server after a mutation instead of stranding it in loading", async () => {
    let listCalls = 0;
    const comment = {
      id: "comment-1",
      ticket_id: "ticket-1",
      author_user_id: "user-1",
      author_name: "정하늘",
      body: "대근 편성 완료",
      is_internal_note: false,
      created_at: "2026-07-23T03:00:00Z",
    };
    const posted: (typeof comment)[] = [];
    stubRoutes({
      "/api/v1/field/sites": () => {
        listCalls += 1;
        return jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 });
      },
      "/api/v1/field/sites/site-1": () => jsonResponse(siteDetail()),
      "/api/v1/support/tickets/ticket-1": () =>
        jsonResponse(ticketDetail({ comments: [...posted] })),
      "/api/v1/support/tickets/ticket-1/comments": () => {
        posted.push(comment);
        return jsonResponse(comment, 201);
      },
    });
    renderScreen(operator);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    await userEvent.click(
      await screen.findByRole("button", { name: text.ticket.open("경비 결원 대근 요청") }),
    );
    const comments = await screen.findByRole("region", { name: ko.support.comments.title });
    await userEvent.type(within(comments).getByLabelText(ko.support.comments.title), "대근 편성 완료");
    await userEvent.click(within(comments).getByRole("button", { name: ko.support.comments.add }));
    // The refreshed ticket carries the comment and the list pane re-renders rows.
    expect(await within(comments).findByText("대근 편성 완료")).toBeVisible();
    await waitFor(() => {
      expect(screen.queryByText(text.loading)).toBeNull();
    });
    expect(listCalls).toBeGreaterThanOrEqual(2);
  });

  it("keeps a created ticket when site-linking fails instead of risking a duplicate intake", async () => {
    let createCalls = 0;
    const created = () =>
      ticketSummary({
        id: "ticket-9",
        status: "OPEN",
        title: "결원 보충",
        site_id: null,
        site_name: null,
        resolved_at: null,
      });
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
      "/api/v1/field/sites/site-1": () => jsonResponse(siteDetail()),
      "/api/v1/support/tickets": (request) => {
        if (request.method !== "POST") {
          return jsonResponse({ items: [], next_cursor: null, total: 0 });
        }
        createCalls += 1;
        return jsonResponse(created(), 201);
      },
      "/api/v1/support/tickets/ticket-9": () =>
        jsonResponse(ticketDetail({ ticket: created() })),
      "/api/v1/support/tickets/ticket-9/link": () =>
        jsonResponse({ error: { code: "conflict", message: "site link rejected" } }, 409),
    });
    renderScreen(operator);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    await screen.findByRole("heading", { name: "대원강업 상주" });
    await userEvent.click(screen.getByRole("button", { name: text.intake }));
    const intakeForm = screen.getByRole("form", { name: text.intake });
    await userEvent.type(within(intakeForm).getByLabelText(/제목/), "결원 보충");
    await userEvent.type(within(intakeForm).getByLabelText(/내용/), "야간 결원 대근 필요");
    await userEvent.click(within(intakeForm).getByRole("button", { name: text.intakeForm.submit }));
    // The link failure surfaces, but the created ticket is kept and offers the
    // manual link action — resubmitting the intake would duplicate the ticket.
    expect(await screen.findByRole("alert")).toHaveTextContent("site link rejected");
    expect(await screen.findByRole("button", { name: text.ticket.linkSite })).toBeVisible();
    expect(createCalls).toBe(1);
  });

  it("hides triage, acceptance, and comment affordances from a read-only viewer", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
      "/api/v1/field/sites/site-1": () => jsonResponse(siteDetail()),
      "/api/v1/support/tickets/ticket-1": () => jsonResponse(ticketDetail()),
    });
    renderScreen(viewer);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    await userEvent.click(
      await screen.findByRole("button", { name: text.ticket.open("경비 결원 대근 요청") }),
    );
    expect(await screen.findByRole("heading", { name: "경비 결원 대근 요청" })).toBeVisible();
    expect(screen.queryByRole("form", { name: text.detail.acceptances })).toBeNull();
    expect(screen.queryByRole("button", { name: text.ticket.linkSite })).toBeNull();
  });

  it("keeps the selection and the intake draft across a remount of the same session", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
      "/api/v1/field/sites/site-1": () => jsonResponse(siteDetail()),
    });
    const first = renderScreen(operator);
    await userEvent.click(await screen.findByText("대원강업 상주"));
    await screen.findByRole("heading", { name: "대원강업 상주" });
    await userEvent.click(screen.getByRole("button", { name: text.intake }));
    const intakeForm = screen.getByRole("form", { name: text.intake });
    await userEvent.type(within(intakeForm).getByLabelText(/제목/), "결원 보충 요청");
    first.unmount();

    renderScreen(operator);
    // Selection survives: the detail pane re-hydrates from the stored site id.
    expect(await screen.findByRole("heading", { name: "대원강업 상주" })).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.intake }));
    expect(screen.getByDisplayValue("결원 보충 요청")).toBeVisible();
  });

  it("marks the customer and load columns as narrow-collapsed so the list never scrolls sideways", async () => {
    stubRoutes({
      "/api/v1/field/sites": () =>
        jsonResponse({ items: [siteRow()], next_cursor: null, total: 1 }),
    });
    renderScreen(viewer);
    await screen.findByText("대원강업 상주");
    const wideCells = document.querySelectorAll(".field__cell--wide");
    // Header + data row each collapse the same two wide columns at narrow widths.
    expect(wideCells).toHaveLength(4);
  });
});
