import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";
import { useNavigate } from "react-router";

import type { ConsoleApiClient } from "../../api/client";
import { payrollStrings as text } from "../../i18n/payroll";
import {
  createPayrollApi,
  PayrollApiError,
  type ClosePreflight,
  type PayrollException,
  type PayrollRunDetail,
  type PayrollRunStatus,
  type PayrollRunSummary,
} from "./payrollApi";
import type { PayrollCapabilities } from "./payrollCapabilities";
import "./payroll.css";

type Props = {
  api: ConsoleApiClient;
  branchId: string;
  actorId: string | undefined;
  capabilities: PayrollCapabilities;
  /** Changes whenever auth replaces the effective tenant/session. */
  sessionKey: string | undefined;
};

const apiFenceIds = new WeakMap<object, number>();
let nextApiFenceId = 1;

function apiFenceKey(api: ConsoleApiClient): number {
  const reference = api as object;
  const existing = apiFenceIds.get(reference);
  if (existing) return existing;
  const id = nextApiFenceId++;
  apiFenceIds.set(reference, id);
  return id;
}

function message(cause: unknown, fallback: string): string {
  return cause instanceof Error ? cause.message : fallback;
}

// ---- run status machine ----------------------------------------------------

/** Pipeline position; legacy 0074 states map onto the closest stage. */
const STATUS_RANK: Record<string, number> = {
  STAGED: 0,
  VOID: 0,
  ATTENDANCE_CLOSED: 1,
  CALCULATING: 2,
  BLOCKED_LEGAL_GATE: 3,
  READY_FOR_REVIEW: 3,
  CALCULATED: 3,
  SUBMITTED: 4,
  REJECTED: 4,
  APPROVED: 5,
  DISBURSEMENT_SCHEDULED: 6,
  PAID: 7,
  ISSUED: 8,
};

function rank(status: string): number {
  return STATUS_RANK[status] ?? 0;
}

function statusLabel(status: string): string {
  if (status in text.status) return text.status[status as keyof typeof text.status];
  return text.status.unknown;
}

type Tone = "neutral" | "info" | "warn" | "danger" | "ok" | "purple";

const CHIP: Record<Tone, string> = {
  neutral: "payroll__chip payroll__chip--neutral",
  info: "payroll__chip payroll__chip--info",
  warn: "payroll__chip payroll__chip--warn",
  danger: "payroll__chip payroll__chip--danger",
  ok: "payroll__chip payroll__chip--ok",
  purple: "payroll__chip payroll__chip--purple",
};

function statusTone(status: PayrollRunStatus): Tone {
  switch (status) {
    case "SUBMITTED":
    case "CALCULATING":
      return "info";
    case "REJECTED":
    case "BLOCKED_LEGAL_GATE":
      return "danger";
    case "APPROVED":
    case "DISBURSEMENT_SCHEDULED":
    case "PAID":
    case "ISSUED":
      return "ok";
    default:
      return "neutral";
  }
}

const SEVERITY_TONE: Record<PayrollException["severity"], Tone> = {
  info: "info",
  warn: "warn",
  danger: "danger",
};

const NTS_TONE: Record<string, Tone> = {
  REQUIRED_NOT_SUPPLIED: "danger",
  SUPPLIED_UNVERIFIED: "warn",
  VERIFIED_SOURCE_ROW: "ok",
};

function ntsLabel(status: string): string {
  if (status in text.ntsStatus) return text.ntsStatus[status as keyof typeof text.ntsStatus];
  return text.ntsStatus.unknown;
}

const LINE_CALC_TONE: Record<string, Tone> = {
  BLOCKED_LEGAL_GATE: "danger",
  READY_FOR_REVIEW: "info",
  APPROVED: "ok",
  ISSUED: "ok",
  VOID: "neutral",
};

const DISB_TONE: Record<string, Tone> = {
  SCHEDULED: "info",
  SUBMITTED_TO_BANK: "info",
  PAID: "ok",
  FAILED: "danger",
};

function lineCalcLabel(status: string): string {
  if (status in text.lineCalcStatus) {
    return text.lineCalcStatus[status as keyof typeof text.lineCalcStatus];
  }
  return text.lineCalcStatus.unknown;
}

function disbLabel(status: string): string {
  if (status in text.disbStatus) return text.disbStatus[status as keyof typeof text.disbStatus];
  return text.disbStatus.unknown;
}

function exKindLabel(kind: string): string {
  if (kind in text.exKind) return text.exKind[kind as keyof typeof text.exKind];
  return text.exKind.unknown;
}

// ---- formatting ------------------------------------------------------------

const NF = new Intl.NumberFormat("ko-KR");

function won(value: number): string {
  return (value < 0 ? "−₩" : "₩") + NF.format(Math.abs(value));
}

function deltaWon(value: number): string {
  return value > 0 ? `+${won(value)}` : won(value);
}

const TIME_FORMAT = new Intl.DateTimeFormat("ko-KR", {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  hour12: false,
});

/** Local-time rendering; the raw value passes through when it is not a date. */
function timeText(iso: string): string {
  const at = new Date(iso);
  return Number.isNaN(at.getTime()) ? iso : TIME_FORMAT.format(at);
}

function count(value: number | null | undefined): string {
  return value == null ? "–" : String(value);
}

// ---- per-user view persistence (personal whitelist, never audited) ---------

type ColKey = "entity" | "work" | "overtime" | "sources" | "calc";

const DEFAULT_COL_W: Record<ColKey, number> = {
  entity: 64,
  work: 52,
  overtime: 52,
  sources: 172,
  calc: 156,
};

interface ViewPrefs {
  runId?: string;
  mask?: boolean;
  colW?: Partial<Record<ColKey, number>>;
}

function clampColW(value: number): number {
  return Math.min(220, Math.max(36, Math.round(value)));
}

function prefsKey(actorId: string | undefined): string {
  return `payroll:view:${actorId ?? "anon"}`;
}

function readPrefs(actorId: string | undefined): ViewPrefs {
  try {
    const raw = window.localStorage.getItem(prefsKey(actorId));
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    const prefs: ViewPrefs = {};
    if (typeof parsed.runId === "string") prefs.runId = parsed.runId;
    if (typeof parsed.mask === "boolean") prefs.mask = parsed.mask;
    if (parsed.colW && typeof parsed.colW === "object") {
      const widths: Partial<Record<ColKey, number>> = {};
      for (const key of Object.keys(DEFAULT_COL_W) as ColKey[]) {
        const value = (parsed.colW as Record<string, unknown>)[key];
        if (typeof value === "number" && Number.isFinite(value)) widths[key] = clampColW(value);
      }
      prefs.colW = widths;
    }
    return prefs;
  } catch {
    return {};
  }
}

function writePrefs(actorId: string | undefined, prefs: ViewPrefs): void {
  try {
    window.localStorage.setItem(prefsKey(actorId), JSON.stringify(prefs));
  } catch {
    // Personal view only; losing it is harmless.
  }
}

/** Traversal target per linked-ref kind; unknown kinds land in the explorer. */
const REF_ROUTE: Record<string, string> = {
  person: "/people",
  attendance: "/attendance",
  approval: "/appr",
  inbox: "/inbox",
  thread: "/messenger",
};

function refRoute(kind: string): string {
  return REF_ROUTE[kind] ?? "/objectExplorer";
}

function refKindLabel(kind: string): string {
  if (kind in text.refKind) return text.refKind[kind as keyof typeof text.refKind];
  return kind;
}

function detailLines(detail: unknown): string[] {
  return Array.isArray(detail) ? detail.filter((line): line is string => typeof line === "string") : [];
}

interface PreflightState {
  open: boolean;
  loading: boolean;
  data?: ClosePreflight;
  error?: string;
  attested: boolean;
}

const PREFLIGHT_CLOSED: PreflightState = { open: false, loading: false, attested: false };

/**
 * Re-mount synchronously whenever effective authority changes. Effects run too
 * late to fence an old tenant/session's selected run, error, or busy state.
 */
export function PayrollScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.branchId,
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <PayrollScreenInner key={sessionFence} {...props} />;
}

function PayrollScreenInner({ api, actorId, capabilities, sessionKey }: Props) {
  const navigate = useNavigate();
  const payrollApi = useMemo(() => createPayrollApi(api), [api]);
  const initialPrefs = useMemo(() => readPrefs(actorId), [actorId]);

  const [runs, setRuns] = useState<PayrollRunSummary[]>([]);
  const [runsLoading, setRunsLoading] = useState(true);
  const [runsError, setRunsError] = useState<string>();
  const [selectedId, setSelectedId] = useState<string | undefined>(initialPrefs.runId);
  const [detail, setDetail] = useState<PayrollRunDetail>();
  const [exceptions, setExceptions] = useState<PayrollException[]>([]);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailError, setDetailError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string>();
  const [maskOn, setMaskOn] = useState(initialPrefs.mask ?? true);
  const [colW, setColW] = useState<Record<ColKey, number>>({ ...DEFAULT_COL_W, ...initialPrefs.colW });
  const [query, setQuery] = useState("");
  const [openLineId, setOpenLineId] = useState<string>();
  const [openExId, setOpenExId] = useState<string>();
  const [holdExId, setHoldExId] = useState<string>();
  const [holdReason, setHoldReason] = useState("");
  const [preflight, setPreflight] = useState<PreflightState>(PREFLIGHT_CLOSED);
  const [scheduleAt, setScheduleAt] = useState("");
  const [failReason, setFailReason] = useState("");
  const [decisionReason, setDecisionReason] = useState("");

  const generation = useRef(0);
  const operation = useRef<AbortController | undefined>(undefined);
  const rowRefs = useRef<(HTMLDivElement | null)[]>([]);
  const scheduleInputRef = useRef<HTMLInputElement | null>(null);
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const preflightReturnFocus = useRef<HTMLElement | null>(null);
  const dragState = useRef<{ key: ColKey; startX: number; startW: number }>(undefined);

  const isCurrent = useCallback((token: number) => generation.current === token, []);

  useEffect(() => {
    writePrefs(actorId, { runId: selectedId, mask: maskOn, colW });
  }, [actorId, selectedId, maskOn, colW]);

  const loadRuns = useCallback(async () => {
    if (!capabilities.canRead) {
      setRuns([]);
      setRunsLoading(false);
      return;
    }
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    setRunsLoading(true);
    setRunsError(undefined);
    try {
      const page = await payrollApi.listRuns(controller.signal);
      if (isCurrent(token)) {
        setRuns(page.items);
        setSelectedId((current) => {
          if (current && page.items.some((run) => run.id === current)) return current;
          if (page.items.length === 0) return undefined;
          const active = page.items.find((run) => run.status !== "ISSUED" && run.status !== "VOID");
          return (active ?? page.items[0]).id;
        });
      }
    } catch (cause) {
      if (isCurrent(token) && !controller.signal.aborted) setRunsError(message(cause, text.loadError));
    } finally {
      if (isCurrent(token)) setRunsLoading(false);
    }
  }, [capabilities.canRead, isCurrent, payrollApi]);

  const loadDetail = useCallback(async (runId: string, quiet = false) => {
    if (!capabilities.canRead) return;
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    if (!quiet) {
      setDetailLoading(true);
      setDetailError(undefined);
    }
    try {
      const [next, page] = await Promise.all([
        payrollApi.getRun(runId, controller.signal),
        payrollApi.listExceptions(runId, controller.signal),
      ]);
      if (isCurrent(token)) {
        setDetail(next);
        setExceptions(page.items);
      }
    } catch (cause) {
      if (isCurrent(token) && !controller.signal.aborted) setDetailError(message(cause, text.loadError));
    } finally {
      if (isCurrent(token) && !quiet) setDetailLoading(false);
    }
  }, [capabilities.canRead, isCurrent, payrollApi]);

  useEffect(() => {
    generation.current += 1;
    operation.current?.abort();
    const start = window.setTimeout(() => {
      void loadRuns();
    }, 0);
    return () => {
      window.clearTimeout(start);
      operation.current?.abort();
    };
  }, [loadRuns, sessionKey]);

  useEffect(() => {
    if (!selectedId) return;
    setOpenLineId(undefined);
    setOpenExId(undefined);
    setHoldExId(undefined);
    setQuery("");
    void loadDetail(selectedId);
  }, [selectedId, loadDetail]);

  // The run list mirrors the freshest detail the server returned.
  useEffect(() => {
    if (!detail) return;
    setRuns((current) => current.map((run) => (run.id === detail.run.id ? detail.run : run)));
  }, [detail]);

  const status = detail?.run.status;
  useEffect(() => {
    if (status !== "CALCULATING" || !selectedId) return;
    const timer = window.setInterval(() => {
      void loadDetail(selectedId, true);
    }, 4000);
    return () => { window.clearInterval(timer); };
  }, [status, selectedId, loadDetail]);

  useEffect(() => {
    if (preflight.open) dialogRef.current?.focus();
  }, [preflight.open]);

  const act = useCallback(async (
    work: (signal: AbortSignal) => Promise<unknown>,
  ): Promise<{ ok: boolean; error?: PayrollApiError }> => {
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    setBusy(true);
    setActionError(undefined);
    try {
      await work(controller.signal);
      return { ok: isCurrent(token) };
    } catch (cause) {
      if (isCurrent(token) && !controller.signal.aborted) setActionError(message(cause, text.actionError));
      return { ok: false, error: cause instanceof PayrollApiError ? cause : undefined };
    } finally {
      if (isCurrent(token)) setBusy(false);
    }
  }, [isCurrent]);

  const runAction = useCallback(async (
    work: (signal: AbortSignal) => Promise<unknown>,
  ): Promise<{ ok: boolean; error?: PayrollApiError }> => {
    const result = await act(work);
    if (result.ok && selectedId) await loadDetail(selectedId, true);
    return result;
  }, [act, loadDetail, selectedId]);

  const closePreflightDialog = useCallback(() => {
    setPreflight(PREFLIGHT_CLOSED);
    preflightReturnFocus.current?.focus();
    preflightReturnFocus.current = null;
  }, []);

  // The attestation covers the checks currently shown, so every (re)fetch clears it.
  const openPreflight = useCallback(async () => {
    if (!selectedId) return;
    setPreflight((current) => {
      if (!current.open && document.activeElement instanceof HTMLElement) {
        preflightReturnFocus.current = document.activeElement;
      }
      return { ...current, open: true, loading: true, error: undefined, attested: false };
    });
    try {
      const data = await payrollApi.closePreflight(selectedId);
      setPreflight((current) => (current.open ? { ...current, loading: false, data } : current));
    } catch (cause) {
      setPreflight((current) => (
        current.open
          ? { ...current, loading: false, error: message(cause, text.preflightError) }
          : current
      ));
    }
  }, [payrollApi, selectedId]);

  const confirmClose = async () => {
    if (!selectedId) return;
    const result = await runAction((signal) => payrollApi.closeAttendance(selectedId, signal));
    if (result.ok) closePreflightDialog();
    else if (result.error?.code === "preflight_blocked") await openPreflight();
  };

  const trapDialogFocus = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Escape") {
      closePreflightDialog();
      return;
    }
    if (event.key !== "Tab") return;
    const dialog = dialogRef.current;
    if (!dialog) return;
    const focusables = dialog.querySelectorAll<HTMLElement>(
      "button:not(:disabled), input:not(:disabled), [tabindex]:not([tabindex='-1'])",
    );
    if (focusables.length === 0) return;
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (event.shiftKey && (document.activeElement === first || document.activeElement === dialog)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  };

  const resolveException = async (exception: PayrollException, action: "CONFIRM" | "HOLD", reason?: string) => {
    if (!selectedId) return;
    const trimmed = reason?.trim();
    const result = await runAction((signal) => payrollApi.resolveException(
      selectedId,
      exception.id,
      { action, ...(trimmed ? { reason: trimmed } : {}) },
      signal,
    ));
    if (result.ok) {
      setHoldExId(undefined);
      setHoldReason("");
      setOpenExId(undefined);
    }
  };

  const schedule = async () => {
    if (!selectedId || !scheduleAt) return;
    const result = await runAction((signal) => payrollApi.scheduleDisbursement(
      selectedId,
      new Date(scheduleAt).toISOString(),
      signal,
    ));
    if (result.ok) setScheduleAt("");
  };

  const attest = async (nextStatus: "SUBMITTED_TO_BANK" | "PAID" | "FAILED") => {
    if (!selectedId) return;
    const reason = failReason.trim();
    if (nextStatus === "FAILED" && !reason) return;
    const result = await runAction((signal) => payrollApi.attestDisbursement(
      selectedId,
      { status: nextStatus, ...(nextStatus === "FAILED" ? { reason } : {}) },
      signal,
    ));
    if (result.ok) setFailReason("");
  };

  const decide = async (decision: "APPROVE" | "REJECT") => {
    if (!selectedId) return;
    const reason = decisionReason.trim();
    if (decision === "REJECT" && !reason) return;
    const result = await runAction((signal) => payrollApi.decide(
      selectedId,
      { decision, ...(reason ? { reason } : {}) },
      signal,
    ));
    if (result.ok) setDecisionReason("");
  };

  const startColDrag = (key: ColKey, event: ReactPointerEvent<HTMLSpanElement>) => {
    event.preventDefault();
    dragState.current = { key, startX: event.clientX, startW: colW[key] };
    const move = (pointer: PointerEvent) => {
      const drag = dragState.current;
      if (!drag) return;
      setColW((current) => ({ ...current, [drag.key]: clampColW(drag.startW + pointer.clientX - drag.startX) }));
    };
    const up = () => {
      dragState.current = undefined;
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  const rosterKeys = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    const key = event.key.toLowerCase();
    if (key !== "j" && key !== "k") return;
    const rows = rowRefs.current.filter((row): row is HTMLDivElement => row != null);
    if (rows.length === 0) return;
    event.preventDefault();
    const active = rows.findIndex((row) => row === document.activeElement);
    const next = key === "j" ? Math.min(active + 1, rows.length - 1) : Math.max(active - 1, 0);
    rows[next]?.focus();
  };

  const run = detail?.run;
  const stage = status ? rank(status) : 0;
  const calced = detail != null && status !== undefined && stage >= 3 && status !== "CALCULATING";
  const calc = detail?.calculation ?? null;
  const disbursement = detail?.disbursement ?? null;
  const delivery = detail?.payslip_delivery ?? null;
  const exceptionsOpen = detail?.exceptions_open ?? 0;
  const exceptionsTotal = detail?.exceptions_total ?? 0;

  const openEmployeeIds = useMemo(() => new Set(
    exceptions.filter((ex) => ex.status === "OPEN" && ex.employee_id != null).map((ex) => ex.employee_id),
  ), [exceptions]);
  const heldEmployeeIds = useMemo(() => new Set(
    exceptions.filter((ex) => ex.status === "HELD" && ex.employee_id != null).map((ex) => ex.employee_id),
  ), [exceptions]);
  const openExceptionByEmployee = useMemo(() => {
    const index = new Map<string, PayrollException>();
    for (const ex of exceptions) {
      if (ex.status === "OPEN" && ex.employee_id != null && !index.has(ex.employee_id)) {
        index.set(ex.employee_id, ex);
      }
    }
    return index;
  }, [exceptions]);
  const accountOpen = exceptions.filter((ex) => ex.kind === "ACCOUNT_VERIFICATION" && ex.status === "OPEN").length;
  const accountHeld = exceptions.filter((ex) => ex.kind === "ACCOUNT_VERIFICATION" && ex.status === "HELD").length;

  const shownLines = useMemo(() => {
    const lines = detail?.lines ?? [];
    const needle = query.trim().toLowerCase();
    const filtered = needle
      ? lines.filter((line) =>
          line.employee_display_name.toLowerCase().includes(needle)
          || line.employee_company.toLowerCase().includes(needle))
      : lines;
    // Single query point: unresolved-exception rows surface first.
    return [...filtered].sort((a, b) =>
      Number(b.employee_id != null && openEmployeeIds.has(b.employee_id))
      - Number(a.employee_id != null && openEmployeeIds.has(a.employee_id)));
  }, [detail, query, openEmployeeIds]);
  rowRefs.current.length = shownLines.length;

  const sortedExceptions = useMemo(() => [...exceptions].sort(
    (a, b) => Number(a.status !== "OPEN") - Number(b.status !== "OPEN"),
  ), [exceptions]);

  const gridCols = `minmax(100px, 1.2fr) ${String(colW.entity)}px ${String(colW.work)}px ${String(colW.overtime)}px ${String(colW.sources)}px ${String(colW.calc)}px`;
  const isCustomLayout = (Object.keys(DEFAULT_COL_W) as ColKey[]).some((key) => colW[key] !== DEFAULT_COL_W[key]);

  if (!capabilities.canRead) {
    return (
      <main className="payroll">
        <section className="payroll__card" aria-labelledby="payroll-title">
          <h1 id="payroll-title">{text.title}</h1>
          <p role="status">{text.denied}</p>
        </section>
      </main>
    );
  }

  const headerChip = run ? (() => {
    switch (run.status) {
      case "SUBMITTED": return <span className={CHIP.info}>{text.chipAwaitingDecision}</span>;
      case "REJECTED": return <span className={CHIP.danger}>{text.chipRejected}</span>;
      case "DISBURSEMENT_SCHEDULED": return <span className={CHIP.ok}>{text.chipReady}</span>;
      case "ISSUED": return <span className={CHIP.ok}>{text.chipIssued}</span>;
      default: return <span className={CHIP[statusTone(run.status)]}>{statusLabel(run.status)}</span>;
    }
  })() : null;

  const cta = run && capabilities.canManage ? (() => {
    switch (run.status) {
      case "STAGED":
        return <button className="payroll__btn payroll__btn--ghost" type="button" disabled={busy} onClick={() => void openPreflight()}>{text.ctaClose}</button>;
      case "ATTENDANCE_CLOSED":
        return <button className="payroll__btn payroll__btn--primary" type="button" disabled={busy} onClick={() => void runAction((signal) => payrollApi.calculate(run.id, signal))}>{text.ctaCalc}</button>;
      case "CALCULATING":
        return <button className="payroll__btn payroll__btn--primary payroll__btn--pulse" type="button" disabled>{text.ctaCalcRunning}</button>;
      case "CALCULATED":
        return exceptionsOpen > 0
          ? <span className={CHIP.warn}>{text.chipExceptionsLeft(exceptionsOpen)}</span>
          : <button className="payroll__btn payroll__btn--primary" type="button" disabled={busy} onClick={() => void runAction((signal) => payrollApi.submit(run.id, signal))}>{text.ctaSubmit}</button>;
      case "SUBMITTED":
        return <button className="payroll__btn payroll__btn--ghost" type="button" onClick={() => { void navigate("/appr"); }}>{text.ctaOpenApproval}</button>;
      case "REJECTED":
        return <button className="payroll__btn payroll__btn--ghost" type="button" disabled={busy} onClick={() => void runAction((signal) => payrollApi.withdraw(run.id, signal))}>{text.ctaWithdraw}</button>;
      case "APPROVED":
        return <button className="payroll__btn payroll__btn--primary" type="button" onClick={() => { scheduleInputRef.current?.scrollIntoView({ block: "center" }); scheduleInputRef.current?.focus(); }}>{text.ctaSchedule}</button>;
      case "PAID":
        return <button className="payroll__btn payroll__btn--primary" type="button" disabled={busy} onClick={() => void runAction((signal) => payrollApi.issuePayslips(run.id, signal))}>{text.ctaIssue}</button>;
      default:
        return null;
    }
  })() : null;

  const steps: { key: string; label: string; sub: string; state: "done" | "active" | "wait" | "reject" | "locked"; onClick?: () => void }[] = run ? [
    {
      key: "close",
      label: text.steps.close,
      sub: stage >= 1 ? text.stepCloseDone : text.stepCloseActive,
      state: stage >= 1 ? "done" : "active",
      ...(run.status === "STAGED" ? { onClick: () => { void navigate("/attendance"); } } : {}),
    },
    {
      key: "calc",
      label: text.steps.calc,
      sub: stage >= 3 && run.status !== "CALCULATING"
        ? text.stepCalcDone(calc?.calculated_lines ?? 0)
        : run.status === "CALCULATING" ? text.stepCalcRunning
        : run.status === "ATTENDANCE_CLOSED" ? text.stepCalcActive
        : text.stepCalcLocked,
      state: stage >= 3 && run.status !== "CALCULATING" ? "done"
        : run.status === "CALCULATING" || run.status === "ATTENDANCE_CLOSED" ? "active"
        : "locked",
    },
    {
      key: "exceptions",
      label: text.steps.exceptions,
      sub: !calced ? text.stepExLocked
        : exceptionsOpen > 0 ? text.stepExOpen(exceptionsOpen)
        : text.stepExDone(exceptionsTotal),
      state: !calced ? "locked" : exceptionsOpen > 0 ? "active" : "done",
    },
    {
      key: "approval",
      label: text.steps.approval,
      sub: stage >= 5 ? text.stepApprovalApproved
        : run.status === "SUBMITTED" ? text.stepApprovalPending
        : run.status === "REJECTED" ? text.stepApprovalRejected
        : text.stepApprovalLocked,
      state: stage >= 5 ? "done"
        : run.status === "SUBMITTED" ? "active"
        : run.status === "REJECTED" ? "reject"
        : "locked",
      ...(run.status === "SUBMITTED" ? { onClick: () => { void navigate("/appr"); } } : {}),
    },
    {
      key: "transfer",
      label: text.steps.transfer,
      sub: run.status === "ISSUED" ? text.stepTransferIssued
        : run.status === "PAID" ? text.stepTransferPaid
        : disbursement?.status === "FAILED" ? text.stepTransferResubmit
        : disbursement ? text.stepTransferScheduled(timeText(disbursement.scheduled_at))
        : text.stepTransferLocked,
      state: run.status === "PAID" || run.status === "ISSUED" ? "done"
        : disbursement?.status === "FAILED" ? "reject"
        : run.status === "DISBURSEMENT_SCHEDULED" ? "active"
        : "locked",
    },
  ] : [];

  const stepStateClass: Record<string, string> = {
    done: "payroll__step payroll__step--done",
    active: "payroll__step payroll__step--active",
    wait: "payroll__step payroll__step--wait",
    reject: "payroll__step payroll__step--reject",
    locked: "payroll__step payroll__step--locked",
  };

  const milestones: { key: string; label: string; state: "done" | "now" | "wait"; at?: string; body?: ReactNode }[] = run ? [
    {
      key: "calc",
      label: text.msCalc,
      state: calced && exceptionsOpen === 0 ? "done" : stage >= 1 ? "now" : "wait",
      ...(calc ? { at: calc.calculated_at } : {}),
    },
    {
      key: "approval",
      label: text.msApproval,
      state: stage >= 5 ? "done" : run.status === "SUBMITTED" || run.status === "REJECTED" ? "now" : "wait",
      ...(run.decided_at ? { at: run.decided_at } : run.submitted_at ? { at: run.submitted_at } : {}),
      body: (
        <>
          {run.status === "REJECTED" && run.decision_reason ? <p className="payroll__msnote">{run.decision_reason}</p> : null}
          {run.status === "SUBMITTED" && capabilities.canDecide ? (
            <div className="payroll__msacts">
              <input
                aria-label={text.decisionReason}
                placeholder={text.decisionReason}
                value={decisionReason}
                onChange={(event) => { setDecisionReason(event.target.value); }}
              />
              <button className="payroll__btn payroll__btn--quiet" type="button" disabled={busy} onClick={() => void decide("APPROVE")}>{text.decisionApprove}</button>
              <button className="payroll__btn payroll__btn--danger" type="button" disabled={busy || !decisionReason.trim()} onClick={() => void decide("REJECT")}>{text.decisionReject}</button>
            </div>
          ) : null}
        </>
      ),
    },
    {
      key: "transfer",
      label: text.msTransfer,
      state: run.status === "PAID" || run.status === "ISSUED" ? "done" : run.status === "DISBURSEMENT_SCHEDULED" ? "now" : "wait",
      ...(disbursement ? { at: disbursement.scheduled_at } : {}),
      body: (
        <>
          {disbursement ? (
            <p className="payroll__msrow">
              <span className={CHIP[DISB_TONE[disbursement.status] ?? "neutral"]}>{disbLabel(disbursement.status)}</span>
              {disbursement.status === "FAILED" && disbursement.reason ? <span className="payroll__msnote">{disbursement.reason}</span> : null}
            </p>
          ) : null}
          {capabilities.canManage && (run.status === "APPROVED" || disbursement?.status === "FAILED") ? (
            <div className="payroll__msacts">
              <label>
                {text.scheduleAt}
                <input
                  ref={scheduleInputRef}
                  type="datetime-local"
                  value={scheduleAt}
                  onChange={(event) => { setScheduleAt(event.target.value); }}
                />
              </label>
              <button className="payroll__btn payroll__btn--quiet" type="button" disabled={busy || !scheduleAt} onClick={() => void schedule()}>{text.scheduleConfirm}</button>
            </div>
          ) : null}
          {capabilities.canManage && disbursement?.status === "SCHEDULED" ? (
            <div className="payroll__msacts">
              <button className="payroll__btn payroll__btn--quiet" type="button" disabled={busy} onClick={() => void attest("SUBMITTED_TO_BANK")}>{text.attestBank}</button>
            </div>
          ) : null}
          {capabilities.canManage && disbursement?.status === "SUBMITTED_TO_BANK" ? (
            <div className="payroll__msacts">
              <button className="payroll__btn payroll__btn--quiet" type="button" disabled={busy} onClick={() => void attest("PAID")}>{text.attestPaid}</button>
              <input
                aria-label={text.attestFailReason}
                placeholder={text.attestFailReason}
                value={failReason}
                onChange={(event) => { setFailReason(event.target.value); }}
              />
              <button className="payroll__btn payroll__btn--danger" type="button" disabled={busy || !failReason.trim()} onClick={() => void attest("FAILED")}>{text.attestFailed}</button>
            </div>
          ) : null}
        </>
      ),
    },
    {
      key: "payslips",
      label: text.msPayslips,
      state: run.status === "ISSUED" ? "done" : run.status === "PAID" ? "now" : "wait",
      body: delivery ? (
        <p className="payroll__msrow">
          <span className={CHIP.ok}>{text.deliveryIssued(delivery.issued)}</span>
          <span className={CHIP.info}>{text.deliveryAck(delivery.acknowledged)}</span>
          <button className={CHIP.purple} type="button" onClick={() => { void navigate("/inbox"); }}>{text.inboxLink}</button>
        </p>
      ) : null,
    },
  ] : [];

  const milestoneChip: Record<"done" | "now" | "wait", ReactNode> = {
    done: <span className={CHIP.ok}>{text.msChipDone}</span>,
    now: <span className={CHIP.info}>{text.msChipNow}</span>,
    wait: <span className={CHIP.neutral}>{text.msChipWait}</span>,
  };

  const totalsTag = calc == null ? text.totalsTagPre : calc.total_net_won != null ? text.totalsTagFinal : text.totalsTagPartial;

  return (
    <main className="payroll" aria-busy={runsLoading || detailLoading || busy}>
      <header className="payroll__head">
        <div className="payroll__headline">
          <h1 id="payroll-title">{text.title}</h1>
          {detail ? (
            <p className="payroll__subline">
              <span>{detail.run.source_label}</span>
              <span>{text.sublinePeriod(detail.run.period_start, detail.run.period_end)}</span>
              {disbursement ? <span>{text.sublinePayDate(timeText(disbursement.scheduled_at))}</span> : null}
              <span>{text.sublineTarget(detail.lines_total)}</span>
            </p>
          ) : null}
        </div>
        <div className="payroll__spacer" />
        {isCustomLayout ? (
          <button className="payroll__btn payroll__btn--quiet" type="button" onClick={() => { setColW({ ...DEFAULT_COL_W }); }}>{text.resetLayout}</button>
        ) : null}
        {headerChip}
        {cta}
      </header>

      {actionError ? (
        <div className="payroll__alert" role="alert"><span>{actionError}</span></div>
      ) : null}

      {runsError ? (
        <div className="payroll__alert" role="alert">
          <span>{runsError}</span>
          <button type="button" onClick={() => { void loadRuns(); }}>{text.retry}</button>
        </div>
      ) : runsLoading ? (
        <p role="status">{text.loadingRuns}</p>
      ) : runs.length === 0 ? (
        <p role="status">{text.emptyRuns}</p>
      ) : (
        <>
          <ol className="payroll__stepper" aria-label={text.stepper}>
            {steps.map((step, index) => {
              const inner = (
                <>
                  <span className="payroll__stepdot" aria-hidden="true">{step.state === "done" ? "✓" : String(index + 1)}</span>
                  <span className="payroll__steptext">
                    <span>{step.label}</span>
                    <span className="payroll__stepsub">{step.sub}</span>
                  </span>
                </>
              );
              return (
                <li key={step.key} className={stepStateClass[step.state]}>
                  {step.onClick
                    ? <button className="payroll__stepbtn" type="button" onClick={step.onClick}>{inner}</button>
                    : inner}
                </li>
              );
            })}
          </ol>

          <div className="payroll__zones">
            <section className="payroll__card payroll__runs" aria-label={text.runList}>
              <h2>{text.runsTitle}</h2>
              <ul>
                {runs.map((item) => (
                  <li key={item.id}>
                    <button
                      className={item.id === selectedId ? "payroll__run payroll__run--selected" : "payroll__run"}
                      type="button"
                      aria-pressed={item.id === selectedId}
                      onClick={() => { setSelectedId(item.id); }}
                    >
                      <span className="payroll__runlabel">{item.source_label}</span>
                      <span className="payroll__mono">{text.sublinePeriod(item.period_start, item.period_end)}</span>
                      <span className={CHIP[statusTone(item.status)]}>{statusLabel(item.status)}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </section>

            {detailError ? (
              <div className="payroll__alert payroll__alert--zone" role="alert">
                <span>{detailError}</span>
                <button type="button" onClick={() => { if (selectedId) void loadDetail(selectedId); }}>{text.retry}</button>
              </div>
            ) : !detail ? (
              <p className="payroll__zonestatus" role="status">{text.loadingRun}</p>
            ) : (
              <>
                <section className="payroll__card payroll__roster" aria-labelledby="payroll-roster-title">
                  <header className="payroll__cardhead">
                    <h2 id="payroll-roster-title">{text.rosterTitle}</h2>
                    {calced ? <span className="payroll__count payroll__mono">{shownLines.length} / {detail.lines_total}</span> : null}
                    <div className="payroll__spacer" />
                    {calced ? (
                      <input
                        className="payroll__search"
                        type="search"
                        value={query}
                        onChange={(event) => { setQuery(event.target.value); }}
                        placeholder={text.rosterSearch}
                        aria-label={text.rosterSearch}
                      />
                    ) : null}
                  </header>
                  {!calced ? (
                    <div className="payroll__gate">
                      <p role="status">
                        {run?.status === "CALCULATING" ? text.rosterGateCalculating
                          : run?.status === "ATTENDANCE_CLOSED" ? text.rosterGateClosed(detail.lines_total)
                          : text.rosterGateStaged}
                      </p>
                      {capabilities.canManage && run?.status === "STAGED" ? (
                        <button className="payroll__btn payroll__btn--ghost" type="button" disabled={busy} onClick={() => void openPreflight()}>{text.ctaClose}</button>
                      ) : null}
                      {capabilities.canManage && run?.status === "ATTENDANCE_CLOSED" ? (
                        <button className="payroll__btn payroll__btn--ghost" type="button" disabled={busy} onClick={() => void runAction((signal) => payrollApi.calculate(run.id, signal))}>{text.ctaCalc}</button>
                      ) : null}
                    </div>
                  ) : (
                    <>
                      <div className="payroll__grid" onKeyDown={rosterKeys}>
                        <div className="payroll__gridhead" style={{ gridTemplateColumns: gridCols }}>
                          <span>{text.colName}</span>
                          <span>
                            {text.colEntity}
                            <span className="payroll__colgrip" role="separator" aria-orientation="vertical" aria-label={text.colEntity} onPointerDown={(event) => { startColDrag("entity", event); }} />
                          </span>
                          <span className="payroll__num">
                            {text.colWorkDays}
                            <span className="payroll__colgrip" role="separator" aria-orientation="vertical" aria-label={text.colWorkDays} onPointerDown={(event) => { startColDrag("work", event); }} />
                          </span>
                          <span className="payroll__num">
                            {text.colOvertime}
                            <span className="payroll__colgrip" role="separator" aria-orientation="vertical" aria-label={text.colOvertime} onPointerDown={(event) => { startColDrag("overtime", event); }} />
                          </span>
                          <span>
                            {text.colSources}
                            <span className="payroll__colgrip" role="separator" aria-orientation="vertical" aria-label={text.colSources} onPointerDown={(event) => { startColDrag("sources", event); }} />
                          </span>
                          <span>
                            {text.colCalc}
                            <span className="payroll__colgrip" role="separator" aria-orientation="vertical" aria-label={text.colCalc} onPointerDown={(event) => { startColDrag("calc", event); }} />
                          </span>
                        </div>
                        {shownLines.length === 0 ? (
                          <p className="payroll__zonestatus" role="status">{text.rosterEmpty}</p>
                        ) : (
                        <div role="list" aria-label={text.rosterTitle}>
                        {shownLines.map((line, index) => {
                          const flag = line.employee_id != null ? openExceptionByEmployee.get(line.employee_id) : undefined;
                          const held = line.employee_id != null && heldEmployeeIds.has(line.employee_id);
                          const open = openLineId === line.id;
                          return (
                            <div key={line.id} role="listitem">
                              <div
                                ref={(node) => { rowRefs.current[index] = node; }}
                                className={open ? "payroll__row payroll__row--selected" : "payroll__row"}
                                role="button"
                                tabIndex={0}
                                aria-expanded={open}
                                style={{ gridTemplateColumns: gridCols }}
                                onClick={() => { setOpenLineId(open ? undefined : line.id); }}
                                onKeyDown={(event) => {
                                  if (event.key === "Enter" || event.key === " ") {
                                    event.preventDefault();
                                    setOpenLineId(open ? undefined : line.id);
                                  }
                                }}
                              >
                                <span className="payroll__who">
                                  <span className="payroll__avatar" aria-hidden="true">{line.employee_display_name.slice(0, 1)}</span>
                                  <span>{line.employee_display_name}</span>
                                </span>
                                <span><span className={CHIP.neutral}>{line.employee_company}</span></span>
                                <span className="payroll__num payroll__mono">{count(line.work_days)}</span>
                                <span className="payroll__num payroll__mono">{count(line.overtime_hours)}</span>
                                <span className="payroll__chips">
                                  <span className={line.gross_pay_source_present ? CHIP.ok : CHIP.danger}>{text.srcGross} {line.gross_pay_source_present ? text.srcPresent : text.srcMissing}</span>
                                  <span className={CHIP[NTS_TONE[line.nts_tax_row_status] ?? "neutral"]}>{ntsLabel(line.nts_tax_row_status)}</span>
                                </span>
                                <span className="payroll__chips">
                                  <span className={CHIP[LINE_CALC_TONE[line.calculation_status] ?? "neutral"]}>{lineCalcLabel(line.calculation_status)}</span>
                                  {line.blockers.length > 0 ? <span className={CHIP.danger}>{text.blockers(line.blockers.length)}</span> : null}
                                  {flag ? (
                                    <button
                                      className={CHIP[SEVERITY_TONE[flag.severity]]}
                                      type="button"
                                      onClick={(event) => {
                                        event.stopPropagation();
                                        setOpenExId(flag.id);
                                      }}
                                    >
                                      {exKindLabel(flag.kind)}
                                    </button>
                                  ) : null}
                                  {held ? <span className={CHIP.warn}>{text.exResolvedHold}</span> : null}
                                </span>
                              </div>
                              {open ? (
                                <div className="payroll__linedetail" aria-label={text.lineDetail}>
                                  <dl>
                                    <dt>{text.hoursWork}</dt><dd className="payroll__mono">{count(line.work_days)}</dd>
                                    <dt>{text.hoursRegular}</dt><dd className="payroll__mono">{count(line.regular_hours)}</dd>
                                    <dt>{text.hoursOvertime}</dt><dd className="payroll__mono">{count(line.overtime_hours)}</dd>
                                    <dt>{text.hoursNight}</dt><dd className="payroll__mono">{count(line.night_hours)}</dd>
                                    <dt>{text.hoursHoliday}</dt><dd className="payroll__mono">{count(line.holiday_hours)}</dd>
                                    <dt>{text.leaveUsed}</dt><dd className="payroll__mono">{count(line.leave_used)}</dd>
                                    <dt>{text.leaveRemaining}</dt><dd className="payroll__mono">{count(line.leave_remaining)}</dd>
                                  </dl>
                                  <button className={CHIP.purple} type="button" onClick={() => { void navigate("/people"); }}>{text.personCard}</button>
                                </div>
                              ) : null}
                            </div>
                          );
                        })}
                        </div>
                        )}
                      </div>
                      <footer className="payroll__cardfoot">{text.rosterAudit}</footer>
                    </>
                  )}
                </section>

                <div className="payroll__side">
                  <section className="payroll__card payroll__ex" aria-labelledby="payroll-ex-title">
                    <header className="payroll__cardhead">
                      <span
                        className={!calced ? "payroll__dot payroll__dot--wait" : exceptionsOpen > 0 ? "payroll__dot payroll__dot--warn" : "payroll__dot payroll__dot--ok"}
                        aria-hidden="true"
                      />
                      <h2 id="payroll-ex-title">{text.exTitle}</h2>
                      <span className="payroll__count payroll__mono">
                        {!calced ? text.exMetaWaiting
                          : exceptionsOpen > 0 ? text.exMetaLeft(exceptionsOpen, exceptionsTotal)
                          : text.exMetaDone(exceptionsTotal)}
                      </span>
                      <div className="payroll__spacer" />
                      {calced && exceptionsOpen > 0 ? <span className="payroll__hint">{text.exHint}</span> : null}
                    </header>
                    {!calced ? (
                      <p className="payroll__zonestatus" role="status">{text.exMetaWaiting}</p>
                    ) : sortedExceptions.length === 0 ? (
                      <p className="payroll__zonestatus" role="status">{text.exEmpty}</p>
                    ) : (
                      <ul className="payroll__exlist">
                        {sortedExceptions.map((exception) => {
                          const resolved = exception.status !== "OPEN";
                          const expanded = openExId === exception.id;
                          return (
                            <li key={exception.id} className={resolved ? "payroll__exrow payroll__exrow--resolved" : "payroll__exrow"}>
                              <div
                                className="payroll__exmain"
                                role="button"
                                tabIndex={0}
                                aria-expanded={expanded}
                                onClick={() => { setOpenExId(expanded ? undefined : exception.id); }}
                                onKeyDown={(event) => {
                                  if (event.key === "Enter" || event.key === " ") {
                                    event.preventDefault();
                                    setOpenExId(expanded ? undefined : exception.id);
                                  }
                                }}
                              >
                                <span className={CHIP[SEVERITY_TONE[exception.severity]]}>{exKindLabel(exception.kind)}</span>
                                <span className="payroll__exwho">
                                  {exception.employee_id != null ? (
                                    <button
                                      className="payroll__namelink"
                                      type="button"
                                      onClick={(event) => { event.stopPropagation(); void navigate("/people"); }}
                                    >
                                      {exception.employee_display_name}
                                    </button>
                                  ) : (
                                    <span className="payroll__namelink payroll__namelink--static">{exception.employee_display_name}</span>
                                  )}
                                  <span className="payroll__exline">{exception.summary_ko}</span>
                                </span>
                                {exception.amount_delta_won != null ? (
                                  <span className="payroll__mono payroll__examt">{maskOn ? text.masked : deltaWon(exception.amount_delta_won)}</span>
                                ) : null}
                                {exception.carried_from_run_id ? <span className={CHIP.info}>{text.exCarried}</span> : null}
                                {exception.status === "CONFIRMED" ? <span className={CHIP.ok}>{text.exResolvedOk}</span> : null}
                                {exception.status === "HELD" ? <span className={CHIP.warn}>{text.exResolvedHold}</span> : null}
                                {exception.status === "OPEN" && capabilities.canManage ? (
                                  <button
                                    className="payroll__btn payroll__btn--quiet"
                                    type="button"
                                    disabled={busy}
                                    onClick={(event) => { event.stopPropagation(); void resolveException(exception, "CONFIRM"); }}
                                  >
                                    {text.exConfirm}
                                  </button>
                                ) : null}
                              </div>
                              {expanded ? (
                                <div className="payroll__exdetail">
                                  {detailLines(exception.detail).map((line) => <p key={line}>{line}</p>)}
                                  {exception.linked_refs.length > 0 ? (
                                    <p className="payroll__msrow">
                                      {exception.linked_refs.map((ref) => (
                                        <button
                                          key={`${ref.kind}:${ref.code}`}
                                          className={CHIP.purple}
                                          type="button"
                                          onClick={() => { void navigate(refRoute(ref.kind)); }}
                                        >
                                          <span className="payroll__refkind">{refKindLabel(ref.kind)}</span>
                                          <span className="payroll__mono">{ref.code}</span>
                                        </button>
                                      ))}
                                    </p>
                                  ) : null}
                                  {exception.status === "OPEN" && capabilities.canManage ? (
                                    <div className="payroll__msacts">
                                      <button className="payroll__btn payroll__btn--primary" type="button" disabled={busy} onClick={() => void resolveException(exception, "CONFIRM")}>{text.exConfirm}</button>
                                      {holdExId === exception.id ? (
                                        <>
                                          <input
                                            aria-label={text.exHoldReason}
                                            placeholder={text.exHoldReason}
                                            value={holdReason}
                                            onChange={(event) => { setHoldReason(event.target.value); }}
                                          />
                                          <button className="payroll__btn payroll__btn--ghost" type="button" disabled={busy || !holdReason.trim()} onClick={() => void resolveException(exception, "HOLD", holdReason)}>{text.exHold}</button>
                                        </>
                                      ) : (
                                        <button className="payroll__btn payroll__btn--ghost" type="button" onClick={() => { setHoldExId(exception.id); }}>{text.exHold}</button>
                                      )}
                                    </div>
                                  ) : null}
                                </div>
                              ) : null}
                            </li>
                          );
                        })}
                      </ul>
                    )}
                  </section>

                  <section className="payroll__card payroll__totals" aria-labelledby="payroll-totals-title">
                    <header className="payroll__cardhead">
                      <h2 id="payroll-totals-title">{text.totalsTitle}</h2>
                      <span className={CHIP.neutral}>{totalsTag}</span>
                      <div className="payroll__spacer" />
                      <button className="payroll__btn payroll__btn--quiet" type="button" aria-pressed={!maskOn} onClick={() => { setMaskOn((current) => !current); }}>
                        {maskOn ? text.maskShow : text.maskHide}
                      </button>
                    </header>
                    {calc == null ? (
                      <p className="payroll__zonestatus" role="status">{text.totalsTagPre}</p>
                    ) : (
                      <div className="payroll__totalsbody">
                        {calc.total_net_won != null ? (
                          <p className="payroll__total payroll__mono">{maskOn ? text.masked : won(calc.total_net_won)}</p>
                        ) : null}
                        <p className="payroll__msrow">
                          <span className={CHIP.neutral}>{text.totalsCalcLines(calc.calculated_lines, calc.blocked_lines)}</span>
                          <span className={calc.payable ? CHIP.ok : CHIP.warn}>{calc.payable ? text.totalsPayable : text.totalsNotPayable}</span>
                        </p>
                        <p className="payroll__msrow">
                          <span className={accountOpen > 0 ? CHIP.danger : accountHeld > 0 ? CHIP.warn : CHIP.ok}>
                            {accountOpen > 0 ? text.accountStatusOpen(accountOpen)
                              : accountHeld > 0 ? text.accountStatusHeld(accountHeld)
                              : text.accountStatusOk}
                          </span>
                          <span className="payroll__chip payroll__chip--neutral payroll__mono">{calc.kernel_rate_table}</span>
                        </p>
                        <p className="payroll__msrow">
                          <button className={CHIP.purple} type="button" onClick={() => { void navigate("/laborcost"); }}>{text.laborcostLink}</button>
                        </p>
                      </div>
                    )}
                  </section>

                  <section className="payroll__card payroll__sched" aria-labelledby="payroll-sched-title">
                    <header className="payroll__cardhead">
                      <h2 id="payroll-sched-title">{text.scheduleTitle}</h2>
                    </header>
                    <ol className="payroll__mslist">
                      {milestones.map((milestone) => (
                        <li key={milestone.key}>
                          <div className="payroll__msline">
                            <span
                              className={milestone.state === "done" ? "payroll__dot payroll__dot--ok" : milestone.state === "now" ? "payroll__dot payroll__dot--now" : "payroll__dot payroll__dot--wait"}
                              aria-hidden="true"
                            />
                            <span className="payroll__mslabel">{milestone.label}</span>
                            {milestone.at ? <span className="payroll__mono payroll__msat">{timeText(milestone.at)}</span> : null}
                            {milestoneChip[milestone.state]}
                          </div>
                          {milestone.body}
                        </li>
                      ))}
                    </ol>
                    <footer className="payroll__cardfoot">{text.scheduleFooter}</footer>
                  </section>
                </div>
              </>
            )}
          </div>
        </>
      )}

      {preflight.open ? (
        <div className="payroll__scrim" onClick={closePreflightDialog}>
          <div
            ref={dialogRef}
            className="payroll__dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="payroll-preflight-title"
            tabIndex={-1}
            onClick={(event) => { event.stopPropagation(); }}
            onKeyDown={trapDialogFocus}
          >
            <h2 id="payroll-preflight-title">{text.preflightTitle}</h2>
            {preflight.error ? (
              <div className="payroll__alert" role="alert">
                <span>{preflight.error}</span>
                <button type="button" onClick={() => void openPreflight()}>{text.retry}</button>
              </div>
            ) : null}
            {actionError ? <div className="payroll__alert" role="alert"><span>{actionError}</span></div> : null}
            {preflight.loading ? (
              <p role="status">{text.preflightLoading}</p>
            ) : preflight.data ? (
              <ul className="payroll__checks">
                {preflight.data.checks.map((check) => (
                  <li key={check.key}>
                    <span className={check.ok ? (check.warn ? CHIP.warn : CHIP.ok) : CHIP.danger}>
                      {check.ok ? (check.warn ? text.checkWarn : text.checkOk) : text.checkBlocked}
                    </span>
                    <span className="payroll__checklabel">
                      <span>{check.label_ko}</span>
                      {check.note ? <span className="payroll__msnote">{check.note}</span> : null}
                    </span>
                    {check.blocking_refs.map((ref) => (
                      <button key={ref} className={CHIP.danger} type="button" onClick={() => { void navigate("/attendance"); }}>
                        <span className="payroll__mono">{ref}</span>
                      </button>
                    ))}
                  </li>
                ))}
              </ul>
            ) : null}
            <label className="payroll__attest">
              <input
                type="checkbox"
                checked={preflight.attested}
                onChange={(event) => { setPreflight((current) => ({ ...current, attested: event.target.checked })); }}
              />
              {text.preflightAttest}
            </label>
            <div className="payroll__msacts">
              <button className="payroll__btn payroll__btn--ghost" type="button" onClick={closePreflightDialog}>{text.preflightCancel}</button>
              <button
                className="payroll__btn payroll__btn--primary"
                type="button"
                disabled={busy || preflight.loading || preflight.data?.can_close !== true || !preflight.attested}
                onClick={() => void confirmClose()}
              >
                {text.preflightConfirm}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </main>
  );
}
