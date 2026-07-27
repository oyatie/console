import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type SyntheticEvent,
} from "react";
import { useNavigate } from "react-router";

import type { components } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";
import { evaluationStrings as text } from "../../i18n/evaluation";
import { consoleScreenPath } from "../shell/nav";
import {
  createEvaluationApi,
  EvaluationApiError,
  evaluationApiErrorFrom,
  type EvaluationCycleDetail,
  type EvaluationCycleStage,
  type EvaluationCycleSummary,
  type EvaluationCycleTransition,
  type EvaluationEvidenceKind,
  type EvaluationEvidenceLinkInput,
  type EvaluationGoalInput,
  type EvaluationGrade,
  type EvaluationLedgerEntry,
  type EvaluationPreflightReport,
  type EvaluationReview,
  type EvaluationReviewKind,
  type EvaluationSubjectDetail,
  type EvaluationTaskSummary,
  type SaveEvaluationReviewRequest,
} from "./evaluationApi";
import type { EvaluationCapabilities } from "./evaluationCapabilities";
import {
  evidenceRoutePolicy,
  parseEvaluationEvidenceKind,
  parseEvaluationMetricKind,
  restoreEvaluationState,
  type EvaluationStoredState,
  type EvaluationView,
} from "./evaluationUiPolicy";
import "./evaluation.css";

type Props = {
  api: ConsoleApiClient;
  branchId: string | undefined;
  actorId: string | undefined;
  capabilities: EvaluationCapabilities;
  /** Changes whenever auth replaces the effective tenant/session. */
  sessionKey: string | undefined;
};

const apiFenceIds = new WeakMap<ConsoleApiClient, number>();
let nextApiFenceId = 1;

function apiFenceKey(api: ConsoleApiClient): number {
  const existing = apiFenceIds.get(api);
  if (existing) return existing;
  const id = nextApiFenceId++;
  apiFenceIds.set(api, id);
  return id;
}

interface Failure {
  message: string;
  status?: number;
}

function failureOf(cause: unknown, fallback: string): Failure {
  if (cause instanceof EvaluationApiError) {
    return { message: cause.message, status: cause.status };
  }
  return { message: cause instanceof Error ? cause.message : fallback };
}

function formText(data: FormData, name: string): string {
  const value = data.get(name);
  return typeof value === "string" ? value : "";
}

/** Fenced async resource: newest run wins; aborted/stale results are dropped. */
function useFenced<T>() {
  const [data, setData] = useState<T>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Failure>();
  const gen = useRef(0);
  const op = useRef<AbortController | undefined>(undefined);
  const run = useCallback(async (work: (signal: AbortSignal) => Promise<T>) => {
    op.current?.abort();
    const controller = new AbortController();
    op.current = controller;
    const token = ++gen.current;
    setLoading(true);
    setError(undefined);
    try {
      const next = await work(controller.signal);
      if (gen.current === token) {
        setData(next);
        setLoading(false);
      }
    } catch (cause) {
      if (gen.current === token && !controller.signal.aborted) {
        setError(failureOf(cause, text.loadError));
        setLoading(false);
      }
    }
  }, []);
  const reset = useCallback(() => {
    gen.current += 1;
    op.current?.abort();
    setData(undefined);
    setLoading(false);
    setError(undefined);
  }, []);
  const reconcile = useCallback((updater: (current: T | undefined) => T | undefined) => {
    setData(updater);
  }, []);
  useEffect(
    () => () => {
      gen.current += 1;
      op.current?.abort();
    },
    [],
  );
  return { data, loading, error, run, reset, reconcile };
}

/** Fenced async resources keyed by backend identity; stale results stay per-key. */
function useKeyedFenced<T>() {
  const [data, setData] = useState<Readonly<Record<string, T>>>({});
  const generations = useRef(new Map<string, number>());
  const operations = useRef(new Map<string, AbortController>());
  const run = useCallback(async (key: string, work: (signal: AbortSignal) => Promise<T>) => {
    operations.current.get(key)?.abort();
    const controller = new AbortController();
    operations.current.set(key, controller);
    const token = (generations.current.get(key) ?? 0) + 1;
    generations.current.set(key, token);
    try {
      const next = await work(controller.signal);
      if (generations.current.get(key) === token) {
        setData((current) => ({ ...current, [key]: next }));
      }
    } catch {
      // Detail gates remain fail-closed when the authoritative report cannot load.
    }
  }, []);
  const reset = useCallback((key?: string) => {
    if (key) {
      generations.current.set(key, (generations.current.get(key) ?? 0) + 1);
      operations.current.get(key)?.abort();
      operations.current.delete(key);
      setData((current) =>
        Object.fromEntries(Object.entries(current).filter(([entryKey]) => entryKey !== key)),
      );
      return;
    }
    for (const operation of operations.current.values()) operation.abort();
    operations.current.clear();
    generations.current.clear();
    setData({});
  }, []);
  useEffect(
    () => () => {
      for (const operation of operations.current.values()) operation.abort();
      operations.current.clear();
      generations.current.clear();
    },
    [],
  );
  return { data, run, reset };
}

type Tone = "ok" | "warn" | "danger" | "info" | "purple" | "muted" | "teal";

const CHIP_CLASS: Record<Tone, string> = {
  ok: "evaluation__chip evaluation__chip--ok",
  warn: "evaluation__chip evaluation__chip--warn",
  danger: "evaluation__chip evaluation__chip--danger",
  info: "evaluation__chip evaluation__chip--info",
  purple: "evaluation__chip evaluation__chip--purple",
  muted: "evaluation__chip evaluation__chip--muted",
  teal: "evaluation__chip evaluation__chip--teal",
};

const BAR_CLASS: Record<"ok" | "warn" | "teal", string> = {
  ok: "evaluation__bar-fill evaluation__bar-fill--ok",
  warn: "evaluation__bar-fill evaluation__bar-fill--warn",
  teal: "evaluation__bar-fill evaluation__bar-fill--teal",
};

const STAGE_TONE: Record<EvaluationCycleStage, Tone> = {
  DRAFT: "muted",
  OPEN: "teal",
  CALIBRATION: "warn",
  FINALIZED: "ok",
  ARCHIVED: "muted",
};

const SUBJECT_STATE_TONE: Record<EvaluationSubjectDetail["state"], Tone> = {
  ENROLLED: "muted",
  IN_REVIEW: "teal",
  REVIEWED: "info",
  CALIBRATED: "warn",
  FINALIZED: "ok",
};

const STAGES: EvaluationCycleStage[] = [
  "DRAFT",
  "OPEN",
  "CALIBRATION",
  "FINALIZED",
  "ARCHIVED",
];

function transitionTarget(
  transition: EvaluationCycleTransition,
): Exclude<EvaluationCycleStage, "DRAFT"> {
  switch (transition) {
    case "open":
      return "OPEN";
    case "start_calibration":
      return "CALIBRATION";
    case "finalize":
      return "FINALIZED";
    case "archive":
      return "ARCHIVED";
  }
}

const GRADES: EvaluationGrade[] = ["S", "A", "B", "C", "D"];

function stageLabel(stage: string): string {
  return Object.entries(text.stage).find(([key]) => key === stage)?.[1] ?? text.stage.unknown;
}

function stateLabel(state: string): string {
  return (
    Object.entries(text.subjectState).find(([key]) => key === state)?.[1] ??
    text.subjectState.unknown
  );
}

function dueChip(dueDate: string): { label: string; tone: Tone } {
  const due = new Date(`${dueDate}T00:00:00`);
  if (Number.isNaN(due.getTime())) return { label: dueDate, tone: "muted" };
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const days = Math.round((due.getTime() - today.getTime()) / 86_400_000);
  if (days < 0) return { label: `D+${String(-days)}`, tone: "danger" };
  if (days === 0) return { label: "D-DAY", tone: "warn" };
  return { label: `D-${String(days)}`, tone: days <= 7 ? "warn" : "muted" };
}

function dateOnly(timestamp: string | null | undefined): string {
  return timestamp ? timestamp.slice(0, 10) : "";
}

function progressTone(pct: number): "ok" | "warn" | "teal" {
  if (pct >= 100) return "ok";
  if (pct < 50) return "warn";
  return "teal";
}

function upsertReview(
  reviews: EvaluationReview[],
  next: EvaluationReview,
): EvaluationReview[] {
  const exists = reviews.some((review) => review.kind === next.kind);
  return exists
    ? reviews.map((review) => (review.kind === next.kind ? next : review))
    : [...reviews, next];
}

type Employee = components["schemas"]["Employee"];

interface EmployeeHit {
  id: string;
  name: string;
  employee_number: string | null;
  org_unit: string | null;
}

async function searchEmployees(
  api: ConsoleApiClient,
  search: string,
  signal: AbortSignal,
): Promise<EmployeeHit[]> {
  const { data, error, response } = await api.GET("/api/v1/employees", {
    params: { query: { search, limit: 8 } },
    signal,
  });
  if (data === undefined) throw evaluationApiErrorFrom(error, response.status);
  return data.items.map((employee: Employee) => ({
    id: employee.id,
    name: employee.name,
    employee_number: employee.employee_number ?? null,
    org_unit: employee.org_unit ?? null,
  }));
}

interface ManagerOption {
  id: string;
  display_name: string;
}

// ponytail: first 100 users only; move to a server-side search param when the
// directory outgrows one page.
async function listManagerOptions(
  api: ConsoleApiClient,
  signal: AbortSignal,
): Promise<ManagerOption[]> {
  const { data, error, response } = await api.GET("/api/v1/users", {
    params: { query: { limit: 100 } },
    signal,
  });
  if (data === undefined) throw evaluationApiErrorFrom(error, response.status);
  return data.items
    .filter((user) => user.is_active)
    .map((user) => ({ id: user.id, display_name: user.display_name }));
}

function EvidenceLinkDisplay({
  kind,
  objectRef,
  label,
}: {
  kind: EvaluationEvidenceKind;
  objectRef: string;
  label: string;
}) {
  const policy = evidenceRoutePolicy(kind);
  return (
    <span
      className="evaluation__evidence-chip"
      aria-label={`${text.evidenceKind[kind]} · ${text.notFound}`}
      data-evidence-route={policy.reason}
    >
      <span className={CHIP_CLASS.muted}>{text.evidenceKind[kind]}</span>
      <span className="evaluation__code">{objectRef}</span>
      <span>{label}</span>
      <span className={CHIP_CLASS.muted}>{text.notFound}</span>
    </span>
  );
}

/**
 * Re-mount synchronously whenever effective authority changes. Effects run too
 * late to fence an old tenant/session's selection, error, or busy state.
 */
export function EvaluationScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.branchId ?? "org",
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <EvaluationBody key={sessionFence} {...props} />;
}

function EvaluationBody({ api, actorId, capabilities }: Props) {
  const evaluationApi = useMemo(() => createEvaluationApi(api), [api]);
  const navigate = useNavigate();
  const storageKey = `evaluation:view:${actorId ?? "anon"}`;
  const restored: EvaluationStoredState = useMemo(
    () => restoreEvaluationState(sessionStorage.getItem(storageKey)),
    [storageKey],
  );

  const [stageFilter, setStageFilter] = useState<EvaluationCycleStage>();
  const [selectedCycleId, setSelectedCycleId] = useState(restored.cycleId);
  const [view, setView] = useState<EvaluationView>(restored.view ?? { kind: "cycle" });
  const [scorecard, setScorecard] = useState<{
    subjectId: string;
    cycleId: string;
    kind: EvaluationReviewKind;
    employeeName: string;
    cycleName: string;
  }>();

  const cycles = useFenced<EvaluationCycleSummary[]>();
  const tasks = useFenced<EvaluationTaskSummary[]>();
  const detail = useFenced<EvaluationCycleDetail>();
  const preflight = useKeyedFenced<EvaluationPreflightReport>();
  const subject = useFenced<EvaluationSubjectDetail>();
  const ledger = useFenced<EvaluationLedgerEntry[]>();
  const card = useFenced<EvaluationSubjectDetail>();
  const { run: runCycles, reconcile: reconcileCycles } = cycles;
  const { run: runTasks } = tasks;
  const { run: runDetail, reset: resetDetail, reconcile: reconcileDetail } = detail;
  const { run: runPreflight, reset: resetPreflight } = preflight;
  const { run: runSubject, reset: resetSubject, reconcile: reconcileSubject } = subject;
  const { run: runLedger, reset: resetLedger } = ledger;
  const { run: runCard, reset: resetCard, reconcile: reconcileCard } = card;

  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<Failure>();
  const mutOp = useRef<AbortController | undefined>(undefined);
  const mutGen = useRef(0);
  const mutate = useCallback(
    async <T,>(work: (signal: AbortSignal) => Promise<T>): Promise<T | undefined> => {
      mutOp.current?.abort();
      const controller = new AbortController();
      mutOp.current = controller;
      const token = ++mutGen.current;
      setBusy(true);
      setActionError(undefined);
      try {
        const value = await work(controller.signal);
        return mutGen.current === token ? value : undefined;
      } catch (cause) {
        if (mutGen.current === token && !controller.signal.aborted) {
          setActionError(failureOf(cause, text.actionError));
        }
        return undefined;
      } finally {
        if (mutGen.current === token) setBusy(false);
      }
    },
    [],
  );
  useEffect(
    () => () => {
      mutGen.current += 1;
      mutOp.current?.abort();
    },
    [],
  );

  useEffect(() => {
    try {
      sessionStorage.setItem(storageKey, JSON.stringify({ cycleId: selectedCycleId, view }));
    } catch {
      // Storage unavailable: selection simply does not survive a refresh.
    }
  }, [storageKey, selectedCycleId, view]);

  const loadCycles = useCallback(() => {
    if (!capabilities.canRead) return;
    void runCycles(async (signal) => {
      const page = await evaluationApi.listCycles(
        stageFilter ? { stage: stageFilter } : undefined,
        signal,
      );
      return page.items;
    });
  }, [capabilities.canRead, runCycles, evaluationApi, stageFilter]);
  useEffect(() => {
    loadCycles();
  }, [loadCycles]);

  const loadTasks = useCallback(() => {
    if (!capabilities.canSubmit) return;
    void runTasks(async (signal) => (await evaluationApi.myTasks(signal)).items);
  }, [capabilities.canSubmit, runTasks, evaluationApi]);
  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  const loadDetail = useCallback(
    (cycleId: string) => {
      void runDetail((signal) => evaluationApi.getCycle(cycleId, signal));
      if (capabilities.canManage) {
        void runPreflight(cycleId, (signal) => evaluationApi.getPreflight(cycleId, signal));
      }
    },
    [runDetail, runPreflight, evaluationApi, capabilities.canManage],
  );
  useEffect(() => {
    if (!capabilities.canRead || !selectedCycleId) {
      resetDetail();
      resetPreflight();
      return;
    }
    loadDetail(selectedCycleId);
  }, [capabilities.canRead, selectedCycleId, loadDetail, resetDetail, resetPreflight]);

  const loadSubject = useCallback(
    (subjectId: string) => {
      void runSubject((signal) => evaluationApi.getSubject(subjectId, signal));
    },
    [runSubject, evaluationApi],
  );
  useEffect(() => {
    if (view.kind !== "subject" || !capabilities.canRead) {
      resetSubject();
      return;
    }
    loadSubject(view.subjectId);
  }, [view, capabilities.canRead, loadSubject, resetSubject]);

  const loadLedger = useCallback(
    (employeeId: string) => {
      void runLedger(async (signal) =>
        (await evaluationApi.employeeReviews(employeeId, signal)).items,
      );
    },
    [runLedger, evaluationApi],
  );
  useEffect(() => {
    if (view.kind !== "person" || !capabilities.canRead) {
      resetLedger();
      return;
    }
    loadLedger(view.employeeId);
  }, [view, capabilities.canRead, loadLedger, resetLedger]);

  useEffect(() => {
    if (!scorecard) {
      resetCard();
      return;
    }
    void runCard((signal) => evaluationApi.getSubject(scorecard.subjectId, signal));
  }, [scorecard, runCard, resetCard, evaluationApi]);

  const reconcilePreflightFromServer = useCallback(
    (cycleId: string) => {
      // A committed mutation invalidates this cycle's local blocker/ready decision.
      // Keep its gate hidden until the server returns a fresh preflight report.
      resetPreflight(cycleId);
      if (!capabilities.canManage) return;
      void runPreflight(cycleId, (signal) => evaluationApi.getPreflight(cycleId, signal));
    },
    [capabilities.canManage, evaluationApi, resetPreflight, runPreflight],
  );

  const runTransition = async (target: EvaluationCycleStage) => {
    const cycleId = selectedCycleId;
    if (!cycleId || !capabilities.canManage) return;
    const call =
      target === "OPEN"
        ? evaluationApi.openCycle
        : target === "CALIBRATION"
          ? evaluationApi.startCalibration
          : target === "FINALIZED"
            ? evaluationApi.finalizeCycle
            : evaluationApi.archiveCycle;
    const next = await mutate((signal) => call(cycleId, signal));
    if (!next) return;
    reconcileDetail(() => next);
    reconcileCycles((current) =>
      current?.map((cycle) => (cycle.id === next.id ? next : cycle)),
    );
    reconcilePreflightFromServer(cycleId);
  };

  const createCycle = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!capabilities.canManage) return;
    const form = event.currentTarget;
    const data = new FormData(form);
    const created = await mutate((signal) =>
      evaluationApi.createCycle(
        {
          name: formText(data, "name").trim(),
          kind: formText(data, "kind") === "PROBATION" ? "PROBATION" : "REGULAR",
          period_label: formText(data, "period_label").trim(),
          due_date: formText(data, "due_date"),
        },
        signal,
      ),
    );
    if (!created) return;
    reconcileCycles((current) => [created, ...(current ?? [])]);
    setSelectedCycleId(created.id);
    setView({ kind: "cycle" });
    form.reset();
  };

  const addSubject = async (employeeId: string, managerUserId: string) => {
    const cycleId = selectedCycleId;
    if (!capabilities.canManage || !cycleId) return false;
    const created = await mutate((signal) =>
      evaluationApi.addSubject(
        { cycle_id: cycleId, employee_id: employeeId, manager_user_id: managerUserId },
        signal,
      ),
    );
    if (!created) return false;
    reconcileDetail((current) =>
      current
        ? {
            ...current,
            subjects: [...current.subjects, created],
            subjects_total: current.subjects_total + 1,
          }
        : current,
    );
    reconcilePreflightFromServer(cycleId);
    return true;
  };

  const saveGoals = async (subjectId: string, goals: EvaluationGoalInput[]) => {
    const next = await mutate((signal) =>
      evaluationApi.replaceGoals(subjectId, { goals }, signal),
    );
    if (!next) return;
    reconcileSubject(() => next);
    reconcilePreflightFromServer(next.cycle_id);
  };

  const calibrate = async (
    subjectId: string,
    finalGrade: EvaluationGrade,
    reason: string,
  ) => {
    if (!capabilities.canCalibrate) return;
    const next = await mutate((signal) =>
      evaluationApi.calibrateSubject(
        subjectId,
        reason ? { final_grade: finalGrade, reason } : { final_grade: finalGrade },
        signal,
      ),
    );
    if (!next) return;
    reconcileSubject(() => next);
    reconcileDetail((current) =>
      current
        ? {
            ...current,
            subjects: current.subjects.map((row) => (row.id === next.id ? next : row)),
          }
        : current,
    );
    reconcilePreflightFromServer(next.cycle_id);
  };

  const saveScorecardDraft = async (fields: SaveEvaluationReviewRequest) => {
    if (!scorecard) return;
    const open = scorecard;
    const saved = await mutate((signal) =>
      evaluationApi.saveReview(open.subjectId, open.kind, fields, signal),
    );
    if (!saved) return;
    reconcileCard((current) =>
      current ? { ...current, reviews: upsertReview(current.reviews, saved) } : current,
    );
  };

  const submitScorecard = async (fields: SaveEvaluationReviewRequest) => {
    if (!scorecard) return;
    const open = scorecard;
    const submitted = await mutate(async (signal) => {
      await evaluationApi.saveReview(open.subjectId, open.kind, fields, signal);
      return evaluationApi.submitReview(open.subjectId, open.kind, signal);
    });
    if (!submitted) return;
    setScorecard(undefined);
    reconcilePreflightFromServer(open.cycleId);
    loadTasks();
    if (view.kind === "subject" && view.subjectId === open.subjectId) {
      loadSubject(open.subjectId);
    }
  };

  if (!capabilities.canRead && !capabilities.canSubmit) {
    return (
      <main className="evaluation">
        <section className="evaluation__card" aria-labelledby="evaluation-title">
          <h1 id="evaluation-title">{text.title}</h1>
          <p role="status">{text.denied}</p>
        </section>
      </main>
    );
  }

  const headerCycle =
    (selectedCycleId ? detail.data : undefined) ??
    cycles.data?.find((cycle) => cycle.stage === "OPEN");
  const headerDue = headerCycle ? dueChip(headerCycle.due_date) : undefined;
  const parentStage =
    subject.data && detail.data?.id === subject.data.cycle_id ? detail.data.stage : undefined;

  return (
    <main
      className="evaluation"
      aria-busy={busy || cycles.loading || detail.loading || subject.loading}
    >
      <header className="evaluation__topbar">
        <h1 id="evaluation-title">{text.title}</h1>
        {headerCycle && headerDue && (
          <span className="evaluation__topbar-chips">
            <strong>{headerCycle.name}</strong>
            <span className={CHIP_CLASS.muted}>{text.kind[headerCycle.kind]}</span>
            <span className={CHIP_CLASS[STAGE_TONE[headerCycle.stage]]}>
              {stageLabel(headerCycle.stage)}
            </span>
            <span className={`${CHIP_CLASS[headerDue.tone]} evaluation__mono`}>
              {headerDue.label}
            </span>
          </span>
        )}
      </header>
      {actionError && !scorecard && (
        <div className="evaluation__alert" role="alert">
          <span>{actionError.message}</span>
        </div>
      )}
      <div className="evaluation__grid">
        <div className="evaluation__rail">
          {capabilities.canRead && (
            <section className="evaluation__card" aria-label={text.cycleList}>
              <h2>{text.cycleList}</h2>
              <div
                className="evaluation__filter"
                role="group"
                aria-label={text.stageFilter}
              >
                <button
                  type="button"
                  className={stageFilter === undefined ? "evaluation__filter-chip evaluation__filter-chip--on" : "evaluation__filter-chip"}
                  aria-pressed={stageFilter === undefined}
                  onClick={() => {
                    setStageFilter(undefined);
                  }}
                >
                  {text.allStages}
                </button>
                {STAGES.map((stage) => (
                  <button
                    key={stage}
                    type="button"
                    className={stageFilter === stage ? "evaluation__filter-chip evaluation__filter-chip--on" : "evaluation__filter-chip"}
                    aria-pressed={stageFilter === stage}
                    onClick={() => {
                      setStageFilter(stage);
                    }}
                  >
                    {stageLabel(stage)}
                  </button>
                ))}
              </div>
              {cycles.error ? (
                <div className="evaluation__alert" role="alert">
                  <span>{cycles.error.message}</span>
                  <button type="button" onClick={loadCycles}>
                    {text.retry}
                  </button>
                </div>
              ) : cycles.loading && !cycles.data ? (
                <p role="status">{text.loading}</p>
              ) : (
                <ul className="evaluation__list" aria-label={text.cycleList}>
                  {cycles.data?.length ? (
                    cycles.data.map((cycle) => {
                      const due = dueChip(cycle.due_date);
                      return (
                        <li key={cycle.id}>
                          <button
                            type="button"
                            className={cycle.id === selectedCycleId ? "evaluation__row evaluation__row--selected" : "evaluation__row"}
                            aria-pressed={cycle.id === selectedCycleId}
                            onClick={() => {
                              setSelectedCycleId(cycle.id);
                              setView({ kind: "cycle" });
                            }}
                          >
                            <span className="evaluation__row-main">
                              <strong>{cycle.name}</strong>
                              <span className={CHIP_CLASS.muted}>{cycle.period_label}</span>
                            </span>
                            <span className="evaluation__row-side">
                              <span className={CHIP_CLASS[STAGE_TONE[cycle.stage]]}>
                                {stageLabel(cycle.stage)}
                              </span>
                              <span className={`${CHIP_CLASS[due.tone]} evaluation__mono`}>
                                {due.label}
                              </span>
                            </span>
                          </button>
                        </li>
                      );
                    })
                  ) : (
                    <li role="status">
                      {capabilities.canManage ? text.cycleEmpty : text.cycleEmptyReadOnly}
                    </li>
                  )}
                </ul>
              )}
              {capabilities.canManage && (
                <CycleCreateForm busy={busy} onCreate={createCycle} />
              )}
            </section>
          )}
          {capabilities.canSubmit && (
            <section className="evaluation__card" aria-label={text.myTasks}>
              <h2>{text.myTasks}</h2>
              {tasks.error ? (
                <div className="evaluation__alert" role="alert">
                  <span>{tasks.error.message}</span>
                  <button type="button" onClick={loadTasks}>
                    {text.retry}
                  </button>
                </div>
              ) : tasks.loading && !tasks.data ? (
                <p role="status">{text.loading}</p>
              ) : (
                <ul className="evaluation__tasks" aria-label={text.myTasks}>
                  {tasks.data?.length ? (
                    tasks.data.map((task) => {
                      const due = dueChip(task.due_date);
                      return (
                        <li key={`${task.subject_id}:${task.kind}`}>
                          <span className={`${CHIP_CLASS[due.tone]} evaluation__mono`}>
                            {due.label}
                          </span>
                          {capabilities.canRead ? (
                            <button
                              type="button"
                              className="evaluation__task-title"
                              onClick={() => {
                                setView({
                                  kind: "person",
                                  employeeId: task.employee_id,
                                  employeeName: task.employee_name,
                                });
                              }}
                            >
                              {`${task.employee_name} · ${text.reviewKind[task.kind]} — ${task.cycle_name}`}
                            </button>
                          ) : (
                            <span className="evaluation__task-title">
                              {`${task.employee_name} · ${text.reviewKind[task.kind]} — ${task.cycle_name}`}
                            </span>
                          )}
                          <button
                            type="button"
                            className="evaluation__solid"
                            disabled={busy}
                            onClick={() => {
                              setScorecard({
                                subjectId: task.subject_id,
                                cycleId: task.cycle_id,
                                kind: task.kind,
                                employeeName: task.employee_name,
                                cycleName: task.cycle_name,
                              });
                            }}
                          >
                            {text.write}
                          </button>
                        </li>
                      );
                    })
                  ) : (
                    <li role="status">{text.tasksEmpty}</li>
                  )}
                </ul>
              )}
            </section>
          )}
        </div>
        {capabilities.canRead && (
          <div className="evaluation__panel">
            {view.kind === "cycle" && (
              <CycleDetailZone
                api={api}
                capabilities={capabilities}
                busy={busy}
                selectedCycleId={selectedCycleId}
                data={detail.data}
                loading={detail.loading}
                error={detail.error}
                preflightReport={selectedCycleId ? preflight.data[selectedCycleId] : undefined}
                onRetry={() => {
                  if (selectedCycleId) loadDetail(selectedCycleId);
                }}
                onTransition={(target) => void runTransition(target)}
                onAddSubject={addSubject}
                onOpenSubject={(subjectId) => {
                  setView({ kind: "subject", subjectId });
                }}
                onOpenPerson={(employeeId, employeeName) => {
                  setView({ kind: "person", employeeId, employeeName });
                }}
              />
            )}
            {view.kind === "subject" && (
              <SubjectZone
                actorId={actorId}
                capabilities={capabilities}
                busy={busy}
                parentStage={parentStage}
                cycleName={detail.data?.id === subject.data?.cycle_id ? detail.data?.name : undefined}
                data={subject.data}
                error={subject.error}
                onBack={() => {
                  setView({ kind: "cycle" });
                }}
                onRetry={() => {
                  loadSubject(view.subjectId);
                }}
                onOpenPerson={(employeeId, employeeName) => {
                  setView({ kind: "person", employeeId, employeeName });
                }}
                onSaveGoals={(subjectId, goals) => void saveGoals(subjectId, goals)}
                onCalibrate={(subjectId, grade, reason) =>
                  void calibrate(subjectId, grade, reason)
                }
                onWriteManager={(subjectId, cycleId, employeeName, cycleName) => {
                  setScorecard({
                    subjectId,
                    cycleId,
                    kind: "MANAGER",
                    employeeName,
                    cycleName,
                  });
                }}
              />
            )}
            {view.kind === "person" && (
              <section className="evaluation__card" aria-label={text.history}>
                <div className="evaluation__zone-head">
                  <button
                    type="button"
                    onClick={() => {
                      setView({ kind: "cycle" });
                    }}
                  >
                    {text.back}
                  </button>
                  <h2>{view.employeeName}</h2>
                  <span className={CHIP_CLASS.info}>{text.auditChip}</span>
                  <button
                    type="button"
                    className="evaluation__link"
                    onClick={() => {
                      void navigate(consoleScreenPath("people"));
                    }}
                  >
                    {text.personDirectory}
                  </button>
                </div>
                <h3>{text.history}</h3>
                {ledger.error ? (
                  ledger.error.status === 403 ? (
                    <p role="status">{text.forbidden}</p>
                  ) : ledger.error.status === 404 ? (
                    <p role="status">{text.notFound}</p>
                  ) : (
                    <div className="evaluation__alert" role="alert">
                      <span>{ledger.error.message}</span>
                      <button
                        type="button"
                        onClick={() => {
                          loadLedger(view.employeeId);
                        }}
                      >
                        {text.retry}
                      </button>
                    </div>
                  )
                ) : ledger.loading && !ledger.data ? (
                  <p role="status">{text.loading}</p>
                ) : (
                  <ul className="evaluation__list" aria-label={text.history}>
                    {ledger.data?.length ? (
                      ledger.data.map((entry) => (
                        <li key={entry.rv_code}>
                          <button
                            type="button"
                            className="evaluation__row"
                            onClick={() => {
                              setView({ kind: "subject", subjectId: entry.subject_id });
                            }}
                          >
                            <span className="evaluation__row-main">
                              <span className={CHIP_CLASS.purple}>{text.ledgerChip}</span>
                              <span className="evaluation__code">{entry.rv_code}</span>
                              <span>{`${entry.cycle_name} · ${entry.period_label}`}</span>
                            </span>
                            <span className="evaluation__row-side">
                              <span className="evaluation__grade">{entry.final_grade}</span>
                              <span className="evaluation__mono">
                                {dateOnly(entry.finalized_at)}
                              </span>
                            </span>
                          </button>
                        </li>
                      ))
                    ) : (
                      <li role="status">{text.historyEmpty}</li>
                    )}
                  </ul>
                )}
              </section>
            )}
          </div>
        )}
      </div>
      {scorecard && (
        <ScorecardDialog
          open={scorecard}
          detail={card.data}
          loading={card.loading}
          error={card.error}
          busy={busy}
          actionError={actionError}
          onRetry={() => {
            void runCard((signal) => evaluationApi.getSubject(scorecard.subjectId, signal));
          }}
          onClose={() => {
            setScorecard(undefined);
            setActionError(undefined);
          }}
          onSaveDraft={(fields) => void saveScorecardDraft(fields)}
          onSubmit={(fields) => void submitScorecard(fields)}
        />
      )}
    </main>
  );
}

function CycleCreateForm({
  busy,
  onCreate,
}: {
  busy: boolean;
  onCreate: (event: SyntheticEvent<HTMLFormElement>) => Promise<void>;
}) {
  const nameId = useId();
  const kindId = useId();
  const periodId = useId();
  const dueId = useId();
  return (
    <form className="evaluation__form" onSubmit={(event) => void onCreate(event)}>
      <h3>{text.newCycle}</h3>
      <label htmlFor={nameId}>
        {text.cycleName}
        <input id={nameId} name="name" maxLength={120} required />
      </label>
      <label htmlFor={kindId}>
        {text.cycleKind}
        <select id={kindId} name="kind" defaultValue="REGULAR">
          <option value="REGULAR">{text.kind.REGULAR}</option>
          <option value="PROBATION">{text.kind.PROBATION}</option>
        </select>
      </label>
      <label htmlFor={periodId}>
        {text.periodLabel}
        <input id={periodId} name="period_label" maxLength={60} required />
      </label>
      <label htmlFor={dueId}>
        {text.dueDate}
        <input id={dueId} name="due_date" type="date" required />
      </label>
      <button type="submit" disabled={busy}>
        {text.createCycle}
      </button>
    </form>
  );
}

function CycleDetailZone({
  api,
  capabilities,
  busy,
  selectedCycleId,
  data,
  loading,
  error,
  preflightReport,
  onRetry,
  onTransition,
  onAddSubject,
  onOpenSubject,
  onOpenPerson,
}: {
  api: ConsoleApiClient;
  capabilities: EvaluationCapabilities;
  busy: boolean;
  selectedCycleId: string | undefined;
  data: EvaluationCycleDetail | undefined;
  loading: boolean;
  error: Failure | undefined;
  preflightReport: EvaluationPreflightReport | undefined;
  onRetry: () => void;
  onTransition: (target: EvaluationCycleStage) => void;
  onAddSubject: (employeeId: string, managerUserId: string) => Promise<boolean>;
  onOpenSubject: (subjectId: string) => void;
  onOpenPerson: (employeeId: string, employeeName: string) => void;
}) {
  if (!selectedCycleId) {
    return (
      <section className="evaluation__card">
        <p role="status">{text.selectCycle}</p>
      </section>
    );
  }
  if (error) {
    return (
      <section className="evaluation__card">
        {error.status === 403 ? (
          <p role="status">{text.forbidden}</p>
        ) : error.status === 404 ? (
          <p role="status">{text.notFound}</p>
        ) : (
          <div className="evaluation__alert" role="alert">
            <span>{error.message}</span>
            <button type="button" onClick={onRetry}>
              {text.retry}
            </button>
          </div>
        )}
      </section>
    );
  }
  if (!data || loading) {
    return (
      <section className="evaluation__card">
        <p role="status">{text.loading}</p>
      </section>
    );
  }
  const due = dueChip(data.due_date);
  const canEnroll =
    capabilities.canManage && (data.stage === "DRAFT" || data.stage === "OPEN");
  const nextTransition = capabilities.canManage
    ? (preflightReport?.next_transition ?? null)
    : null;
  const transitionStage = nextTransition ? transitionTarget(nextTransition) : undefined;
  const transitionLabel = transitionStage ? text.transition[transitionStage] : undefined;
  return (
    <section className="evaluation__card" aria-label={data.name}>
      <div className="evaluation__zone-head">
        <h2>{data.name}</h2>
        <span className={CHIP_CLASS.muted}>{text.kind[data.kind]}</span>
        <span className={CHIP_CLASS[STAGE_TONE[data.stage]]}>{stageLabel(data.stage)}</span>
        <span className={CHIP_CLASS.muted}>{data.period_label}</span>
        <span className={`${CHIP_CLASS[due.tone]} evaluation__mono`}>{due.label}</span>
      </div>
      <ul className="evaluation__stats">
        <li>
          <span>{text.stats.subjects}</span>
          <strong className="evaluation__mono">{data.subjects_total}</strong>
        </li>
        <li>
          <span>{text.stats.self}</span>
          <strong className="evaluation__mono">{data.self_submitted}</strong>
        </li>
        <li>
          <span>{text.stats.manager}</span>
          <strong className="evaluation__mono">{data.manager_submitted}</strong>
        </li>
        <li>
          <span>{text.stats.calibrated}</span>
          <strong className="evaluation__mono">{data.calibrated}</strong>
        </li>
        <li>
          <span>{text.stats.finalized}</span>
          <strong className="evaluation__mono">{data.finalized}</strong>
        </li>
      </ul>
      <h3>{text.teamProgress}</h3>
      {data.progress_by_unit.length ? (
        <ul className="evaluation__teams" aria-label={text.teamProgress}>
          {data.progress_by_unit.map((unit) => {
            const pct = unit.total
              ? Math.round((unit.manager_submitted / unit.total) * 100)
              : 0;
            return (
              <li key={unit.org_unit ?? "unassigned"}>
                <span className="evaluation__team-name">{unit.org_unit ?? "—"}</span>
                <span className="evaluation__bar">
                  <span
                    className={BAR_CLASS[progressTone(pct)]}
                    style={{ width: `${String(pct)}%` }}
                  />
                </span>
                <span className="evaluation__mono">{`${String(pct)}%`}</span>
              </li>
            );
          })}
        </ul>
      ) : (
        <p role="status">{text.teamProgressEmpty}</p>
      )}
      {nextTransition && transitionStage && transitionLabel && preflightReport && (
        <div className="evaluation__preflight">
          <button
            type="button"
            className="evaluation__solid"
            disabled={busy || preflightReport.blockers.length > 0}
            onClick={() => {
              onTransition(transitionStage);
            }}
          >
            {transitionLabel}
          </button>
          {preflightReport.blockers.length > 0 && (
            <ul className="evaluation__gate-list" aria-label={text.blockers}>
              {preflightReport.blockers.map((blocker) => (
                <li key={`${blocker.code}:${blocker.subject_id ?? "cycle"}`}>
                  <span className={CHIP_CLASS.danger}>{blocker.message}</span>
                </li>
              ))}
            </ul>
          )}
          {preflightReport.advisories.length > 0 && (
            <ul className="evaluation__gate-list" aria-label={text.advisories}>
              {preflightReport.advisories.map((advisory) => (
                <li key={`${advisory.code}:${advisory.subject_id ?? "cycle"}`}>
                  <span className={CHIP_CLASS.warn}>{advisory.message}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      <h3>{text.subjects}</h3>
      <ul className="evaluation__list" aria-label={text.subjects}>
        {data.subjects.length ? (
          data.subjects.map((row) => (
            <li key={row.id}>
              <button
                type="button"
                className="evaluation__row"
                onClick={() => {
                  onOpenSubject(row.id);
                }}
              >
                <span className="evaluation__row-main">
                  <strong>{row.employee_name}</strong>
                  {row.org_unit && <span className={CHIP_CLASS.muted}>{row.org_unit}</span>}
                </span>
                <span className="evaluation__row-side">
                  {row.rv_code && <span className="evaluation__code">{row.rv_code}</span>}
                  {row.final_grade && (
                    <span className="evaluation__grade">{row.final_grade}</span>
                  )}
                  <span className={CHIP_CLASS[SUBJECT_STATE_TONE[row.state]]}>
                    {stateLabel(row.state)}
                  </span>
                </span>
              </button>
            </li>
          ))
        ) : (
          <li role="status">{text.subjectsEmpty}</li>
        )}
      </ul>
      {canEnroll && <AddSubjectForm api={api} busy={busy} onAdd={onAddSubject} />}
      {data.subjects.length > 0 && (
        <ul className="evaluation__linkrow" aria-label={text.history}>
          {data.subjects.slice(0, 6).map((row) => (
            <li key={row.id}>
              <button
                type="button"
                className="evaluation__link"
                onClick={() => {
                  onOpenPerson(row.employee_id, row.employee_name);
                }}
              >
                {row.employee_name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function AddSubjectForm({
  api,
  busy,
  onAdd,
}: {
  api: ConsoleApiClient;
  busy: boolean;
  onAdd: (employeeId: string, managerUserId: string) => Promise<boolean>;
}) {
  const searchId = useId();
  const managerSelectId = useId();
  const [query, setQuery] = useState("");
  const [picked, setPicked] = useState<EmployeeHit>();
  const [manager, setManager] = useState("");
  const results = useFenced<EmployeeHit[]>();
  const managers = useFenced<ManagerOption[]>();
  const { run: runResults, reset: resetResults } = results;
  const { run: runManagers } = managers;

  useEffect(() => {
    void runManagers((signal) => listManagerOptions(api, signal));
  }, [api, runManagers]);

  useEffect(() => {
    const trimmed = query.trim();
    if (trimmed.length === 0 || picked) {
      resetResults();
      return;
    }
    const timer = window.setTimeout(() => {
      void runResults((signal) => searchEmployees(api, trimmed, signal));
    }, 200);
    return () => {
      window.clearTimeout(timer);
    };
  }, [query, picked, api, runResults, resetResults]);

  const submit = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!picked || !manager) return;
    const added = await onAdd(picked.id, manager);
    if (added) {
      setPicked(undefined);
      setQuery("");
      setManager("");
    }
  };

  return (
    <form className="evaluation__form" onSubmit={(event) => void submit(event)}>
      <h3>{text.addSubject}</h3>
      <label htmlFor={searchId}>
        {text.employeeSearch}
        <input
          id={searchId}
          value={picked ? picked.name : query}
          onChange={(event) => {
            setPicked(undefined);
            setQuery(event.currentTarget.value);
          }}
          autoComplete="off"
        />
      </label>
      {results.error && (
        <div className="evaluation__alert" role="alert">
          <span>{results.error.message}</span>
        </div>
      )}
      {!picked && (results.data?.length ?? 0) > 0 && (
        <ul className="evaluation__typeahead" aria-label={text.employeeResults}>
          {results.data?.map((hit) => (
            <li key={hit.id}>
              <button
                type="button"
                onClick={() => {
                  setPicked(hit);
                }}
              >
                <strong>{hit.name}</strong>
                {hit.employee_number && (
                  <span className="evaluation__mono">{hit.employee_number}</span>
                )}
                {hit.org_unit && <span className={CHIP_CLASS.muted}>{hit.org_unit}</span>}
              </button>
            </li>
          ))}
        </ul>
      )}
      <label htmlFor={managerSelectId}>
        {text.managerPick}
        <select
          id={managerSelectId}
          value={manager}
          onChange={(event) => {
            setManager(event.currentTarget.value);
          }}
          required
        >
          <option value="">{text.managerPlaceholder}</option>
          {managers.data?.map((option) => (
            <option key={option.id} value={option.id}>
              {option.display_name}
            </option>
          ))}
        </select>
      </label>
      {managers.error && (
        <div className="evaluation__alert" role="alert">
          <span>{managers.error.message}</span>
        </div>
      )}
      <button type="submit" disabled={busy || !picked || !manager}>
        {text.add}
      </button>
    </form>
  );
}

function SubjectZone({
  actorId,
  capabilities,
  busy,
  parentStage,
  cycleName,
  data,
  error,
  onBack,
  onRetry,
  onOpenPerson,
  onSaveGoals,
  onCalibrate,
  onWriteManager,
}: {
  actorId: string | undefined;
  capabilities: EvaluationCapabilities;
  busy: boolean;
  parentStage: EvaluationCycleStage | undefined;
  cycleName: string | undefined;
  data: EvaluationSubjectDetail | undefined;
  error: Failure | undefined;
  onBack: () => void;
  onRetry: () => void;
  onOpenPerson: (employeeId: string, employeeName: string) => void;
  onSaveGoals: (subjectId: string, goals: EvaluationGoalInput[]) => void;
  onCalibrate: (subjectId: string, grade: EvaluationGrade, reason: string) => void;
  onWriteManager: (
    subjectId: string,
    cycleId: string,
    employeeName: string,
    cycleName: string,
  ) => void;
}) {
  const [calGrade, setCalGrade] = useState<EvaluationGrade>();
  const [calReason, setCalReason] = useState("");
  if (error) {
    return (
      <section className="evaluation__card">
        <div className="evaluation__zone-head">
          <button type="button" onClick={onBack}>
            {text.back}
          </button>
        </div>
        {error.status === 403 ? (
          <p role="status">{text.forbidden}</p>
        ) : error.status === 404 ? (
          <p role="status">{text.notFound}</p>
        ) : (
          <div className="evaluation__alert" role="alert">
            <span>{error.message}</span>
            <button type="button" onClick={onRetry}>
              {text.retry}
            </button>
          </div>
        )}
      </section>
    );
  }
  if (!data) {
    return (
      <section className="evaluation__card">
        <div className="evaluation__zone-head">
          <button type="button" onClick={onBack}>
            {text.back}
          </button>
        </div>
        <p role="status">{text.loading}</p>
      </section>
    );
  }
  const managerReview = data.reviews.find((review) => review.kind === "MANAGER");
  const selfReview = data.reviews.find((review) => review.kind === "SELF");
  const goalsEditable =
    (capabilities.canManage || actorId === data.manager_user_id) &&
    (parentStage === "DRAFT" || parentStage === "OPEN");
  const canWriteManager =
    actorId === data.manager_user_id &&
    parentStage === "OPEN" &&
    managerReview?.status !== "SUBMITTED";
  const calibrationVisible =
    capabilities.canCalibrate &&
    parentStage === "CALIBRATION" &&
    (data.state === "REVIEWED" || data.state === "CALIBRATED") &&
    actorId !== managerReview?.evaluator_user_id;
  return (
    <section className="evaluation__card" aria-label={data.employee_name}>
      <div className="evaluation__zone-head">
        <button type="button" onClick={onBack}>
          {text.back}
        </button>
        <h2>
          <button
            type="button"
            className="evaluation__link"
            onClick={() => {
              onOpenPerson(data.employee_id, data.employee_name);
            }}
          >
            {data.employee_name}
          </button>
        </h2>
        {data.org_unit && <span className={CHIP_CLASS.muted}>{data.org_unit}</span>}
        <span className={CHIP_CLASS[SUBJECT_STATE_TONE[data.state]]}>
          {stateLabel(data.state)}
        </span>
        {data.rv_code && <span className="evaluation__code">{data.rv_code}</span>}
        {data.final_grade && <span className="evaluation__grade">{data.final_grade}</span>}
      </div>
      <h3>{text.goals}</h3>
      {goalsEditable ? (
        <GoalsEditor
          busy={busy}
          goals={data.goals}
          onSave={(goals) => {
            onSaveGoals(data.id, goals);
          }}
        />
      ) : data.goals.length ? (
        <ul className="evaluation__goal-list" aria-label={text.goals}>
          {[...data.goals]
            .sort((a, b) => a.sort_order - b.sort_order)
            .map((goal) => (
              <li key={goal.id}>
                <span className={CHIP_CLASS.muted}>{text.metricKind[goal.metric_kind]}</span>
                <strong>{goal.title}</strong>
                <span>{goal.target_label}</span>
                <span className="evaluation__mono">{`${String(goal.weight_pct)}%`}</span>
              </li>
            ))}
        </ul>
      ) : (
        <p role="status">{text.goalsEmpty}</p>
      )}
      <h3>{text.reviews}</h3>
      <div className="evaluation__reviews">
        {(
          [
            ["SELF", selfReview],
            ["MANAGER", managerReview],
          ] as const
        ).map(([kind, review]) => (
          <div className="evaluation__review" key={kind}>
            <div className="evaluation__review-head">
              <strong>{text.reviewKind[kind]}</strong>
              {review ? (
                <span className={CHIP_CLASS[review.status === "SUBMITTED" ? "ok" : "warn"]}>
                  {text.reviewStatus[review.status]}
                </span>
              ) : (
                <span className={CHIP_CLASS.muted}>{text.reviewStatus.missing}</span>
              )}
              {review?.grade && <span className="evaluation__grade">{review.grade}</span>}
              {review?.submitted_at && (
                <span className="evaluation__mono">{dateOnly(review.submitted_at)}</span>
              )}
              {kind === "MANAGER" && canWriteManager && (
                <button
                  type="button"
                  className="evaluation__solid"
                  disabled={busy}
                  onClick={() => {
                    onWriteManager(data.id, data.cycle_id, data.employee_name, cycleName ?? "");
                  }}
                >
                  {text.write}
                </button>
              )}
            </div>
            {review?.note && <p className="evaluation__note">{review.note}</p>}
            {review && review.evidence_links.length > 0 && (
              <ul className="evaluation__evidence" aria-label={text.evidence}>
                {[...review.evidence_links]
                  .sort((a, b) => a.sort_order - b.sort_order)
                  .map((link) => (
                    <li key={link.id}>
                      <EvidenceLinkDisplay
                        kind={link.object_kind}
                        objectRef={link.object_ref}
                        label={link.label}
                      />
                    </li>
                  ))}
              </ul>
            )}
          </div>
        ))}
      </div>
      {(data.calibrated_grade || calibrationVisible) && <h3>{text.calibration}</h3>}
      {data.calibrated_grade && (
        <div className="evaluation__zone-head">
          <span className="evaluation__grade">{data.calibrated_grade}</span>
          {data.calibration_reason && (
            <p className="evaluation__note">{data.calibration_reason}</p>
          )}
          {data.calibrated_at && (
            <span className="evaluation__mono">{dateOnly(data.calibrated_at)}</span>
          )}
        </div>
      )}
      {calibrationVisible && (
        <div className="evaluation__calibrate">
          <div className="evaluation__segment" role="group" aria-label={text.gradeLabel}>
            {GRADES.map((grade) => (
              <button
                key={grade}
                type="button"
                className={calGrade === grade ? "evaluation__segment-btn evaluation__segment-btn--on" : "evaluation__segment-btn"}
                aria-pressed={calGrade === grade}
                onClick={() => {
                  setCalGrade(grade);
                }}
              >
                {grade}
              </button>
            ))}
          </div>
          <textarea
            value={calReason}
            onChange={(event) => {
              setCalReason(event.currentTarget.value);
            }}
            maxLength={2000}
            placeholder={text.calibrationReason}
            aria-label={text.calibrationReason}
          />
          <button
            type="button"
            className="evaluation__solid"
            disabled={
              busy ||
              !calGrade ||
              (managerReview?.grade != null &&
                calGrade !== managerReview.grade &&
                calReason.trim().length === 0)
            }
            onClick={() => {
              if (calGrade) onCalibrate(data.id, calGrade, calReason.trim());
            }}
          >
            {text.calibrate}
          </button>
        </div>
      )}
    </section>
  );
}

function GoalsEditor({
  busy,
  goals,
  onSave,
}: {
  busy: boolean;
  goals: EvaluationSubjectDetail["goals"];
  onSave: (goals: EvaluationGoalInput[]) => void;
}) {
  const [rows, setRows] = useState<EvaluationGoalInput[]>(() =>
    [...goals]
      .sort((a, b) => a.sort_order - b.sort_order)
      .map((goal) => ({
        title: goal.title,
        metric_kind: goal.metric_kind,
        target_label: goal.target_label,
        weight_pct: goal.weight_pct,
      })),
  );
  const update = (index: number, patch: Partial<EvaluationGoalInput>) => {
    setRows((current) =>
      current.map((row, i) => (i === index ? { ...row, ...patch } : row)),
    );
  };
  const valid =
    rows.length <= 20 &&
    rows.every(
      (row) =>
        row.title.trim().length > 0 &&
        row.target_label.trim().length > 0 &&
        Number.isFinite(row.weight_pct) &&
        row.weight_pct >= 0 &&
        row.weight_pct <= 100,
    );
  const weightSum = rows.reduce(
    (sum, row) => sum + (Number.isFinite(row.weight_pct) ? row.weight_pct : 0),
    0,
  );
  return (
    <div className="evaluation__goal-editor">
      {rows.map((row, index) => (
        // ponytail: index keys — replace-set rows have no identity until saved.
        <div className="evaluation__goal-row" key={index}>
          <input
            value={row.title}
            maxLength={200}
            aria-label={text.goalTitle}
            placeholder={text.goalTitle}
            onChange={(event) => {
              update(index, { title: event.currentTarget.value });
            }}
          />
          <select
            value={row.metric_kind}
            aria-label={text.metric}
            onChange={(event) => {
              const metricKind = parseEvaluationMetricKind(event.currentTarget.value);
              if (metricKind) update(index, { metric_kind: metricKind });
            }}
          >
            <option value="KPI">{text.metricKind.KPI}</option>
            <option value="ATTENDANCE">{text.metricKind.ATTENDANCE}</option>
            <option value="TASK">{text.metricKind.TASK}</option>
            <option value="CUSTOM">{text.metricKind.CUSTOM}</option>
          </select>
          <input
            value={row.target_label}
            maxLength={200}
            aria-label={text.target}
            placeholder={text.target}
            onChange={(event) => {
              update(index, { target_label: event.currentTarget.value });
            }}
          />
          <input
            type="number"
            min={0}
            max={100}
            value={row.weight_pct}
            aria-label={text.weight}
            onChange={(event) => {
              update(index, { weight_pct: Number(event.currentTarget.value) });
            }}
          />
          <button
            type="button"
            onClick={() => {
              setRows((current) => current.filter((_, i) => i !== index));
            }}
          >
            {text.removeGoal}
          </button>
        </div>
      ))}
      <div className="evaluation__goal-actions">
        <button
          type="button"
          disabled={rows.length >= 20}
          onClick={() => {
            setRows((current) => [
              ...current,
              { title: "", metric_kind: "KPI", target_label: "", weight_pct: 0 },
            ]);
          }}
        >
          {text.addGoal}
        </button>
        <span className="evaluation__mono">{`${text.weightSum} ${String(weightSum)}%`}</span>
        <button
          type="button"
          className="evaluation__solid"
          disabled={busy || !valid}
          onClick={() => {
            onSave(rows);
          }}
        >
          {text.saveGoals}
        </button>
      </div>
    </div>
  );
}

function ScorecardDialog({
  open,
  detail,
  loading,
  error,
  busy,
  actionError,
  onRetry,
  onClose,
  onSaveDraft,
  onSubmit,
}: {
  open: { subjectId: string; kind: EvaluationReviewKind; employeeName: string; cycleName: string };
  detail: EvaluationSubjectDetail | undefined;
  loading: boolean;
  error: Failure | undefined;
  busy: boolean;
  actionError: Failure | undefined;
  onRetry: () => void;
  onClose: () => void;
  onSaveDraft: (fields: SaveEvaluationReviewRequest) => void;
  onSubmit: (fields: SaveEvaluationReviewRequest) => void;
}) {
  const dialogRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    const previous = document.activeElement;
    dialogRef.current?.focus();
    return () => {
      if (previous instanceof HTMLElement) previous.focus();
    };
  }, []);
  const title = open.cycleName
    ? `${open.employeeName} — ${open.cycleName}`
    : open.employeeName;
  return (
    <div
      className="evaluation__overlay"
      role="presentation"
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") onClose();
      }}
    >
      <div
        ref={dialogRef}
        className="evaluation__dialog"
        role="dialog"
        aria-modal="true"
        aria-label={text.scorecard}
        tabIndex={-1}
      >
        <div className="evaluation__dialog-head">
          <strong>{title}</strong>
          <span className={CHIP_CLASS.muted}>{text.objectTypeChip}</span>
          <span className={CHIP_CLASS.info}>{text.reviewKind[open.kind]}</span>
        </div>
        {actionError && (
          <div className="evaluation__alert" role="alert">
            <span>{actionError.message}</span>
          </div>
        )}
        {error ? (
          <div className="evaluation__alert" role="alert">
            <span>{error.message}</span>
            <button type="button" onClick={onRetry}>
              {text.retry}
            </button>
          </div>
        ) : loading || !detail ? (
          <p role="status">{text.loading}</p>
        ) : (
          <ScorecardForm
            key={`${detail.id}:${open.kind}`}
            kind={open.kind}
            detail={detail}
            busy={busy}
            onClose={onClose}
            onSaveDraft={onSaveDraft}
            onSubmit={onSubmit}
          />
        )}
      </div>
    </div>
  );
}

function ScorecardForm({
  kind,
  detail,
  busy,
  onClose,
  onSaveDraft,
  onSubmit,
}: {
  kind: EvaluationReviewKind;
  detail: EvaluationSubjectDetail;
  busy: boolean;
  onClose: () => void;
  onSaveDraft: (fields: SaveEvaluationReviewRequest) => void;
  onSubmit: (fields: SaveEvaluationReviewRequest) => void;
}) {
  const existing = detail.reviews.find((review) => review.kind === kind);
  const [grade, setGrade] = useState<EvaluationGrade | undefined>(existing?.grade ?? undefined);
  const [note, setNote] = useState(existing?.note ?? "");
  const [evidence, setEvidence] = useState<EvaluationEvidenceLinkInput[]>(
    () =>
      [...(existing?.evidence_links ?? [])]
        .sort((a, b) => a.sort_order - b.sort_order)
        .map((link) => ({
          object_kind: link.object_kind,
          object_ref: link.object_ref,
          label: link.label,
        })),
  );
  const [draftKind, setDraftKind] = useState<EvaluationEvidenceKind>("KPI");
  const [draftRef, setDraftRef] = useState("");
  const [draftLabel, setDraftLabel] = useState("");
  const fields = (): SaveEvaluationReviewRequest => ({
    ...(grade === undefined ? {} : { grade }),
    note: note.trim() ? note.trim() : null,
    evidence_links: evidence,
  });
  const submitDisabled =
    busy || !grade || (kind === "MANAGER" && evidence.length === 0);
  const goals = [...detail.goals].sort((a, b) => a.sort_order - b.sort_order);
  return (
    <div className="evaluation__scorecard">
      {goals.length > 0 && (
        <div className="evaluation__scorecard-section">
          <h3>{text.goals}</h3>
          <ul className="evaluation__goal-list" aria-label={text.goals}>
            {goals.map((goal) => (
              <li key={goal.id}>
                <span className={CHIP_CLASS.muted}>{text.metricKind[goal.metric_kind]}</span>
                <strong>{goal.title}</strong>
                <span>{goal.target_label}</span>
                <span className="evaluation__mono">{`${String(goal.weight_pct)}%`}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className="evaluation__scorecard-section">
        <h3>{text.evidence}</h3>
        {evidence.length ? (
          <ul className="evaluation__evidence" aria-label={text.evidence}>
            {evidence.map((link, index) => (
              <li key={`${link.object_ref}:${String(index)}`}>
                <EvidenceLinkDisplay
                  kind={link.object_kind}
                  objectRef={link.object_ref}
                  label={link.label}
                />
                <button
                  type="button"
                  onClick={() => {
                    setEvidence((current) => current.filter((_, i) => i !== index));
                  }}
                >
                  {text.removeEvidence}
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p role="status">{text.evidenceEmpty}</p>
        )}
        <div className="evaluation__evidence-add">
          <select
            value={draftKind}
            aria-label={text.evidence}
            onChange={(event) => {
              const evidenceKind = parseEvaluationEvidenceKind(event.currentTarget.value);
              if (evidenceKind) setDraftKind(evidenceKind);
            }}
          >
            <option value="ATTENDANCE">{text.evidenceKind.ATTENDANCE}</option>
            <option value="WORK_ORDER">{text.evidenceKind.WORK_ORDER}</option>
            <option value="APPROVAL">{text.evidenceKind.APPROVAL}</option>
            <option value="KPI">{text.evidenceKind.KPI}</option>
            <option value="OTHER">{text.evidenceKind.OTHER}</option>
          </select>
          <input
            value={draftRef}
            maxLength={120}
            aria-label={text.evidenceRef}
            placeholder={text.evidenceRef}
            onChange={(event) => {
              setDraftRef(event.currentTarget.value);
            }}
          />
          <input
            value={draftLabel}
            maxLength={200}
            aria-label={text.evidenceLabel}
            placeholder={text.evidenceLabel}
            onChange={(event) => {
              setDraftLabel(event.currentTarget.value);
            }}
          />
          <button
            type="button"
            disabled={
              evidence.length >= 10 ||
              draftRef.trim().length === 0 ||
              draftLabel.trim().length === 0
            }
            onClick={() => {
              setEvidence((current) => [
                ...current,
                {
                  object_kind: draftKind,
                  object_ref: draftRef.trim(),
                  label: draftLabel.trim(),
                },
              ]);
              setDraftRef("");
              setDraftLabel("");
            }}
          >
            {text.addEvidence}
          </button>
        </div>
      </div>
      <div className="evaluation__segment" role="group" aria-label={text.gradeLabel}>
        {GRADES.map((option) => (
          <button
            key={option}
            type="button"
            className={grade === option ? "evaluation__segment-btn evaluation__segment-btn--on" : "evaluation__segment-btn"}
            aria-pressed={grade === option}
            onClick={() => {
              setGrade(option);
            }}
          >
            {option}
          </button>
        ))}
      </div>
      <textarea
        value={note}
        maxLength={2000}
        rows={3}
        placeholder={text.notePlaceholder}
        aria-label={text.notePlaceholder}
        onChange={(event) => {
          setNote(event.currentTarget.value);
        }}
      />
      <div className="evaluation__dialog-foot">
        <button type="button" onClick={onClose}>
          {text.cancel}
        </button>
        <button
          type="button"
          disabled={busy}
          onClick={() => {
            onSaveDraft(fields());
          }}
        >
          {text.saveDraft}
        </button>
        <button
          type="button"
          className="evaluation__solid"
          disabled={submitDisabled}
          onClick={() => {
            onSubmit(fields());
          }}
        >
          {text.submit}
        </button>
      </div>
    </div>
  );
}
