import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { payrollStrings as text } from "../../i18n/payroll";
import type {
  PayrollException,
  PayrollLineSummary,
  PayrollRunDetail,
  PayrollRunStatus,
  PayrollRunSummary,
  RunCalcSummary,
} from "./payrollApi";
import type { PayrollCapabilities } from "./payrollCapabilities";
import { PayrollScreen } from "./PayrollScreen";

const navigateSpy = vi.fn();
vi.mock("react-router", () => ({
  useNavigate: () => navigateSpy,
}));

const manager: PayrollCapabilities = { canRead: true, canManage: true, canDecide: true, canReadSelf: true };
const reader: PayrollCapabilities = { canRead: true, canManage: false, canDecide: false, canReadSelf: true };
const denied: PayrollCapabilities = { canRead: false, canManage: false, canDecide: false, canReadSelf: true };

function runSummary(status: PayrollRunStatus = "STAGED", id = "run-1"): PayrollRunSummary {
  return {
    id,
    period_start: "2026-06-01",
    period_end: "2026-06-30",
    source_label: "7월 정기",
    status,
    calculation_enabled: true,
    created_at: "2026-07-01T00:00:00Z",
    updated_at: "2026-07-01T00:00:00Z",
  };
}

function line(overrides: Partial<PayrollLineSummary> = {}): PayrollLineSummary {
  return {
    id: "line-1",
    employee_id: "emp-1",
    employee_display_name: "김성아",
    employee_company: "코스콕",
    work_days: 21,
    overtime_hours: 12,
    gross_pay_source_present: true,
    net_pay_source_present: true,
    nts_tax_row_status: "VERIFIED_SOURCE_ROW",
    calculation_status: "READY_FOR_REVIEW",
    blockers: [],
    ...overrides,
  };
}

function calcSummary(overrides: Partial<RunCalcSummary> = {}): RunCalcSummary {
  return {
    version: 1,
    calculated_at: "2026-07-03T09:00:00Z",
    calculated_lines: 2,
    blocked_lines: 0,
    payable: false,
    kernel_rate_table: "RT-2026-07",
    total_net_won: null,
    ...overrides,
  };
}

function exception(overrides: Partial<PayrollException> = {}): PayrollException {
  return {
    id: "ex-1",
    run_id: "run-1",
    employee_id: "emp-1",
    employee_display_name: "김성아",
    kind: "OVERTIME_ALLOWANCE",
    severity: "warn",
    amount_delta_won: 182000,
    summary_ko: "연장 12시간 승인분 반영",
    linked_refs: [{ kind: "attendance", code: "AT-1042" }],
    status: "OPEN",
    ...overrides,
  };
}

function runDetail(status: PayrollRunStatus, overrides: Partial<PayrollRunDetail> = {}): PayrollRunDetail {
  return {
    run: runSummary(status),
    legal_basis: {},
    source_summary: {},
    lines: [line()],
    lines_total: 1,
    lines_limit: 50,
    lines_offset: 0,
    exceptions_open: 0,
    exceptions_total: 0,
    calculation: null,
    disbursement: null,
    payslip_delivery: null,
    ...overrides,
  };
}

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function client() {
  return { GET: vi.fn(), POST: vi.fn() } as unknown as ConsoleApiClient;
}

/** Route the three read endpoints of the mocked client from one state object. */
function wire(api: ConsoleApiClient, state: { detail: PayrollRunDetail; exceptions: PayrollException[] }) {
  vi.mocked(api.GET).mockImplementation((url: string) => {
    if (url === "/api/v1/payroll/runs") {
      return Promise.resolve(ok({ items: [state.detail.run], total: 1, limit: 50, offset: 0 })) as never;
    }
    if (url.endsWith("/exceptions")) {
      return Promise.resolve(ok({ items: state.exceptions, total: state.exceptions.length, limit: 50, offset: 0 })) as never;
    }
    if (url.endsWith("/close-preflight")) {
      return Promise.resolve(ok({
        checks: [{ key: "attendance", label_ko: "근태 예외 0건", ok: true, warn: false, blocking_refs: [] }],
        can_close: true,
      })) as never;
    }
    return Promise.resolve(ok(state.detail)) as never;
  });
}

function renderScreen(capabilities: PayrollCapabilities, api = client()) {
  return render(
    <PayrollScreen api={api} branchId="branch-1" actorId="actor-1" capabilities={capabilities} sessionKey="session-a" />,
  );
}

beforeEach(() => {
  window.localStorage.clear();
  navigateSpy.mockReset();
});

describe("PayrollScreen", () => {
  it("denies an unauthorized user before fetching or exposing actions", () => {
    const api = client();
    renderScreen(denied, api);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(screen.queryByRole("button")).toBeNull();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("retries an initial error and renders the backend run list", async () => {
    const api = client();
    const state = { detail: runDetail("STAGED"), exceptions: [] };
    vi.mocked(api.GET).mockResolvedValueOnce(ok(undefined));
    renderScreen(manager, api);
    expect(await screen.findByRole("alert")).toHaveTextContent("Payroll request failed (200)");
    wire(api, state);
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await screen.findByRole("button", { name: /7월 정기/ })).toBeVisible();
  });

  it("shows the truthful empty state when the org has no runs", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(ok({ items: [], total: 0, limit: 50, offset: 0 }));
    renderScreen(manager, api);
    expect(await screen.findByText(text.emptyRuns)).toBeVisible();
  });

  it("gates the roster before calculation and walks the attested close preflight", async () => {
    const api = client();
    const state = { detail: runDetail("STAGED"), exceptions: [] };
    wire(api, state);
    vi.mocked(api.POST).mockImplementation(() => {
      state.detail = runDetail("ATTENDANCE_CLOSED");
      return Promise.resolve(ok(state.detail)) as never;
    });
    renderScreen(manager, api);
    expect(await screen.findByText(text.rosterGateStaged)).toBeVisible();
    expect(screen.queryByText(text.colName)).toBeNull();

    await userEvent.click(screen.getAllByRole("button", { name: text.ctaClose })[0]);
    const dialog = await screen.findByRole("dialog", { name: text.preflightTitle });
    expect(dialog).toBeVisible();
    expect(await screen.findByText("근태 예외 0건")).toBeVisible();
    const confirm = screen.getByRole("button", { name: text.preflightConfirm });
    expect(confirm).toBeDisabled();
    await userEvent.click(screen.getByRole("checkbox", { name: text.preflightAttest }));
    expect(confirm).toBeEnabled();
    await userEvent.click(confirm);
    expect(api.POST).toHaveBeenCalledWith("/api/v1/payroll/runs/{id}/close-attendance", expect.objectContaining({
      params: { path: { id: "run-1" } },
      body: { attest: true },
    }));
    const calcButtons = await screen.findAllByRole("button", { name: text.ctaCalc });
    expect(calcButtons.length).toBeGreaterThan(0);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("runs calculation from the attendance-closed state", async () => {
    const api = client();
    const state = { detail: runDetail("ATTENDANCE_CLOSED"), exceptions: [] };
    wire(api, state);
    vi.mocked(api.POST).mockImplementation(() => {
      state.detail = runDetail("CALCULATED", { calculation: calcSummary(), exceptions_open: 1, exceptions_total: 1 });
      state.exceptions = [exception()];
      return Promise.resolve(ok(state.detail)) as never;
    });
    renderScreen(manager, api);
    await userEvent.click((await screen.findAllByRole("button", { name: text.ctaCalc }))[0]);
    expect(api.POST).toHaveBeenCalledWith("/api/v1/payroll/runs/{id}/calculate", expect.objectContaining({
      params: { path: { id: "run-1" } },
    }));
    expect(await screen.findByText(text.chipExceptionsLeft(1))).toBeVisible();
    expect(screen.getByText(text.colName)).toBeVisible();
  });

  it("blocks submission on open exceptions and unlocks it after a resolve", async () => {
    const api = client();
    const state = {
      detail: runDetail("CALCULATED", { calculation: calcSummary(), exceptions_open: 1, exceptions_total: 1 }),
      exceptions: [exception()],
    };
    wire(api, state);
    vi.mocked(api.POST).mockImplementation(() => {
      state.detail = runDetail("CALCULATED", { calculation: calcSummary(), exceptions_open: 0, exceptions_total: 1 });
      state.exceptions = [exception({ status: "CONFIRMED" })];
      return Promise.resolve(ok(state.exceptions[0])) as never;
    });
    renderScreen(manager, api);
    expect(await screen.findByText(text.chipExceptionsLeft(1))).toBeVisible();
    expect(screen.queryByRole("button", { name: text.ctaSubmit })).toBeNull();

    await userEvent.click(screen.getByRole("button", { name: text.exConfirm }));
    expect(api.POST).toHaveBeenCalledWith("/api/v1/payroll/runs/{id}/exceptions/{exceptionId}/resolve", expect.objectContaining({
      params: { path: { id: "run-1", exceptionId: "ex-1" } },
      body: { action: "CONFIRM" },
    }));
    expect(await screen.findByText(text.exResolvedOk)).toBeVisible();
    expect(await screen.findByRole("button", { name: text.ctaSubmit })).toBeVisible();
  });

  it("masks pay amounts by default and reveals them only on request", async () => {
    const api = client();
    wire(api, {
      detail: runDetail("CALCULATED", {
        calculation: calcSummary({ total_net_won: 4180000000 }),
        exceptions_open: 1,
        exceptions_total: 1,
      }),
      exceptions: [exception()],
    });
    renderScreen(manager, api);
    expect(await screen.findAllByText(text.masked)).toHaveLength(2);
    expect(screen.queryByText("+₩182,000")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: text.maskShow }));
    expect(screen.getByText("+₩182,000")).toBeVisible();
    expect(screen.getByText("₩4,180,000,000")).toBeVisible();
    expect(screen.queryByText(text.masked)).toBeNull();
  });

  it("moves the roster cursor with J/K and opens the person drill with Enter", async () => {
    const api = client();
    wire(api, {
      detail: runDetail("CALCULATED", {
        calculation: calcSummary(),
        lines: [line(), line({ id: "line-2", employee_id: "emp-2", employee_display_name: "전성진" })],
        lines_total: 2,
      }),
      exceptions: [],
    });
    renderScreen(manager, api);
    const rows = await screen.findAllByRole("button", { expanded: false });
    const first = rows.find((row) => row.textContent.includes("김성아"));
    expect(first).toBeDefined();
    first?.focus();
    await userEvent.keyboard("j");
    expect(document.activeElement?.textContent).toContain("전성진");
    await userEvent.keyboard("k");
    expect(document.activeElement?.textContent).toContain("김성아");
    await userEvent.keyboard("{Enter}");
    expect(await screen.findByRole("button", { name: text.personCard })).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.personCard }));
    expect(navigateSpy).toHaveBeenCalledWith("/people");
  });

  it("surfaces flagged rows first and links exception refs to their objects", async () => {
    const api = client();
    wire(api, {
      detail: runDetail("CALCULATED", {
        calculation: calcSummary(),
        exceptions_open: 1,
        exceptions_total: 1,
        lines: [
          line({ id: "line-2", employee_id: "emp-2", employee_display_name: "전성진" }),
          line(),
        ],
        lines_total: 2,
      }),
      exceptions: [exception()],
    });
    renderScreen(manager, api);
    const list = await screen.findByRole("list", { name: text.rosterTitle });
    const names = [...list.querySelectorAll(".payroll__who")].map((node) => node.textContent);
    expect(names[0]).toContain("김성아");

    const flagChip = screen.getAllByRole("button", { name: /연장수당/ })
      .find((node) => node.tagName === "BUTTON" && node.className.includes("payroll__chip"));
    if (!flagChip) throw new Error("roster flag chip missing");
    await userEvent.click(flagChip);
    await userEvent.click(await screen.findByRole("button", { name: /AT-1042/ }));
    expect(navigateSpy).toHaveBeenCalledWith("/attendance");
  });

  it("offers no mutation affordances to a read-only capability", async () => {
    const api = client();
    wire(api, {
      detail: runDetail("CALCULATED", { calculation: calcSummary(), exceptions_open: 1, exceptions_total: 1 }),
      exceptions: [exception()],
    });
    renderScreen(reader, api);
    expect(await screen.findByText(text.colName)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.exConfirm })).toBeNull();
    expect(screen.queryByRole("button", { name: text.ctaSubmit })).toBeNull();
    expect(screen.queryByRole("button", { name: text.ctaCalc })).toBeNull();
  });

  it("walks the rejected run back through withdraw", async () => {
    const api = client();
    const state = { detail: runDetail("REJECTED", { calculation: calcSummary(), exceptions_total: 1 }), exceptions: [exception({ status: "CONFIRMED" })] };
    wire(api, state);
    vi.mocked(api.POST).mockImplementation(() => {
      state.detail = runDetail("CALCULATED", { calculation: calcSummary(), exceptions_total: 1 });
      return Promise.resolve(ok(state.detail)) as never;
    });
    renderScreen(manager, api);
    expect(await screen.findByText(text.chipRejected)).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.ctaWithdraw }));
    expect(api.POST).toHaveBeenCalledWith("/api/v1/payroll/runs/{id}/withdraw", expect.anything());
    expect(await screen.findByRole("button", { name: text.ctaSubmit })).toBeVisible();
  });

  it("reads the paid run forward into payslip issuance and delivery counts", async () => {
    const api = client();
    const state = {
      detail: runDetail("PAID", {
        calculation: calcSummary({ payable: true, total_net_won: 4180000000 }),
        disbursement: { id: "d-1", run_id: "run-1", scheduled_at: "2026-07-10T04:00:00Z", status: "PAID" },
      }),
      exceptions: [] as PayrollException[],
    };
    wire(api, state);
    vi.mocked(api.POST).mockImplementation(() => {
      state.detail = runDetail("ISSUED", {
        calculation: calcSummary({ payable: true, total_net_won: 4180000000 }),
        disbursement: { id: "d-1", run_id: "run-1", scheduled_at: "2026-07-10T04:00:00Z", status: "PAID" },
        payslip_delivery: { run_id: "run-1", issued: 1, acknowledged: 0, items: [], limit: 50, offset: 0, total: 1 },
      });
      return Promise.resolve(ok(state.detail.payslip_delivery)) as never;
    });
    renderScreen(manager, api);
    await userEvent.click(await screen.findByRole("button", { name: text.ctaIssue }));
    expect(api.POST).toHaveBeenCalledWith("/api/v1/payroll/runs/{id}/issue-payslips", expect.anything());
    expect((await screen.findAllByText(text.chipIssued)).length).toBeGreaterThan(0);
    expect(screen.getByText(text.deliveryIssued(1))).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.inboxLink }));
    expect(navigateSpy).toHaveBeenCalledWith("/inbox");
  });

  it("keeps the selected run across a remount from the personal view", async () => {
    const api = client();
    const past = runSummary("ISSUED", "run-0");
    const current = runSummary("STAGED", "run-1");
    vi.mocked(api.GET).mockImplementation((url: string, init?: { params?: { path?: { id?: string } } }) => {
      if (url === "/api/v1/payroll/runs") {
        return Promise.resolve(ok({ items: [current, past], total: 2, limit: 50, offset: 0 })) as never;
      }
      if (url.endsWith("/exceptions")) {
        return Promise.resolve(ok({ items: [], total: 0, limit: 50, offset: 0 })) as never;
      }
      const id = init?.params?.path?.id;
      return Promise.resolve(ok(runDetail(id === "run-0" ? "ISSUED" : "STAGED", {
        run: id === "run-0" ? past : current,
      }))) as never;
    });
    const view = renderScreen(manager, api);
    await userEvent.click(await screen.findByRole("button", { name: /ISSUED|명세서 발송됨/ }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /명세서 발송됨/ })).toHaveAttribute("aria-pressed", "true");
    });
    view.unmount();

    renderScreen(manager, api);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /명세서 발송됨/ })).toHaveAttribute("aria-pressed", "true");
    });
  });
});
