import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { useAuth } = vi.hoisted(() => ({ useAuth: vi.fn() }));
vi.mock("../../context/auth", () => ({ useAuth }));

import { FacilitiesWorkflowPage } from "./FacilitiesWorkflowPage";

const caseId = "11111111-1111-1111-1111-111111111111";
const techId = "22222222-2222-2222-2222-222222222222";
const evidenceA = "33333333-3333-3333-3333-333333333333";
const evidenceB = "44444444-4444-4444-4444-444444444444";
const actorId = techId;

const allFacilitiesCapabilities = [
  "facilities_manage",
  "facilities_dispatch",
  "facilities_execute",
  "facilities_accept",
  "facilities_observe",
];

function authzResponse(features = allFacilitiesCapabilities) {
  return new Response(JSON.stringify({
    roles: ["OPERATOR"],
    branch_scope: { kind: "all" },
    capabilities: features.map((feature) => ({ feature, permission: "allow", branch_scope: { kind: "all" } })),
  }), { status: 200, headers: { "content-type": "application/json" } });
}

function caseView(status: string, overrides = {}) {
  return { id: caseId, branchId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", status, assigneeId: status === "DUE" || status === "SCHEDULED" ? null : techId, responseDueAt: "2030-01-01T09:00:00Z", completionDueAt: "2030-01-01T12:00:00Z", acceptanceDueAt: "2030-01-02T12:00:00Z", energyDeltaKwh: null, totalCostKrw: 0, ...overrides };
}

function setupApi(initial = "DUE", userId = actorId) {
  let status = initial;
  let value = caseView(status);
  const GET = vi.fn((path: string, options?: { signal?: AbortSignal }) => {
    void options;
    if (path === "/api/v1/facilities/cases") return Promise.resolve({ data: [value], response: new Response() });
    return Promise.resolve({ data: value, response: new Response() });
  });
  const POST = vi.fn((path: string, options?: { body?: Record<string, unknown> }) => {
    if (path.endsWith("/triage")) status = "SCHEDULED";
    else if (path.endsWith("/assign")) status = "ASSIGNED";
    else if (path.endsWith("/start")) status = "IN_PROGRESS";
    else if (path.endsWith("/observations")) value = caseView(status, { energyDeltaKwh: "-8.500", totalCostKrw: options?.body?.costKrw ?? 0 });
    else if (path.endsWith("/submit")) status = "AWAITING_ACCEPTANCE";
    else if (path.endsWith("/acceptance")) status = options?.body?.decision === "ACCEPTED" ? "CLOSED" : "REWORK_REQUIRED";
    value = { ...value, ...caseView(status), ...(path.endsWith("/observations") ? { energyDeltaKwh: "-8.500", totalCostKrw: options?.body?.costKrw ?? 0 } : {}) };
    return Promise.resolve({ data: path === "/api/v1/facilities/cases" ? value : {}, response: new Response() });
  });
  useAuth.mockReturnValue({ api: { GET, POST }, session: { access_token: "token", org_id: "org", user_id: userId, client_session_incarnation: "session" } });
  return { GET, POST };
}

describe("FacilitiesWorkflowPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse()));
  });
  afterEach(() => vi.unstubAllGlobals());

  it("executes the real lifecycle commands and renders only server-read state", async () => {
    const { POST } = setupApi();
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);
    await screen.findByRole("heading", { name: "접수 대기" });

    await user.clear(screen.getByLabelText("현장 예정 시각"));
    await user.type(screen.getByLabelText("현장 예정 시각"), "2030-01-01T10:00");
    await user.click(screen.getByRole("button", { name: "일정 확정" }));
    await screen.findByRole("heading", { name: "일정 확정" });

    await user.type(screen.getByLabelText("담당 사용자 ID"), techId);
    await user.click(screen.getByRole("button", { name: "담당 배정" }));
    await screen.findByRole("heading", { name: "담당 배정" });
    await user.click(screen.getByRole("button", { name: "작업 시작" }));
    await screen.findByRole("heading", { name: "작업 진행" });

    await user.type(screen.getByLabelText("작업 전 kWh"), "100.000");
    await user.type(screen.getByLabelText("작업 후 kWh"), "91.500");
    await user.type(screen.getByLabelText("비용 (KRW)"), "42000");
    await user.click(screen.getByRole("button", { name: "관측 기록" }));
    await screen.findByText("-8.500 kWh");
    expect(screen.getByText("42,000 KRW")).toBeInTheDocument();

    await user.type(screen.getByLabelText("안전 점검 증빙 ID"), evidenceA);
    await user.type(screen.getByLabelText("서비스 보고 증빙 ID"), evidenceB);
    await user.click(screen.getByRole("button", { name: "인수 요청 제출" }));
    await screen.findByRole("heading", { name: "인수 확인 대기" });
    await user.click(screen.getByRole("button", { name: "인수 및 종결" }));
    await screen.findByRole("heading", { name: "종결" });
    expect(screen.getByText("종결된 사례")).toBeInTheDocument();

    expect(POST.mock.calls.map(([path]) => path)).toEqual([
      "/api/v1/facilities/cases/{case_id}/triage",
      "/api/v1/facilities/cases/{case_id}/assign",
      "/api/v1/facilities/cases/{case_id}/start",
      "/api/v1/facilities/cases/{case_id}/observations",
      "/api/v1/facilities/cases/{case_id}/submit",
      "/api/v1/facilities/cases/{case_id}/acceptance",
    ]);
  });

  it("requires a rejected case to restart before observation and evidence submission", async () => {
    const { POST } = setupApi("AWAITING_ACCEPTANCE");
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);
    await screen.findByRole("heading", { name: "인수 확인 대기" });

    await user.type(screen.getByLabelText("반려 사유 (반려 시 기록)"), "현장 조정이 필요합니다");
    await user.click(screen.getByRole("button", { name: "재작업 요청" }));
    await screen.findByRole("heading", { name: "재작업 필요" });
    expect(screen.queryByRole("button", { name: "인수 요청 제출" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "작업 시작" }));
    await screen.findByRole("heading", { name: "작업 진행" });
    await user.type(screen.getByLabelText("작업 전 kWh"), "91.500");
    await user.type(screen.getByLabelText("작업 후 kWh"), "90.000");
    await user.click(screen.getByRole("button", { name: "관측 기록" }));
    await screen.findByText("-8.500 kWh");
    await user.type(screen.getByLabelText("안전 점검 증빙 ID"), evidenceA);
    await user.type(screen.getByLabelText("서비스 보고 증빙 ID"), evidenceB);
    await user.click(screen.getByRole("button", { name: "인수 요청 제출" }));
    await screen.findByRole("heading", { name: "인수 확인 대기" });

    expect(POST.mock.calls.map(([path]) => path)).toEqual([
      "/api/v1/facilities/cases/{case_id}/acceptance",
      "/api/v1/facilities/cases/{case_id}/start",
      "/api/v1/facilities/cases/{case_id}/observations",
      "/api/v1/facilities/cases/{case_id}/submit",
    ]);
  });

  it("keeps observation-only operators read-only except for observations", async () => {
    const { POST } = setupApi("IN_PROGRESS");
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse(["facilities_observe"])));
    render(<FacilitiesWorkflowPage />);

    expect(await screen.findByRole("button", { name: "관측 기록" })).toBeVisible();
    expect(screen.queryByRole("button", { name: "인수 요청 제출" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "사례 접수" })).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "작업 진행" })).toBeInTheDocument();
    expect(POST).not.toHaveBeenCalled();
  });

  it("omits execute controls for a non-assignee even with the execute capability", async () => {
    setupApi("ASSIGNED", "another-operator");
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(authzResponse(["facilities_execute"])));
    render(<FacilitiesWorkflowPage />);

    expect(await screen.findByRole("heading", { name: "담당 배정" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "작업 시작" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "인수 요청 제출" })).not.toBeInTheDocument();
  });

  it("marks the selected case with semantic pressed state", async () => {
    setupApi();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    expect(screen.getByRole("button", { name: `사례 ${caseId}` })).toHaveAttribute("aria-pressed", "true");
  });

  it("aborts a superseded authoritative list read before issuing a refresh", async () => {
    const { GET } = setupApi();
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    const initialListCall = GET.mock.calls.find(([path]) => path === "/api/v1/facilities/cases");
    expect(initialListCall?.[1]?.signal).toBeInstanceOf(AbortSignal);
    await user.click(screen.getByRole("button", { name: "새로 고침" }));
    await waitFor(() => {
      expect(GET.mock.calls.filter(([path]) => path === "/api/v1/facilities/cases")).toHaveLength(2);
    });
    expect(initialListCall?.[1]?.signal.aborted).toBe(true);
  });

  it("clears a successful create pending state and permits a second idempotent intake", async () => {
    const { POST } = setupApi();
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    const obligation = screen.getByLabelText("활성 HVAC 의무 ID");
    const create = screen.getByRole("button", { name: "사례 접수" });
    await user.type(obligation, "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    await user.click(create);
    await waitFor(() => {
      expect(POST).toHaveBeenCalledTimes(1);
      expect(create).toBeEnabled();
    });
    expect(obligation).toHaveValue("");

    await user.type(obligation, "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
    await user.click(create);
    await waitFor(() => {
      expect(POST).toHaveBeenCalledTimes(2);
    });
    expect(POST.mock.calls.map(([path]) => path)).toEqual([
      "/api/v1/facilities/cases",
      "/api/v1/facilities/cases",
    ]);
  });

  it("clears a canceled lifecycle command without applying its late result", async () => {
    const secondCaseId = "55555555-5555-5555-5555-555555555555";
    const first = caseView("DUE");
    const second = caseView("DUE", { id: secondCaseId });
    let resolveTriage: ((value: { data: object; response: Response }) => void) | undefined;
    const triage = new Promise<{ data: object; response: Response }>((resolve) => {
      resolveTriage = resolve;
    });
    const GET = vi.fn((path: string, options?: { params?: { path?: { case_id?: string } } }) => {
      if (path === "/api/v1/facilities/cases") return Promise.resolve({ data: [first, second], response: new Response() });
      return Promise.resolve({ data: options?.params?.path?.case_id === secondCaseId ? second : first, response: new Response() });
    });
    const POST = vi.fn((path: string) => path.endsWith("/triage") ? triage : Promise.resolve({ data: {}, response: new Response() }));
    useAuth.mockReturnValue({ api: { GET, POST }, session: { access_token: "token", org_id: "org", user_id: actorId, client_session_incarnation: "session" } });
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    const create = screen.getByRole("button", { name: "사례 접수" });
    await user.click(screen.getByRole("button", { name: "일정 확정" }));
    await waitFor(() => { expect(POST).toHaveBeenCalledTimes(1); });
    expect(create).toBeDisabled();

    await user.click(screen.getByRole("button", { name: `사례 ${secondCaseId}` }));
    await waitFor(() => { expect(screen.getByRole("button", { name: `사례 ${secondCaseId}` })).toHaveAttribute("aria-pressed", "true"); });
    expect(POST.mock.calls[0]?.[1]?.signal).toMatchObject({ aborted: true });
    expect(create).toBeDisabled();
    const readsBeforeLateResult = GET.mock.calls.length;

    resolveTriage?.({ data: {}, response: new Response() });
    await waitFor(() => { expect(create).toBeEnabled(); });
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(GET).toHaveBeenCalledTimes(readsBeforeLateResult);
    expect(screen.getByRole("button", { name: `사례 ${secondCaseId}` })).toHaveAttribute("aria-pressed", "true");
  });

  it("clears a canceled intake without selecting or clearing from its late result", async () => {
    const secondCaseId = "55555555-5555-5555-5555-555555555555";
    const createdCaseId = "66666666-6666-4666-8666-666666666666";
    const first = caseView("DUE");
    const second = caseView("DUE", { id: secondCaseId });
    let resolveCreate: ((value: { data: typeof first; response: Response }) => void) | undefined;
    const createResult = new Promise<{ data: typeof first; response: Response }>((resolve) => {
      resolveCreate = resolve;
    });
    const GET = vi.fn((path: string, options?: { params?: { path?: { case_id?: string } } }) => {
      if (path === "/api/v1/facilities/cases") return Promise.resolve({ data: [first, second], response: new Response() });
      return Promise.resolve({ data: options?.params?.path?.case_id === secondCaseId ? second : first, response: new Response() });
    });
    const POST = vi.fn(() => createResult);
    useAuth.mockReturnValue({ api: { GET, POST }, session: { access_token: "token", org_id: "org", user_id: actorId, client_session_incarnation: "session" } });
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    const obligation = screen.getByLabelText("활성 HVAC 의무 ID");
    const create = screen.getByRole("button", { name: "사례 접수" });
    await user.type(obligation, "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    await user.click(create);
    await waitFor(() => { expect(POST).toHaveBeenCalledTimes(1); });
    expect(create).toBeDisabled();

    await user.click(screen.getByRole("button", { name: `사례 ${secondCaseId}` }));
    await waitFor(() => { expect(screen.getByRole("button", { name: `사례 ${secondCaseId}` })).toHaveAttribute("aria-pressed", "true"); });
    expect(POST.mock.calls[0]?.[1]?.signal).toMatchObject({ aborted: true });
    const readsBeforeLateResult = GET.mock.calls.length;

    resolveCreate?.({ data: caseView("DUE", { id: createdCaseId }), response: new Response() });
    await waitFor(() => { expect(create).toBeEnabled(); });
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(obligation).toHaveValue("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    expect(GET).toHaveBeenCalledTimes(readsBeforeLateResult);
    expect(screen.getByRole("button", { name: `사례 ${secondCaseId}` })).toHaveAttribute("aria-pressed", "true");
  });

  it("clears the old detail and cannot post to it while a new case detail is pending", async () => {
    const secondCaseId = "55555555-5555-5555-5555-555555555555";
    const first = caseView("DUE");
    const second = caseView("DUE", { id: secondCaseId });
    let resolveSecondDetail: (() => void) | undefined;
    const secondDetail = new Promise<{ data: typeof second; response: Response }>((resolve) => {
      resolveSecondDetail = () => { resolve({ data: second, response: new Response() }); };
    });
    const GET = vi.fn((path: string, options?: { params?: { path?: { case_id?: string } } }) => {
      if (path === "/api/v1/facilities/cases") return Promise.resolve({ data: [first, second], response: new Response() });
      return options?.params?.path?.case_id === secondCaseId
        ? secondDetail
        : Promise.resolve({ data: first, response: new Response() });
    });
    const POST = vi.fn().mockResolvedValue({ data: {}, response: new Response() });
    useAuth.mockReturnValue({ api: { GET, POST }, session: { access_token: "token", org_id: "org", user_id: actorId, client_session_incarnation: "session" } });
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    await user.click(screen.getByRole("button", { name: `사례 ${secondCaseId}` }));
    expect(screen.queryByRole("button", { name: "일정 확정" })).not.toBeInTheDocument();
    expect(POST).not.toHaveBeenCalled();

    resolveSecondDetail?.();
    await screen.findByRole("heading", { name: "접수 대기" });
    await user.click(screen.getByRole("button", { name: "일정 확정" }));
    await waitFor(() => {
      expect(POST).toHaveBeenCalledTimes(1);
    });
    expect(POST).toHaveBeenCalledWith(
      "/api/v1/facilities/cases/{case_id}/triage",
      expect.objectContaining({ params: { path: { case_id: secondCaseId } } }),
    );
  });

  it("fails closed when a lifecycle response omits data and does not reread", async () => {
    const { GET, POST } = setupApi();
    POST.mockResolvedValueOnce({ response: new Response() });
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);

    await screen.findByRole("heading", { name: "접수 대기" });
    const readsBefore = GET.mock.calls.length;
    await user.click(screen.getByRole("button", { name: "일정 확정" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("서버가 요청을 처리하지 못했습니다.");
    expect(GET).toHaveBeenCalledTimes(readsBefore);
    expect(screen.getByRole("heading", { name: "접수 대기" })).toBeInTheDocument();
  });

  it("does not send a submission without both mandatory evidence records", async () => {
    const { POST } = setupApi("IN_PROGRESS");
    render(<FacilitiesWorkflowPage />);
    await screen.findByRole("heading", { name: "작업 진행" });
    const submitForm = screen.getByRole("button", { name: "인수 요청 제출" }).closest("form");
    if (!submitForm) throw new Error("submission form is missing");
    fireEvent.submit(submitForm);
    expect(await screen.findByRole("alert")).toHaveTextContent("안전 점검과 서비스 보고 증빙 ID가 모두 필요합니다.");
    expect(POST).not.toHaveBeenCalled();
  });

  it("renders the backend failure rather than inventing a transition", async () => {
    const { POST } = setupApi();
    POST.mockResolvedValueOnce({ error: new Error("illegal transition"), response: new Response("", { status: 409 }) });
    const user = userEvent.setup();
    render(<FacilitiesWorkflowPage />);
    await screen.findByRole("heading", { name: "접수 대기" });
    await user.click(screen.getByRole("button", { name: "일정 확정" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("illegal transition");
    expect(screen.getByRole("heading", { name: "접수 대기" })).toBeInTheDocument();
  });
});
