import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { evaluationStrings as text } from "../../i18n/evaluation";
import type {
  EvaluationCycleDetail,
  EvaluationCycleSummary,
  EvaluationPreflightReport,
  EvaluationSubjectDetail,
  EvaluationTaskSummary,
} from "./evaluationApi";
import type { EvaluationCapabilities } from "./evaluationCapabilities";
import { EvaluationScreen } from "./EvaluationScreen";

const manage: EvaluationCapabilities = {
  canRead: true,
  canManage: true,
  canSubmit: true,
  canCalibrate: true,
};
const readOnly: EvaluationCapabilities = {
  canRead: true,
  canManage: false,
  canSubmit: false,
  canCalibrate: false,
};
const submitter: EvaluationCapabilities = {
  canRead: true,
  canManage: false,
  canSubmit: true,
  canCalibrate: false,
};
const submitOnly: EvaluationCapabilities = {
  canRead: false,
  canManage: false,
  canSubmit: true,
  canCalibrate: false,
};
const denied: EvaluationCapabilities = {
  canRead: false,
  canManage: false,
  canSubmit: false,
  canCalibrate: false,
};

const cycle = (stage: EvaluationCycleSummary["stage"] = "OPEN"): EvaluationCycleSummary => ({
  id: "cycle-1",
  name: "2026 상반기 정기평가",
  kind: "REGULAR",
  period_label: "2026 H1",
  due_date: "2099-07-18",
  stage,
  subjects_total: 2,
  manager_submitted: 1,
  self_submitted: 1,
  calibrated: 0,
  finalized: 0,
  created_at: "2026-07-01T00:00:00Z",
});

const detail = (stage: EvaluationCycleSummary["stage"] = "OPEN"): EvaluationCycleDetail => ({
  ...cycle(stage),
  opened_at: null,
  calibration_started_at: null,
  finalized_at: null,
  archived_at: null,
  created_by: "user-admin",
  progress_by_unit: [{ org_unit: "정비사업팀", total: 4, manager_submitted: 2 }],
  subjects: [
    {
      id: "subject-1",
      cycle_id: "cycle-1",
      employee_id: "emp-1",
      employee_name: "조이슨",
      org_unit: "정비사업팀",
      manager_user_id: "user-mgr",
      state: "IN_REVIEW",
      final_grade: null,
      rv_code: null,
    },
  ],
});

const preflight = (): EvaluationPreflightReport => ({
  next_transition: "start_calibration",
  blockers: [],
  advisories: [],
});

const blockedPreflight = (
  nextTransition: EvaluationPreflightReport["next_transition"],
): EvaluationPreflightReport => ({
  next_transition: nextTransition,
  blockers: [
    {
      code: "MISSING_GOALS",
      message: "목표를 저장해야 다음 단계로 진행할 수 있습니다.",
      subject_id: "subject-1",
    },
  ],
  advisories: [],
});

const task = (): EvaluationTaskSummary => ({
  subject_id: "subject-1",
  cycle_id: "cycle-1",
  cycle_name: "2026 상반기 정기평가",
  due_date: "2099-07-18",
  kind: "MANAGER",
  employee_id: "emp-1",
  employee_name: "조이슨",
  review_status: null,
});

const subjectDetail = (): EvaluationSubjectDetail => ({
  id: "subject-1",
  cycle_id: "cycle-1",
  employee_id: "emp-1",
  employee_name: "조이슨",
  org_unit: "정비사업팀",
  manager_user_id: "user-mgr",
  state: "IN_REVIEW",
  final_grade: null,
  rv_code: null,
  goals: [
    {
      id: "goal-1",
      title: "고객 응답 SLA",
      metric_kind: "KPI",
      target_label: "95%",
      weight_pct: 60,
      sort_order: 1,
    },
  ],
  reviews: [],
  calibrated_grade: null,
  calibration_reason: null,
  calibrated_by: null,
  calibrated_at: null,
  finalized_at: null,
});

function ok<T>(data: T) {
  return { data, response: new Response(null, { status: 200 }) };
}

function reject(status: number, message: string) {
  return { error: { error: { message } }, response: new Response(null, { status }) };
}

type RouteMap = Partial<Record<string, () => unknown>>;

function client(routes: RouteMap = {}) {
  const impl = { GET: vi.fn(), POST: vi.fn(), PUT: vi.fn() };
  impl.GET.mockImplementation((path: string) => {
    const handler = routes[path];
    return Promise.resolve(handler ? handler() : reject(404, `unmocked GET ${path}`));
  });
  return { impl, api: impl as unknown as ConsoleApiClient };
}

const defaultRoutes = (): RouteMap => ({
  "/api/v1/evaluation/cycles": () => ok({ items: [cycle()], total: 1 }),
  "/api/v1/evaluation/my-tasks": () => ok({ items: [task()] }),
  "/api/v1/evaluation/cycles/{cycle_id}": () => ok(detail()),
  "/api/v1/evaluation/cycles/{cycle_id}/preflight": () => ok(preflight()),
  "/api/v1/evaluation/subjects/{subject_id}": () => ok(subjectDetail()),
  "/api/v1/users": () =>
    ok({ items: [{ id: "user-mgr", display_name: "김성아", is_active: true }], limit: 100, offset: 0, total: 1 }),
  "/api/v1/employees": () =>
    ok({ items: [{ id: "emp-2", name: "최민석", employee_number: "E-2", org_unit: "경비팀" }], limit: 8, offset: 0, total: 1 }),
});

function renderScreen(
  capabilities: EvaluationCapabilities,
  api: ConsoleApiClient,
  sessionKey = "session-a",
) {
  return render(
    <MemoryRouter initialEntries={["/console/evaluation"]}>
      <EvaluationScreen
        api={api}
        branchId="branch-a"
        actorId="user-mgr"
        capabilities={capabilities}
        sessionKey={sessionKey}
      />
    </MemoryRouter>,
  );
}

async function openCycleRow() {
  const list = await screen.findByRole("list", { name: text.cycleList });
  const row = within(list).getByRole("button", { name: /2026 상반기 정기평가/ });
  await userEvent.click(row);
  return list;
}

describe("EvaluationScreen", () => {
  afterEach(() => {
    sessionStorage.clear();
  });

  it("denies an unauthorized user before fetching or exposing controls", () => {
    const { impl, api } = client();
    renderScreen(denied, api);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.write })).toBeNull();
    expect(impl.GET).not.toHaveBeenCalled();
  });

  it("retries an initial cycle-list error and renders the backend list", async () => {
    const { impl, api } = client();
    impl.GET.mockResolvedValueOnce(reject(500, "boom")).mockResolvedValueOnce(
      ok({ items: [cycle()], total: 1 }),
    );
    renderScreen(readOnly, api);
    expect(await screen.findByRole("alert")).toHaveTextContent("boom");
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(
      await screen.findByRole("button", { name: /2026 상반기 정기평가/ }),
    ).toBeVisible();
    expect(impl.GET).toHaveBeenCalledTimes(2);
  });

  it("shows empty states with a next action instead of fabricated rows", async () => {
    const { api } = client({
      "/api/v1/evaluation/cycles": () => ok({ items: [], total: 0 }),
      "/api/v1/evaluation/my-tasks": () => ok({ items: [] }),
    });
    renderScreen(submitter, api);
    expect(await screen.findByText(text.cycleEmptyReadOnly)).toBeVisible();
    expect(await screen.findByText(text.tasksEmpty)).toBeVisible();
  });

  it("opens a cycle with native keyboard activation and renders progress from the backend", async () => {
    const { api } = client(defaultRoutes());
    renderScreen(readOnly, api);
    const row = await screen.findByRole("button", { name: /2026 상반기 정기평가/ });
    row.focus();
    await userEvent.keyboard("{Enter}");
    const teams = await screen.findByRole("list", { name: text.teamProgress });
    expect(within(teams).getByText("정비사업팀")).toBeVisible();
    expect(within(teams).getByText("50%")).toBeVisible();
    expect(screen.getByText(text.stats.subjects)).toBeVisible();
  });

  it("renders a denied cycle detail as a status, not a retryable error", async () => {
    const routes = defaultRoutes();
    routes["/api/v1/evaluation/cycles/{cycle_id}"] = () => reject(403, "forbidden");
    const { api } = client(routes);
    renderScreen(readOnly, api);
    await userEvent.click(
      await screen.findByRole("button", { name: /2026 상반기 정기평가/ }),
    );
    expect(await screen.findByText(text.forbidden)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.retry })).toBeNull();
  });

  it("offers stage transitions only to manage capability and reconciles from the backend", async () => {
    const { impl, api } = client(defaultRoutes());
    impl.POST.mockResolvedValue(ok(detail("CALIBRATION")));
    renderScreen(manage, api);
    const list = await openCycleRow();
    const transition = await screen.findByRole("button", {
      name: text.transition.CALIBRATION,
    });
    await userEvent.click(transition);
    expect(impl.POST).toHaveBeenCalledWith(
      "/api/v1/evaluation/cycles/{cycle_id}/start-calibration",
      expect.objectContaining({ params: { path: { cycle_id: "cycle-1" } } }),
    );
    await waitFor(() => {
      expect(within(list).getByText(text.stage.CALIBRATION)).toBeVisible();
    });
  });

  it("hides manage-only controls from a read-only capability", async () => {
    const { api } = client(defaultRoutes());
    renderScreen(readOnly, api);
    await openCycleRow();
    expect(await screen.findByRole("list", { name: text.teamProgress })).toBeVisible();
    expect(screen.queryByRole("button", { name: text.createCycle })).toBeNull();
    expect(screen.queryByRole("button", { name: text.transition.CALIBRATION })).toBeNull();
    expect(screen.queryByRole("button", { name: text.add })).toBeNull();
  });

  it("enrolls a subject through the employee typeahead and reconciles the returned subject", async () => {
    const { impl, api } = client(defaultRoutes());
    impl.POST.mockResolvedValue(
      ok({
        ...subjectDetail(),
        id: "subject-2",
        employee_id: "emp-2",
        employee_name: "최민석",
        state: "ENROLLED",
      }),
    );
    renderScreen(manage, api);
    await openCycleRow();
    await userEvent.type(await screen.findByLabelText(text.employeeSearch), "최민");
    await userEvent.click(await screen.findByRole("button", { name: /최민석/ }));
    await userEvent.selectOptions(screen.getByLabelText(text.managerPick), "user-mgr");
    await userEvent.click(screen.getByRole("button", { name: text.add }));
    expect(impl.POST).toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects",
      expect.objectContaining({
        body: {
          cycle_id: "cycle-1",
          employee_id: "emp-2",
          manager_user_id: "user-mgr",
        },
      }),
    );
    const subjects = await screen.findByRole("list", { name: text.subjects });
    await waitFor(() => {
      expect(subjects).toHaveTextContent("최민석");
    });
  });

  it("replaces goals then reloads server preflight before enabling cycle open", async () => {
    const routes = defaultRoutes();
    let preflightCalls = 0;
    routes["/api/v1/evaluation/cycles"] = () => ok({ items: [cycle("DRAFT")], total: 1 });
    routes["/api/v1/evaluation/cycles/{cycle_id}"] = () => ok(detail("DRAFT"));
    routes["/api/v1/evaluation/cycles/{cycle_id}/preflight"] = () => {
      preflightCalls += 1;
      return ok(preflightCalls === 1 ? blockedPreflight("open") : { ...blockedPreflight("open"), blockers: [] });
    };
    const { impl, api } = client(routes);
    impl.PUT.mockResolvedValue(ok(subjectDetail()));
    renderScreen(manage, api);
    await openCycleRow();
    const cyclePanel = await screen.findByRole("region", { name: cycle().name });
    expect(within(cyclePanel).getByRole("button", { name: text.transition.OPEN })).toBeDisabled();
    await userEvent.click(
      within(screen.getByRole("list", { name: text.subjects })).getByRole("button", {
        name: /^조이슨/,
      }),
    );
    await screen.findByRole("region", { name: "조이슨" });
    await userEvent.click(screen.getByRole("button", { name: text.saveGoals }));
    await waitFor(() => {
      expect(preflightCalls).toBe(2);
    });
    await userEvent.click(screen.getByRole("button", { name: text.back }));
    await waitFor(() => {
      expect(
        within(screen.getByRole("region", { name: cycle().name })).getByRole("button", {
          name: text.transition.OPEN,
        }),
      ).toBeEnabled();
    });
  });

  it("calibrates then reloads server preflight before enabling finalization", async () => {
    const routes = defaultRoutes();
    let preflightCalls = 0;
    routes["/api/v1/evaluation/cycles"] = () => ok({ items: [cycle("CALIBRATION")], total: 1 });
    routes["/api/v1/evaluation/cycles/{cycle_id}"] = () => ok(detail("CALIBRATION"));
    routes["/api/v1/evaluation/cycles/{cycle_id}/preflight"] = () => {
      preflightCalls += 1;
      return ok(preflightCalls === 1 ? blockedPreflight("finalize") : { ...blockedPreflight("finalize"), blockers: [] });
    };
    routes["/api/v1/evaluation/subjects/{subject_id}"] = () =>
      ok({ ...subjectDetail(), state: "REVIEWED" });
    const { impl, api } = client(routes);
    impl.POST.mockResolvedValue(
      ok({
        ...subjectDetail(),
        state: "CALIBRATED",
        calibrated_grade: "A",
        calibrated_by: "user-admin",
        calibrated_at: "2026-07-25T00:00:00Z",
      }),
    );
    renderScreen(manage, api);
    await openCycleRow();
    const cyclePanel = await screen.findByRole("region", { name: cycle().name });
    expect(
      within(cyclePanel).getByRole("button", { name: text.transition.FINALIZED }),
    ).toBeDisabled();
    await userEvent.click(
      within(screen.getByRole("list", { name: text.subjects })).getByRole("button", {
        name: /^조이슨/,
      }),
    );
    await screen.findByRole("region", { name: "조이슨" });
    await userEvent.click(screen.getByRole("button", { name: "A" }));
    await userEvent.click(screen.getByRole("button", { name: text.calibrate }));
    await waitFor(() => {
      expect(preflightCalls).toBe(2);
    });
    await userEvent.click(screen.getByRole("button", { name: text.back }));
    await waitFor(() => {
      expect(
        within(screen.getByRole("region", { name: cycle().name })).getByRole("button", {
          name: text.transition.FINALIZED,
        }),
      ).toBeEnabled();
    });
  });

  it("walks the scorecard: evidence required for a manager grade, submit clears the task", async () => {
    const routes = defaultRoutes();
    let taskCalls = 0;
    routes["/api/v1/evaluation/my-tasks"] = () => {
      taskCalls += 1;
      return taskCalls === 1
        ? ok({ items: [task()] })
        : ok({ items: [] });
    };
    const { impl, api } = client(routes);
    impl.PUT.mockResolvedValue(ok({ id: "review-1", subject_id: "subject-1", kind: "MANAGER", status: "DRAFT", evaluator_user_id: "user-mgr", evidence_links: [], updated_at: "2026-07-24T00:00:00Z" }));
    impl.POST.mockResolvedValue(ok({ id: "review-1", subject_id: "subject-1", kind: "MANAGER", status: "SUBMITTED", grade: "A", evaluator_user_id: "user-mgr", evidence_links: [], submitted_at: "2026-07-24T00:00:00Z", updated_at: "2026-07-24T00:00:00Z" }));
    renderScreen(submitter, api);
    await userEvent.click(await screen.findByRole("button", { name: text.write }));
    const dialog = await screen.findByRole("dialog", { name: text.scorecard });
    expect(dialog).toHaveTextContent("고객 응답 SLA");
    const submit = screen.getByRole("button", { name: text.submit });
    await userEvent.click(screen.getByRole("button", { name: "A" }));
    expect(submit).toBeDisabled();
    await userEvent.type(screen.getByLabelText(text.evidenceRef), "KPI-SLA");
    await userEvent.type(screen.getByLabelText(text.evidenceLabel), "고객 응답 97%");
    await userEvent.click(screen.getByRole("button", { name: text.addEvidence }));
    expect(screen.getByText(text.notFound)).toBeVisible();
    expect(screen.queryByRole("button", { name: /KPI.*KPI-SLA/ })).toBeNull();
    expect(submit).toBeEnabled();
    await userEvent.click(submit);
    expect(impl.PUT).toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}",
      expect.objectContaining({
        params: { path: { subject_id: "subject-1", kind: "manager" } },
        body: expect.objectContaining({
          grade: "A",
          evidence_links: [
            expect.objectContaining({ object_ref: "KPI-SLA" }),
          ],
        }),
      }),
    );
    expect(impl.POST).toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}/submit",
      expect.anything(),
    );
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: text.scorecard })).toBeNull();
    });
    await waitFor(() => {
      expect(screen.getByText(text.tasksEmpty)).toBeVisible();
    });
  });

  it("lets a linked employee draft and submit only their SELF task through the typed self review route", async () => {
    const selfTask: EvaluationTaskSummary = {
      ...task(),
      kind: "SELF",
      employee_id: "emp-linked",
      employee_name: "이연결",
    };
    const selfSubject: EvaluationSubjectDetail = {
      ...subjectDetail(),
      employee_id: selfTask.employee_id,
      employee_name: selfTask.employee_name,
    };
    const routes = defaultRoutes();
    routes["/api/v1/evaluation/my-tasks"] = () => ok({ items: [selfTask] });
    routes["/api/v1/evaluation/subjects/{subject_id}"] = () => ok(selfSubject);
    const { impl, api } = client(routes);
    impl.PUT.mockResolvedValue(
      ok({
        id: "review-self",
        subject_id: selfTask.subject_id,
        kind: "SELF",
        status: "DRAFT",
        evaluator_user_id: "user-linked",
        grade: "A",
        note: "목표를 달성했습니다.",
        evidence_links: [],
        submitted_at: null,
        updated_at: "2026-07-25T00:00:00Z",
      }),
    );
    impl.POST.mockResolvedValue(
      ok({
        id: "review-self",
        subject_id: selfTask.subject_id,
        kind: "SELF",
        status: "SUBMITTED",
        grade: "A",
        note: "목표를 달성했습니다.",
        evaluator_user_id: "user-linked",
        evidence_links: [],
        submitted_at: "2026-07-25T00:00:00Z",
        updated_at: "2026-07-25T00:00:00Z",
      }),
    );
    renderScreen(submitOnly, api);

    const tasks = await screen.findByRole("list", { name: text.myTasks });
    expect(within(tasks).getByText(/이연결 · 자기평가/)).toBeVisible();
    await userEvent.click(within(tasks).getByRole("button", { name: text.write }));

    const dialog = await screen.findByRole("dialog", { name: text.scorecard });
    expect(dialog).toHaveTextContent(text.reviewKind.SELF);
    await userEvent.click(within(dialog).getByRole("button", { name: "A" }));
    await userEvent.type(screen.getByLabelText(text.notePlaceholder), "목표를 달성했습니다.");
    await userEvent.click(screen.getByRole("button", { name: text.saveDraft }));
    expect(impl.PUT).toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}",
      expect.objectContaining({
        params: { path: { subject_id: selfTask.subject_id, kind: "self" } },
        body: {
          grade: "A",
          note: "목표를 달성했습니다.",
          evidence_links: [],
        },
      }),
    );

    await userEvent.click(screen.getByRole("button", { name: text.submit }));
    expect(impl.POST).toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}/submit",
      expect.objectContaining({
        params: { path: { subject_id: selfTask.subject_id, kind: "self" } },
      }),
    );
    expect(impl.POST).not.toHaveBeenCalledWith(
      "/api/v1/evaluation/subjects/{subject_id}/reviews/{kind}/submit",
      expect.objectContaining({
        params: { path: expect.objectContaining({ kind: "manager" }) },
      }),
    );
  });

  it("keeps the selected cycle gate bound to that cycle when a task from another cycle is submitted", async () => {
    const cycleA = { ...cycle("DRAFT"), id: "cycle-a" };
    const cycleB = {
      ...cycle("CALIBRATION"),
      id: "cycle-b",
      name: "2026 하반기 정기평가",
      period_label: "2026 H2",
    };
    const detailA = { ...detail("DRAFT"), ...cycleA };
    const detailB = { ...detail("CALIBRATION"), ...cycleB, subjects: [] };
    const taskB: EvaluationTaskSummary = {
      ...task(),
      subject_id: "subject-b",
      cycle_id: cycleB.id,
      cycle_name: cycleB.name,
      employee_id: "emp-b",
      employee_name: "한지민",
    };
    const subjectB = {
      ...subjectDetail(),
      id: taskB.subject_id,
      cycle_id: cycleB.id,
      employee_id: taskB.employee_id,
      employee_name: taskB.employee_name,
    };
    const routes = defaultRoutes();
    routes["/api/v1/evaluation/cycles"] = () => ok({ items: [cycleA, cycleB], total: 2 });
    routes["/api/v1/evaluation/my-tasks"] = () => ok({ items: [taskB] });
    const { impl, api } = client(routes);
    impl.GET.mockImplementation(
      (path: string, request?: { params?: { path?: Record<string, string> } }) => {
        const cycleId = request?.params?.path?.cycle_id;
        if (path === "/api/v1/evaluation/cycles/{cycle_id}") {
          return Promise.resolve(ok(cycleId === cycleB.id ? detailB : detailA));
        }
        if (path === "/api/v1/evaluation/cycles/{cycle_id}/preflight") {
          return Promise.resolve(
            ok(cycleId === cycleB.id ? preflight() : { ...preflight(), next_transition: "open" }),
          );
        }
        if (path === "/api/v1/evaluation/subjects/{subject_id}") {
          return Promise.resolve(ok(subjectB));
        }
        const handler = routes[path];
        return Promise.resolve(handler ? handler() : reject(404, `unmocked GET ${path}`));
      },
    );
    impl.PUT.mockResolvedValue(
      ok({
        id: "review-b",
        subject_id: taskB.subject_id,
        kind: "MANAGER",
        status: "DRAFT",
        evaluator_user_id: "user-mgr",
        evidence_links: [],
        updated_at: "2026-07-25T00:00:00Z",
      }),
    );
    impl.POST.mockResolvedValue(
      ok({
        id: "review-b",
        subject_id: taskB.subject_id,
        kind: "MANAGER",
        status: "SUBMITTED",
        grade: "A",
        evaluator_user_id: "user-mgr",
        evidence_links: [],
        submitted_at: "2026-07-25T00:00:00Z",
        updated_at: "2026-07-25T00:00:00Z",
      }),
    );

    renderScreen(manage, api);
    await openCycleRow();
    const cyclePanel = await screen.findByRole("region", { name: cycleA.name });
    expect(within(cyclePanel).getByRole("button", { name: text.transition.OPEN })).toBeEnabled();

    await userEvent.click(await screen.findByRole("button", { name: text.write }));
    await screen.findByRole("dialog", { name: text.scorecard });
    await userEvent.click(screen.getByRole("button", { name: "A" }));
    await userEvent.type(screen.getByLabelText(text.evidenceRef), "KPI-H2");
    await userEvent.type(screen.getByLabelText(text.evidenceLabel), "하반기 KPI");
    await userEvent.click(screen.getByRole("button", { name: text.addEvidence }));
    await userEvent.click(screen.getByRole("button", { name: text.submit }));

    await waitFor(() => {
      expect(impl.GET).toHaveBeenCalledWith(
        "/api/v1/evaluation/cycles/{cycle_id}/preflight",
        expect.objectContaining({ params: { path: { cycle_id: cycleB.id } } }),
      );
    });
    expect(within(cyclePanel).getByRole("button", { name: text.transition.OPEN })).toBeEnabled();
    expect(within(cyclePanel).queryByRole("button", { name: text.transition.FINALIZED })).toBeNull();
  });

  it("closes the scorecard with Escape without submitting", async () => {
    const { impl, api } = client(defaultRoutes());
    renderScreen(submitter, api);
    await userEvent.click(await screen.findByRole("button", { name: text.write }));
    await screen.findByRole("dialog", { name: text.scorecard });
    await userEvent.keyboard("{Escape}");
    await waitFor(() => {
      expect(screen.queryByRole("dialog", { name: text.scorecard })).toBeNull();
    });
    expect(impl.POST).not.toHaveBeenCalled();
  });

  it("opens the audited person ledger from a task and drills back into the RV- object", async () => {
    const routes = defaultRoutes();
    routes["/api/v1/evaluation/employees/{employee_id}/reviews"] = () =>
      ok({
        items: [
          {
            rv_code: "RV-2501",
            cycle_id: "cycle-0",
            cycle_name: "2025 하반기 정기",
            period_label: "2025 H2",
            final_grade: "A",
            finalized_at: "2026-01-15T00:00:00Z",
            subject_id: "subject-0",
          },
        ],
      });
    routes["/api/v1/evaluation/subjects/{subject_id}"] = () =>
      ok({ ...subjectDetail(), id: "subject-0", state: "FINALIZED", rv_code: "RV-2501", final_grade: "A" });
    const { impl, api } = client(routes);
    renderScreen(submitter, api);
    await userEvent.click(
      await screen.findByRole("button", { name: /조이슨 · 관리자 평가/ }),
    );
    expect(await screen.findByText(text.auditChip)).toBeVisible();
    expect(await screen.findByText("RV-2501")).toBeVisible();
    expect(impl.GET).toHaveBeenCalledWith(
      "/api/v1/evaluation/employees/{employee_id}/reviews",
      expect.objectContaining({ params: { path: { employee_id: "emp-1" } } }),
    );
    await userEvent.click(screen.getByRole("button", { name: /RV-2501/ }));
    const zone = await screen.findByRole("region", { name: "조이슨" });
    expect(within(zone).getByText(text.subjectState.FINALIZED)).toBeVisible();
  });

  it("renders a denied-by-omission ledger 404 as a status, not a retryable error", async () => {
    const routes = defaultRoutes();
    routes["/api/v1/evaluation/employees/{employee_id}/reviews"] = () =>
      reject(404, "not found");
    const { api } = client(routes);
    renderScreen(submitter, api);
    await userEvent.click(
      await screen.findByRole("button", { name: /조이슨 · 관리자 평가/ }),
    );
    expect(await screen.findByText(text.notFound)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.retry })).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("renders the task subject as plain text for a submit-only capability instead of a dead drill", async () => {
    const { impl, api } = client(defaultRoutes());
    renderScreen(submitOnly, api);
    const tasks = await screen.findByRole("list", { name: text.myTasks });
    expect(within(tasks).getByText(/조이슨 · 관리자 평가/)).toBeVisible();
    expect(
      within(tasks).queryByRole("button", { name: /조이슨 · 관리자 평가/ }),
    ).toBeNull();
    expect(within(tasks).getByRole("button", { name: text.write })).toBeEnabled();
    expect(impl.GET).not.toHaveBeenCalledWith("/api/v1/evaluation/cycles", expect.anything());
  });

  it("fences stale responses when the authenticated API client is replaced", async () => {
    let resolveFirst!: (value: unknown) => void;
    const first = new Promise((resolve) => {
      resolveFirst = resolve;
    });
    const staleImpl = { GET: vi.fn().mockReturnValue(first), POST: vi.fn(), PUT: vi.fn() };
    const staleApi = staleImpl as unknown as ConsoleApiClient;
    const { api } = client({
      ...defaultRoutes(),
      "/api/v1/evaluation/cycles": () =>
        ok({ items: [cycle("CALIBRATION")], total: 1 }),
    });
    const view = render(
      <MemoryRouter initialEntries={["/console/evaluation"]}>
        <EvaluationScreen
          api={staleApi}
          branchId="branch-a"
          actorId="user-mgr"
          capabilities={readOnly}
          sessionKey="session-a"
        />
      </MemoryRouter>,
    );
    await waitFor(() => {
      expect(staleImpl.GET).toHaveBeenCalledTimes(1);
    });
    view.rerender(
      <MemoryRouter initialEntries={["/console/evaluation"]}>
        <EvaluationScreen
          api={api}
          branchId="branch-a"
          actorId="user-mgr"
          capabilities={readOnly}
          sessionKey="session-b"
        />
      </MemoryRouter>,
    );
    expect(
      await screen.findByRole("button", { name: /2026 상반기 정기평가/ }),
    ).toHaveTextContent(text.stage.CALIBRATION);
    resolveFirst(ok({ items: [cycle("DRAFT")], total: 1 }));
    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /2026 상반기 정기평가/ }),
      ).toHaveTextContent(text.stage.CALIBRATION);
    });
  });
});
