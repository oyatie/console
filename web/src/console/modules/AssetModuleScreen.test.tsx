import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { createConsoleApiClient } from "../../api/client";
import { AuthContext, type AuthContextValue } from "../../context/auth";
import { AssetWorkspace } from "../asset/AssetWorkspace";
import { AssetModuleScreen } from "./AssetModuleScreen";

const equipmentId = "00000000-0000-4000-8000-000000000001";
const consoleTokens = readFileSync(resolve(process.cwd(), "src/console/tokens.css"), "utf8");
const assetWorkspaceStyles = readFileSync(resolve(process.cwd(), "src/console/asset/assetWorkspace.css"), "utf8");

function declarationBlock(css: string, selector: string): string {
  const selectorStart = css.indexOf(selector);
  const blockStart = css.indexOf("{", selectorStart);
  const blockEnd = css.indexOf("\n}", blockStart);
  if (selectorStart < 0 || blockStart < 0 || blockEnd < 0) throw new Error(`Missing ${selector} token block`);
  return css.slice(blockStart + 1, blockEnd);
}

function token(block: string, name: string): string {
  const value = block.match(new RegExp(`${name}:\\s*(#[0-9a-f]{6})`, "i"))?.[1];
  if (!value) throw new Error(`Missing ${name} token`);
  return value;
}

function contrastRatio(foreground: string, background: string): number {
  const luminance = (hex: string) => {
    const channels = hex.match(/[0-9a-f]{2}/gi)?.map((channel) => Number.parseInt(channel, 16) / 255);
    if (!channels || channels.length !== 3) throw new Error(`Invalid color ${hex}`);
    const [red, green, blue] = channels.map((channel) => channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4);
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
  };
  const [lighter, darker] = [luminance(foreground), luminance(background)].sort((left, right) => right - left);
  return (lighter + 0.05) / (darker + 0.05);
}

const row = {
  equipment_id: equipmentId,
  branch_id: "00000000-0000-4000-8000-000000000002",
  equipment_no: "EQ-900",
  management_no: "MG-77",
  status: "rented",
  model: "ZX-9",
  maker: "MakerOne",
  specification: "3단 마스트",
  ton_text: "3.0t",
  customer_name: "고객 A",
  site_name: "서울 센터",
  asset_owner: "케이엔엘",
  vin: "VIN-900",
  updated_at: "2026-07-09T12:30:00Z",
} as const;

function apiForTest() {
  const api = createConsoleApiClient("asset-test-token");
  const get = vi.spyOn(api, "GET").mockImplementation(async (path: string) => {
    await Promise.resolve();
    if (path === "/api/v1/equipment/list") return { data: { items: [row], total: 1, limit: 50, offset: 0 } };
    if (path === "/api/v1/equipment/{id}/timeline-graph") return { data: { equipment: { ...row, customer_id: "customer-1", site_id: "site-1" }, lifecycle_events: [{ id: "event-1", kind: "maintenance", label: "정비 완료", description: "정기 점검", event_date: "2026-07-08", occurred_at: null, href: "/work-orders/wo-1" }], graph: { nodes: [{ id: "equipment", node_type: "equipment", label: "EQ-900", subtitle: null, href: null, current: true }, { id: "customer", node_type: "customer", label: "고객 A", subtitle: null, href: "/customers/customer-1", current: false }], edges: [{ from: "equipment", to: "customer", kind: "assigned", label: "배치" }] }, work_order_count: 1, cost_ledger_total_won: 120000 } };
    if (path === "/api/v1/equipment/{id}/versions") return { data: { items: [{ version: 2, status: "CAPTURED", content: {}, createdAt: "2026-07-09T12:30:00Z" }] } };
    if (path === "/api/v1/equipment/{id}/substitutes") return { data: { items: [{ equipment_id: "candidate-1", branch_id: row.branch_id, equipment_no: "EQ-901", management_no: null, model: "ZX-9", status: "spare", specification: "3단 마스트", ton_text: "3.0t", ton_milli: 3000, power_code: "E", power_label: "전동", customer_name: "", site_name: "", placement_location: null, match_kind: "EXACT_TON", ton_delta_milli: 0 }], total: 1 } };
    if (path === "/api/v1/equipment/{id}/ownership-transfer-requests") return { data: { items: [] } };
    if (path === "/api/v1/financial/equipment/{equipmentId}/lifecycle-cost") return { data: { equipment_id: equipmentId, equipment_no: "EQ-900", status: "rented", acquisition_cost_won: 5000000, acquisition_date: null, acquisition_source: "EXPLICIT", maintenance_total_won: 120000, manual_total_won: 0, purchase_total_won: 0, entry_count: 1, outsource_unlinked_won: 0, residual_value_won: 4880000, sale_price_won: null, sold_at: null, gross_margin_won: null, tco_won: 5120000, cost_per_month_won: 320000, cost_per_hour_won: 12000, timeline: [] } };
    throw new Error(`Unexpected GET: ${path}`);
  });
  const post = vi.spyOn(api, "POST").mockImplementation(async (path: string) => {
    await Promise.resolve();
    if (path === "/api/v1/equipment/{id}/versions/{version}/rollback") return { data: { version: 3 } };
    throw new Error(`Unexpected POST: ${path}`);
  });
  return { api, get, post };
}

describe("AssetWorkspace", () => {
  it("keeps primary-action contrast at WCAG AA in light and dark console themes", () => {
    const primaryButton = declarationBlock(assetWorkspaceStyles, ".asset-workspace__button");
    expect(primaryButton).toContain("background: var(--teal);");
    expect(primaryButton).toContain("color: var(--surface);");

    const light = declarationBlock(consoleTokens, ".console {");
    const dark = declarationBlock(consoleTokens, ".console[data-console-theme=\"dark\"],");
    for (const theme of [light, dark]) {
      expect(contrastRatio(token(theme, "--surface"), token(theme, "--teal"))).toBeGreaterThanOrEqual(4.5);
    }
  });

  it("never mounts an old authorized asset workspace after the session changes", async () => {
    let resolveOld!: (value: unknown) => void;
    const oldAuthz = new Promise<unknown>((resolve) => { resolveOld = resolve; });
    const api = createConsoleApiClient("asset-test-token");
    let authzCalls = 0;
    vi.spyOn(api, "GET").mockImplementation(((path: string) => {
      if (path === "/api/v1/me/authz") {
        authzCalls += 1;
        return authzCalls === 1 ? oldAuthz : new Promise(() => undefined);
      }
      return Promise.resolve({ data: { items: [row], total: 1, limit: 50, offset: 0 } });
    }) as never);
    const authValue = (token: string, incarnation: string) => ({
      api, session: { access_token: token, client_session_incarnation: incarnation, branches: [row.branch_id] }, restoring: false,
      login: vi.fn(), logout: vi.fn(), refresh: vi.fn(), acceptTokens: vi.fn(), clearPasskeySetup: vi.fn(), viewAs: undefined, enterViewAs: vi.fn(), exitViewAs: vi.fn(),
    }) as unknown as AuthContextValue;
    const view = render(<AuthContext.Provider value={authValue("old", "old-session")}><AssetModuleScreen /></AuthContext.Provider>);
    await waitFor(() => { expect(authzCalls).toBe(1); });
    view.rerender(<AuthContext.Provider value={authValue("new", "new-session")}><AssetModuleScreen /></AuthContext.Provider>);
    await waitFor(() => { expect(authzCalls).toBe(2); });
    resolveOld({ capabilities: [{ feature: "work_order_read_all", permission: "allow", branch_scope: { kind: "all" } }] });
    expect(await screen.findByRole("status")).toHaveTextContent("권한을 확인하는 중입니다.");
    expect(screen.queryByRole("heading", { name: "자산" })).not.toBeInTheDocument();
  });

  it("discards a stale list response after a session switch", async () => {
    let resolveFirst!: (value: unknown) => void;
    const first = new Promise<unknown>((resolve) => { resolveFirst = resolve; });
    const api = createConsoleApiClient("asset-test-token");
    let calls = 0;
    const get = vi.spyOn(api, "GET").mockImplementation((() => {
      calls += 1;
      return calls === 1 ? first : new Promise(() => undefined);
    }) as never);
    const props = { api, capabilities: { canRead: true, canManage: () => false, canReadCost: () => false, canImport: () => false } };
    const view = render(<AssetWorkspace {...props} sessionKey="session-a" />);
    await waitFor(() => { expect(get).toHaveBeenCalledTimes(1); });
    view.rerender(<AssetWorkspace {...props} sessionKey="session-b" />);
    await waitFor(() => { expect(get).toHaveBeenCalledTimes(2); });
    resolveFirst({ data: { items: [row], total: 1, limit: 50, offset: 0 } });
    await waitFor(() => { expect(screen.queryByText("EQ-900")).not.toBeInTheDocument(); });
  });

  it("renders source-backed lifecycle, graph, versions and cost, then records rollback through the generated operation", async () => {
    const { api, get, post } = apiForTest();
    render(<AssetWorkspace api={api} sessionKey="session-a" capabilities={{ canRead: true, canManage: () => true, canReadCost: () => true, canImport: () => true }} />);

    fireEvent.click(await screen.findByRole("button", { name: "EQ-900 상세 열기" }));

    expect(await screen.findByText("정비 완료")).toBeVisible();
    expect(screen.getByRole("link", { name: "고객 A" })).toHaveAttribute("href", "/customers/customer-1");
    expect(screen.getByText("생애주기 비용")).toBeVisible();
    expect(screen.getByText("대차")).toBeVisible();
    expect(screen.getByText("이 화면에서는 현재 세션에서 만든 대차만 반납할 수 있습니다. 새로고침 또는 세션 변경 뒤에는 이 화면에서 해당 대차를 확인하거나 반납할 수 없습니다.")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "되돌림" }));

    await waitFor(() => {
      expect(post).toHaveBeenCalledWith(
        "/api/v1/equipment/{id}/versions/{version}/rollback",
        expect.objectContaining({ params: { path: { id: equipmentId, version: 2 } } }),
      );
    });
    expect(get).toHaveBeenCalledWith("/api/v1/equipment/{id}/timeline-graph", expect.anything());
    expect(await screen.findByText("버전 3으로 되돌림 이력을 추가했습니다.")).toBeVisible();
  });

  it("denies management requests for an unresolved/denied capability", async () => {
    const { api, get } = apiForTest();
    render(<AssetWorkspace api={api} sessionKey="session-a" capabilities={{ canRead: true, canManage: () => false, canReadCost: () => false, canImport: () => false }} />);

    fireEvent.click(await screen.findByRole("button", { name: "EQ-900 상세 열기" }));
    expect(await screen.findByText("정비 완료")).toBeVisible();
    expect(screen.queryByText("대차")).not.toBeInTheDocument();
    expect(screen.queryByText("생애주기 비용")).not.toBeInTheDocument();
    const paths = get.mock.calls.map(([path]) => {
      return path;
    });
    expect(paths).not.toContain("/api/v1/equipment/{id}/substitutes");
    expect(paths).not.toContain("/api/v1/financial/equipment/{equipmentId}/lifecycle-cost");
  });

  it("omits master-list import when an executive has manage but not import capability", async () => {
    const { api } = apiForTest();
    render(<AssetWorkspace api={api} sessionKey="session-a" capabilities={{ canRead: true, canManage: () => true, canReadCost: () => true, canImport: () => false }} />);

    fireEvent.click(await screen.findByRole("button", { name: "EQ-900 상세 열기" }));
    expect(await screen.findByText("대차")).toBeVisible();
    expect(screen.getByRole("button", { name: "장비 등록" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "엑셀 가져오기" })).not.toBeInTheDocument();
  });

  it("surfaces an explicit generated-client error envelope with retry", async () => {
    const api = createConsoleApiClient("asset-test-token");
    vi.spyOn(api, "GET").mockImplementation(async () => {
      await Promise.resolve();
      return { error: { message: "권한이 없습니다." } };
    });
    render(<AssetWorkspace api={api} sessionKey="session-a" capabilities={{ canRead: true, canManage: () => false, canReadCost: () => false, canImport: () => false }} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("권한이 없습니다.");
    expect(screen.getByRole("button", { name: "다시 시도" })).toBeVisible();
  });
});
