import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type DragEvent,
  type KeyboardEvent,
  type SyntheticEvent,
} from "react";

import type { ConsoleApiClient } from "../../api/client";
import { maintenanceStrings as text } from "../../i18n/maintenance";
import {
  createMaintenanceApi,
  MaintenanceApiError,
  type MaintenanceCause,
  type MaintenanceType,
  type PriorityLevel,
  type SettlementLine,
  type SettlementLineKind,
  type WorkOrderDetail,
  type WorkOrderLens,
  type WorkOrderListQuery,
  type WorkOrderRow,
  type WorkOrderSettlement,
  type WorkOrderStatus,
  type WorkResultType,
} from "./maintenanceApi";
import type { MaintenanceCapabilities } from "./maintenanceCapabilities";
import "./maintenance.css";

type Props = {
  api: ConsoleApiClient;
  branchId: string;
  actorId: string | undefined;
  capabilities: MaintenanceCapabilities;
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

const CLOSED_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "FINAL_COMPLETED",
  "REJECTED",
  "CANCELLED",
  "ARCHIVED",
]);
const TERMINAL_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "REJECTED",
  "CANCELLED",
  "ARCHIVED",
]);
const OPEN_STATUSES: readonly WorkOrderStatus[] = [
  "RECEIVED",
  "UNASSIGNED",
  "ASSIGNED",
  "IN_PROGRESS",
  "ON_HOLD",
  "DELAYED",
  "TEMPORARY_ACTION",
  "PART_WAITING",
  "EQUIPMENT_IN_USE",
  "REVISIT_REQUIRED",
  "REPORT_SUBMITTED",
  "ADMIN_REVIEW",
];
const ACTIVE_ASSIGNED_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "ASSIGNED",
  "IN_PROGRESS",
  "ON_HOLD",
  "DELAYED",
  "TEMPORARY_ACTION",
  "PART_WAITING",
  "EQUIPMENT_IN_USE",
  "REVISIT_REQUIRED",
]);
const PRE_EXECUTION_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "RECEIVED",
  "UNASSIGNED",
  "ASSIGNED",
  "ON_HOLD",
  "PART_WAITING",
  "DELAYED",
]);
const EXECUTION_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "IN_PROGRESS",
  "EQUIPMENT_IN_USE",
  "TEMPORARY_ACTION",
  "REVISIT_REQUIRED",
]);
const REVIEW_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "REPORT_SUBMITTED",
  "ADMIN_REVIEW",
]);
const SETTLEMENT_STATUSES: ReadonlySet<WorkOrderStatus> = new Set([
  "REPORT_SUBMITTED",
  "ADMIN_REVIEW",
  "FINAL_COMPLETED",
]);

/** §15 lifecycle projection: backend FSM statuses → the design's five flow steps. */
const FLOW_STEP_STATUSES: readonly (readonly WorkOrderStatus[])[] = [
  ["RECEIVED", "UNASSIGNED"],
  ["ASSIGNED", "ON_HOLD", "PART_WAITING", "DELAYED"],
  ["IN_PROGRESS", "EQUIPMENT_IN_USE", "TEMPORARY_ACTION", "REVISIT_REQUIRED"],
  ["REPORT_SUBMITTED", "ADMIN_REVIEW"],
  ["FINAL_COMPLETED"],
];
const FLOW_STEP_LABELS = [text.flow.intake, text.flow.plan, text.flow.execute, text.flow.settle, text.flow.voucher] as const;

const STATUS_TONE: Partial<Record<WorkOrderStatus, string>> = {
  RECEIVED: "info",
  UNASSIGNED: "warn",
  ASSIGNED: "info",
  IN_PROGRESS: "info",
  EQUIPMENT_IN_USE: "info",
  TEMPORARY_ACTION: "warn",
  PART_WAITING: "warn",
  ON_HOLD: "warn",
  DELAYED: "danger",
  REVISIT_REQUIRED: "warn",
  REPORT_SUBMITTED: "info",
  ADMIN_REVIEW: "info",
  FINAL_COMPLETED: "ok",
  REJECTED: "danger",
};
const PRIORITY_TONE: Partial<Record<PriorityLevel, string>> = {
  P1: "danger",
  P2: "warn",
  OUTSOURCE: "info",
};
// RETURNED is a review DECISION, not a settlement status — returning a
// settlement puts it back in DRAFT.
const SETTLEMENT_TONE: Record<WorkOrderSettlement["status"], string> = {
  DRAFT: "neutral",
  SUBMITTED: "info",
  APPROVED: "ok",
  VOID: "danger",
};
const WORM_TONE = { PENDING: "warn", VERIFIED: "ok", FAILED: "danger" } as const;

function chipClass(tone: string | undefined): string {
  if (tone === "ok") return "maintenance__chip maintenance__chip--ok";
  if (tone === "warn") return "maintenance__chip maintenance__chip--warn";
  if (tone === "danger") return "maintenance__chip maintenance__chip--danger";
  if (tone === "info") return "maintenance__chip maintenance__chip--info";
  return "maintenance__chip";
}

function statusLabel(status: string): string {
  if (status in text.status) return text.status[status as keyof typeof text.status];
  return text.status.unknown;
}

function priorityLabel(priority: PriorityLevel): string {
  return text.priority[priority];
}

function typeLabel(value: MaintenanceType): string {
  return text.maintenanceType[value];
}

function causeLabel(value: MaintenanceCause): string {
  return text.maintenanceCause[value];
}

function equipmentLabel(row: WorkOrderRow | WorkOrderDetail): string {
  return row.equipment.model ?? row.equipment.equipment_no;
}

const dateTime = new Intl.DateTimeFormat("ko-KR", { dateStyle: "short", timeStyle: "short" });
const krw = new Intl.NumberFormat("ko-KR");

function fmtWhen(iso: string | null | undefined): string | undefined {
  if (!iso) return undefined;
  const at = new Date(iso);
  return Number.isNaN(at.getTime()) ? undefined : dateTime.format(at);
}

function fmtRate(rate: number): string {
  const percent = rate <= 1 ? rate * 100 : rate;
  return `${String(Math.round(percent))}%`;
}

function isOverdue(row: { target_due_at: string | null; status: WorkOrderStatus }): boolean {
  return Boolean(
    row.target_due_at &&
    new Date(row.target_due_at).getTime() < Date.now() &&
    !CLOSED_STATUSES.has(row.status),
  );
}

function message(cause: unknown, fallback: string): string {
  return cause instanceof Error ? cause.message : fallback;
}

function formText(data: FormData, name: string): string {
  const value = data.get(name);
  return typeof value === "string" ? value.trim() : "";
}

function readStore(key: string): string | undefined {
  try {
    return window.sessionStorage.getItem(key) ?? undefined;
  } catch {
    return undefined;
  }
}

function writeStore(key: string, value: string | undefined): void {
  try {
    if (value === undefined) window.sessionStorage.removeItem(key);
    else window.sessionStorage.setItem(key, value);
  } catch {
    // Storage unavailable: selection/draft simply will not survive refresh.
  }
}

type SortKey = "order" | "work" | "site" | "assignee";

const SORTERS: Record<SortKey, (a: WorkOrderRow, b: WorkOrderRow) => number> = {
  order: (a, b) => a.request_no.localeCompare(b.request_no),
  work: (a, b) => equipmentLabel(a).localeCompare(equipmentLabel(b)),
  site: (a, b) => a.site.name.localeCompare(b.site.name),
  assignee: (a, b) =>
    (a.assignments[0]?.mechanic_name ?? "").localeCompare(b.assignments[0]?.mechanic_name ?? ""),
};

type DetailState =
  | { state: "idle" | "loading" }
  | { state: "denied" | "missing" }
  | { state: "error"; error: string }
  | { state: "ready"; value: WorkOrderDetail; settlement: WorkOrderSettlement | "denied" | undefined };

/**
 * Re-mount synchronously whenever effective authority changes. Effects run too
 * late to fence an old tenant/session's selection, error, or busy state.
 */
export function MaintenanceScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.branchId,
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <MaintenanceScreenBody key={sessionFence} {...props} />;
}

function MaintenanceScreenBody({ api, branchId, actorId, capabilities, sessionKey }: Props) {
  const selectionKey = `maintenance:${branchId}:selected`;
  const draftKey = `maintenance:${branchId}:draft`;
  const [rows, setRows] = useState<WorkOrderRow[]>([]);
  const [lens, setLens] = useState<WorkOrderLens>();
  const [query, setQuery] = useState<WorkOrderListQuery>({});
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState<{ key: SortKey; dir: 1 | -1 }>();
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [actionError, setActionError] = useState<string>();
  const [selectedId, setSelectedIdState] = useState<string | undefined>(() => readStore(selectionKey));
  const [detail, setDetail] = useState<DetailState>({ state: "idle" });
  const [composerOpen, setComposerOpen] = useState<boolean>(() => readStore(draftKey) !== undefined);
  const listGeneration = useRef(0);
  const listOperation = useRef<AbortController>(undefined);
  const detailGeneration = useRef(0);
  const detailOperation = useRef<AbortController>(undefined);
  const mutation = useRef<AbortController>(undefined);
  const maintenanceApi = useMemo(() => createMaintenanceApi(api), [api]);

  const setSelectedId = useCallback((id: string | undefined) => {
    setSelectedIdState(id);
    writeStore(selectionKey, id);
  }, [selectionKey]);

  const load = useCallback(async () => {
    if (!capabilities.canRead) {
      setRows([]);
      setLens(undefined);
      setLoading(false);
      return;
    }
    listOperation.current?.abort();
    const controller = new AbortController();
    listOperation.current = controller;
    const token = ++listGeneration.current;
    setLoading(true);
    setError(undefined);
    try {
      const page = await maintenanceApi.list(query, controller.signal);
      if (listGeneration.current === token) {
        setRows(page.items.filter((row) => row.branch_id === branchId));
        setLens(page.lens);
      }
    } catch (cause) {
      if (listGeneration.current === token && !controller.signal.aborted) {
        setError(message(cause, text.loadError));
      }
    } finally {
      if (listGeneration.current === token) setLoading(false);
    }
  }, [branchId, capabilities.canRead, maintenanceApi, query]);

  const loadDetail = useCallback(async () => {
    if (!capabilities.canRead || !selectedId) {
      setDetail({ state: "idle" });
      return;
    }
    detailOperation.current?.abort();
    const controller = new AbortController();
    detailOperation.current = controller;
    const token = ++detailGeneration.current;
    setDetail({ state: "loading" });
    try {
      const value = await maintenanceApi.detail(selectedId, controller.signal);
      if (detailGeneration.current !== token) return;
      if (value.branch_id !== branchId) {
        setDetail({ state: "missing" });
        return;
      }
      let settlement: WorkOrderSettlement | "denied" | undefined;
      if (SETTLEMENT_STATUSES.has(value.status)) {
        try {
          settlement = await maintenanceApi.settlement(selectedId, controller.signal);
        } catch (cause) {
          // A settlement-only denial must not deny the authorized detail read;
          // deny-by-omission renders the detail without the settlement zone.
          if (!(cause instanceof MaintenanceApiError && cause.status === 403)) throw cause;
          settlement = "denied";
        }
        if (detailGeneration.current !== token) return;
      }
      setDetail({ state: "ready", value, settlement });
    } catch (cause) {
      if (detailGeneration.current !== token || controller.signal.aborted) return;
      if (cause instanceof MaintenanceApiError && cause.status === 403) {
        setDetail({ state: "denied" });
      } else if (cause instanceof MaintenanceApiError && cause.status === 404) {
        setDetail({ state: "missing" });
      } else {
        setDetail({ state: "error", error: message(cause, text.loadError) });
      }
    }
  }, [branchId, capabilities.canRead, maintenanceApi, selectedId]);

  useEffect(() => {
    listGeneration.current += 1;
    listOperation.current?.abort();
    const start = window.setTimeout(() => {
      void load();
    }, 0);
    return () => {
      window.clearTimeout(start);
      listOperation.current?.abort();
    };
  }, [load, sessionKey]);

  useEffect(() => {
    detailGeneration.current += 1;
    detailOperation.current?.abort();
    const start = window.setTimeout(() => {
      void loadDetail();
    }, 0);
    return () => {
      window.clearTimeout(start);
      detailOperation.current?.abort();
    };
  }, [loadDetail, sessionKey]);

  /** Run a mutation, then reconcile from the backend list + detail reads. */
  const act = useCallback(async (work: (signal: AbortSignal) => Promise<unknown>) => {
    mutation.current?.abort();
    const controller = new AbortController();
    mutation.current = controller;
    setBusy(true);
    setActionError(undefined);
    try {
      const outcome = await work(controller.signal);
      if (controller.signal.aborted) return undefined;
      await Promise.all([load(), loadDetail()]);
      return outcome;
    } catch (cause) {
      if (!controller.signal.aborted) setActionError(message(cause, text.actionError));
      return undefined;
    } finally {
      if (mutation.current === controller) setBusy(false);
    }
  }, [load, loadDetail]);

  const applyFilters = useCallback((filters: Record<string, string>) => {
    setQuery((current) => {
      const next: WorkOrderListQuery = { ...current };
      for (const [key, value] of Object.entries(filters)) {
        if (key === "status") next.status = [value as WorkOrderStatus];
        else if (key === "priority") next.priority = [value as PriorityLevel];
        else if (key === "maintenance_type") next.maintenance_type = [value as MaintenanceType];
        else if (key === "maintenance_cause") next.maintenance_cause = [value as MaintenanceCause];
        else if (key === "equipment_id") next.equipment_id = value;
        else if (key === "customer_id") next.customer_id = value;
        else if (key === "site_id") next.site_id = value;
        else if (key === "assigned_to") next.assigned_to = value;
        else if (key === "around_work_order_id") next.around_work_order_id = value;
        else if (key === "target_due_to") next.target_due_to = value;
      }
      return next;
    });
  }, []);

  const filtersActive = Object.keys(query).length > 0;

  const onListKeyDown = (event: KeyboardEvent<HTMLUListElement>) => {
    if (event.key !== "j" && event.key !== "k") return;
    const buttons = Array.from(
      event.currentTarget.querySelectorAll<HTMLButtonElement>("button[data-row]"),
    );
    if (!buttons.length) return;
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const next = event.key === "j" ? Math.min(at + 1, buttons.length - 1) : Math.max(at - 1, 0);
    buttons[next]?.focus();
    event.preventDefault();
  };

  /** Header search narrows the already-authorized page client-side; the list API has no free-text param. */
  const visible = useMemo(() => {
    const needle = search.trim().toLowerCase();
    if (!needle) return rows;
    return rows.filter((row) =>
      row.request_no.toLowerCase().includes(needle) ||
      equipmentLabel(row).toLowerCase().includes(needle) ||
      row.site.name.toLowerCase().includes(needle) ||
      (row.assignments[0]?.mechanic_name ?? "").toLowerCase().includes(needle));
  }, [rows, search]);

  const sorted = useMemo(() => {
    if (!sort) return visible;
    return [...visible].sort((a, b) => SORTERS[sort.key](a, b) * sort.dir);
  }, [visible, sort]);

  const unassignedOpen = visible.filter(
    (row) => row.assignments.length === 0 && !CLOSED_STATUSES.has(row.status),
  );
  const laneDueSoon = unassignedOpen.filter((row) => row.priority === "P1" || isOverdue(row));
  const lanePlanned = unassignedOpen.filter((row) => !laneDueSoon.includes(row));
  const laneActive = visible.filter(
    (row) => row.assignments.length > 0 && ACTIVE_ASSIGNED_STATUSES.has(row.status),
  );

  const create = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!capabilities.canCreate) return;
    const form = event.currentTarget;
    const data = new FormData(form);
    const targetDue = formText(data, "target_due");
    const maintenanceType = formText(data, "maintenance_type");
    const maintenanceCause = formText(data, "maintenance_cause");
    const created = await act((signal) => maintenanceApi.create({
      branch_id: branchId,
      management_no: formText(data, "management_no"),
      symptom: formText(data, "symptom"),
      ...(formText(data, "customer_request") ? { customer_request: formText(data, "customer_request") } : {}),
      ...(targetDue ? { target_due_at: new Date(targetDue).toISOString() } : {}),
      ...(maintenanceType ? { maintenance_type: maintenanceType as MaintenanceType } : {}),
      ...(maintenanceCause ? { maintenance_cause: maintenanceCause as MaintenanceCause } : {}),
    }, signal));
    if (created && typeof created === "object" && "id" in created && typeof created.id === "string") {
      writeStore(draftKey, undefined);
      setComposerOpen(false);
      setSelectedId(created.id);
    }
  };

  const persistDraft = (event: SyntheticEvent<HTMLFormElement>) => {
    const entries = [...new FormData(event.currentTarget).entries()]
      .filter((entry): entry is [string, string] => typeof entry[1] === "string");
    writeStore(draftKey, JSON.stringify(Object.fromEntries(entries)));
  };

  /** Read at render time so a draft cleared after create cannot resurrect. */
  const readDraft = (): Record<string, string> | undefined => {
    const raw = readStore(draftKey);
    if (!raw) return undefined;
    try {
      return JSON.parse(raw) as Record<string, string>;
    } catch {
      return undefined;
    }
  };

  if (!capabilities.canRead) {
    return (
      <main className="maintenance">
        <section className="maintenance__panel" aria-labelledby="maintenance-title">
          <h1 id="maintenance-title">{text.title}</h1>
          <p role="status">{text.denied}</p>
        </section>
      </main>
    );
  }

  return (
    <main className="maintenance" aria-busy={loading || busy}>
      <header className="maintenance__header">
        <h1>{text.title}</h1>
        {lens && (
          <div className="maintenance__statbar" role="group" aria-label={text.stats.bar}>
            <button type="button" className="maintenance__stat" onClick={() => { setQuery({}); }}>
              <span>{text.stats.total}</span>
              <strong>{String(lens.aggregates.total_count)}</strong>
            </button>
            <button
              type="button"
              className="maintenance__stat maintenance__stat--danger"
              onClick={() => { applyFilters({ priority: "P1" }); }}
            >
              <span>{text.stats.p1}</span>
              <strong>{String(lens.aggregates.p1_count)}</strong>
            </button>
            <button
              type="button"
              className="maintenance__stat maintenance__stat--warn"
              onClick={() => {
                setQuery({ status: [...OPEN_STATUSES], target_due_to: new Date().toISOString() });
              }}
            >
              <span>{text.stats.overdue}</span>
              <strong>{String(lens.aggregates.overdue_open_count)}</strong>
            </button>
            <button
              type="button"
              className="maintenance__stat"
              onClick={() => { applyFilters({ status: "UNASSIGNED" }); }}
            >
              <span>{text.stats.unassigned}</span>
              <strong>{String(lens.aggregates.unassigned_count)}</strong>
            </button>
            {typeof lens.aggregates.preventive_on_time_rate === "number" && (
              <button
                type="button"
                className="maintenance__stat"
                onClick={() => { applyFilters({ maintenance_type: "PREVENTIVE" }); }}
              >
                <span>{text.stats.preventiveOnTime}</span>
                <strong>{fmtRate(lens.aggregates.preventive_on_time_rate)}</strong>
              </button>
            )}
            {typeof lens.aggregates.mttr_minutes === "number" && (
              <span className="maintenance__stat maintenance__stat--plain">
                <span>{text.stats.mttr}</span>
                <strong>{`${String(Math.round(lens.aggregates.mttr_minutes))}${text.stats.minutesUnit}`}</strong>
              </span>
            )}
          </div>
        )}
        <span className="maintenance__spacer" />
        <input
          type="search"
          className="maintenance__search"
          aria-label={text.search}
          value={search}
          onChange={(event) => { setSearch(event.currentTarget.value); }}
        />
        {capabilities.canCreate && (
          <button
            type="button"
            className="maintenance__action"
            aria-expanded={composerOpen}
            onClick={() => { setComposerOpen((open) => !open); }}
          >
            {text.create}
          </button>
        )}
      </header>
      {error && (
        <div className="maintenance__alert" role="alert">
          <span>{error}</span>
          <button type="button" className="maintenance__action maintenance__action--quiet" onClick={() => { void load(); }}>
            {text.retry}
          </button>
        </div>
      )}
      {composerOpen && capabilities.canCreate && (
        <ComposerForm draft={readDraft()} busy={busy} onSubmit={create} onInput={persistDraft} onClose={() => { setComposerOpen(false); }} />
      )}
      {lens && (
        <div className="maintenance__filters" aria-label={text.facets.heading}>
          <span>{text.facets.status}</span>
          {lens.facets.status.map((bucket) => (
            <button
              key={bucket.value}
              type="button"
              className={chipClass(STATUS_TONE[bucket.value as WorkOrderStatus])}
              aria-pressed={query.status?.length === 1 && query.status[0] === bucket.value}
              onClick={() => { applyFilters(bucket.filters); }}
            >
              {`${statusLabel(bucket.value)} ${String(bucket.count)}`}
            </button>
          ))}
          <span>{text.facets.priority}</span>
          {lens.facets.priority.map((bucket) => (
            <button
              key={bucket.value}
              type="button"
              className={chipClass(PRIORITY_TONE[bucket.value as PriorityLevel])}
              aria-pressed={query.priority?.length === 1 && query.priority[0] === bucket.value}
              onClick={() => { applyFilters(bucket.filters); }}
            >
              {`${priorityLabel(bucket.value as PriorityLevel)} ${String(bucket.count)}`}
            </button>
          ))}
          {filtersActive && (
            <button type="button" className="maintenance__chip" onClick={() => { setQuery({}); }}>
              {text.clearFilters}
            </button>
          )}
        </div>
      )}
      {!loading && (
        <section className="maintenance__lanes" aria-label={text.board}>
          <BoardLane tone="danger" label={text.lanes.dueSoonUnassigned} rows={laneDueSoon} selectedId={selectedId} onSelect={setSelectedId} />
          <BoardLane tone="warn" label={text.lanes.plannedUnassigned} rows={lanePlanned} selectedId={selectedId} onSelect={setSelectedId} />
          <BoardLane tone="ok" label={text.lanes.assignedActive} rows={laneActive} selectedId={selectedId} onSelect={setSelectedId} />
        </section>
      )}
      <div className="maintenance__body">
        <section className="maintenance__panel" aria-label={text.list}>
          <div className="maintenance__cols">
            {(["order", "work", "site", "assignee"] as const).map((key) => (
              <button
                key={key}
                type="button"
                aria-pressed={sort?.key === key}
                onClick={() => {
                  setSort((current) =>
                    current?.key === key
                      ? current.dir === 1 ? { key, dir: -1 } : undefined
                      : { key, dir: 1 });
                }}
              >
                {text.cols[key]}
              </button>
            ))}
            <span>{text.facets.status}</span>
          </div>
          {loading ? (
            <p role="status">{text.loading}</p>
          ) : (
            <ul className="maintenance__list" aria-label={text.list} onKeyDown={onListKeyDown}>
              {sorted.length ? sorted.map((row) => (
                <li key={row.id}>
                  <button
                    type="button"
                    data-row={row.id}
                    className={row.id === selectedId ? "maintenance__row maintenance__row--selected" : "maintenance__row"}
                    aria-pressed={row.id === selectedId}
                    draggable
                    onDragStart={(event: DragEvent<HTMLButtonElement>) => {
                      event.dataTransfer.setData("text/plain", `${row.request_no} ${equipmentLabel(row)}`);
                    }}
                    onClick={() => { setSelectedId(row.id); }}
                  >
                    <code>{row.request_no}</code>
                    <span>
                      {row.maintenance_type
                        ? `${typeLabel(row.maintenance_type)} — ${equipmentLabel(row)}`
                        : equipmentLabel(row)}
                    </span>
                    <span>{row.site.name}</span>
                    <span>{row.assignments[0]?.mechanic_name ?? text.kv.unassigned}</span>
                    <span className={chipClass(isOverdue(row) ? "danger" : STATUS_TONE[row.status])}>
                      {isOverdue(row) ? text.kv.overdue : statusLabel(row.status)}
                    </span>
                  </button>
                </li>
              )) : <li role="status">{text.empty}</li>}
            </ul>
          )}
        </section>
        <section className="maintenance__panel" aria-label={text.detail} aria-live="polite">
          {actionError && (
            <div className="maintenance__alert" role="alert">
              <span>{actionError}</span>
            </div>
          )}
          {detail.state === "idle" && <p>{text.select}</p>}
          {detail.state === "loading" && <p role="status">{text.detailLoading}</p>}
          {detail.state === "denied" && <p role="status">{text.detailDenied}</p>}
          {detail.state === "missing" && <p role="status">{text.detailNotFound}</p>}
          {detail.state === "error" && (
            <div className="maintenance__alert" role="alert">
              <span>{detail.error}</span>
              <button type="button" className="maintenance__action maintenance__action--quiet" onClick={() => { void loadDetail(); }}>
                {text.retry}
              </button>
            </div>
          )}
          {detail.state === "ready" && (
            <DetailPanel
              key={detail.value.id}
              detail={detail.value}
              settlement={detail.settlement}
              capabilities={capabilities}
              actorId={actorId}
              busy={busy}
              maintenanceApi={maintenanceApi}
              act={act}
              applyFilters={applyFilters}
            />
          )}
        </section>
      </div>
    </main>
  );
}

function BoardLane({ tone, label, rows, selectedId, onSelect }: {
  tone: "danger" | "warn" | "ok";
  label: string;
  rows: WorkOrderRow[];
  selectedId: string | undefined;
  onSelect: (id: string) => void;
}) {
  const laneClass = tone === "danger"
    ? "maintenance__lane maintenance__lane--danger"
    : tone === "warn"
      ? "maintenance__lane maintenance__lane--warn"
      : "maintenance__lane maintenance__lane--ok";
  return (
    <section className={laneClass} aria-label={label}>
      <h3>
        {label}
        <span className={chipClass(tone)}>{String(rows.length)}</span>
      </h3>
      <ul>
        {rows.length ? rows.map((row) => (
          <li key={row.id}>
            <button
              type="button"
              className="maintenance__card"
              aria-pressed={row.id === selectedId}
              onClick={() => { onSelect(row.id); }}
            >
              <span>
                <code>{row.request_no}</code>
                <span className={chipClass(isOverdue(row) ? "danger" : STATUS_TONE[row.status])}>
                  {isOverdue(row) ? text.kv.overdue : statusLabel(row.status)}
                </span>
              </span>
              <em>{`${equipmentLabel(row)} · ${row.site.name}`}</em>
            </button>
          </li>
        )) : <li role="status">{text.lanes.empty}</li>}
      </ul>
    </section>
  );
}

function ComposerForm({ draft, busy, onSubmit, onInput, onClose }: {
  draft: Record<string, string> | undefined;
  busy: boolean;
  onSubmit: (event: SyntheticEvent<HTMLFormElement>) => Promise<void>;
  onInput: (event: SyntheticEvent<HTMLFormElement>) => void;
  onClose: () => void;
}) {
  const id = useId();
  return (
    <form className="maintenance__form" aria-label={text.create} onSubmit={(event) => void onSubmit(event)} onInput={onInput}>
      <div className="maintenance__form-grid">
        <label htmlFor={`${id}-management-no`}>
          {text.form.managementNo}
          <input id={`${id}-management-no`} name="management_no" defaultValue={draft?.management_no ?? ""} required />
        </label>
        <label htmlFor={`${id}-target-due`}>
          {text.form.targetDue}
          <input id={`${id}-target-due`} name="target_due" type="datetime-local" defaultValue={draft?.target_due ?? ""} />
        </label>
        <label htmlFor={`${id}-type`}>
          {text.maintenanceType.label}
          <select id={`${id}-type`} name="maintenance_type" defaultValue={draft?.maintenance_type ?? ""}>
            <option value="">{text.form.none}</option>
            {(["EMERGENCY", "CORRECTIVE", "PREVENTIVE", "INSPECTION"] as const).map((value) => (
              <option key={value} value={value}>{typeLabel(value)}</option>
            ))}
          </select>
        </label>
        <label htmlFor={`${id}-cause`}>
          {text.maintenanceCause.label}
          <select id={`${id}-cause`} name="maintenance_cause" defaultValue={draft?.maintenance_cause ?? ""}>
            <option value="">{text.form.none}</option>
            {(["BREAKDOWN", "RETURN_PREP", "SCHEDULED", "INSPECTION_FINDING", "OTHER"] as const).map((value) => (
              <option key={value} value={value}>{causeLabel(value)}</option>
            ))}
          </select>
        </label>
      </div>
      <label htmlFor={`${id}-symptom`}>
        {text.form.symptom}
        <textarea id={`${id}-symptom`} name="symptom" maxLength={2000} defaultValue={draft?.symptom ?? ""} required />
      </label>
      <label htmlFor={`${id}-customer-request`}>
        {text.form.customerRequest}
        <textarea id={`${id}-customer-request`} name="customer_request" maxLength={2000} defaultValue={draft?.customer_request ?? ""} />
      </label>
      <div className="maintenance__acts">
        <div>
          <button type="submit" className="maintenance__action" disabled={busy}>{text.form.submit}</button>
          <button type="button" className="maintenance__action maintenance__action--quiet" onClick={onClose}>{text.form.close}</button>
        </div>
      </div>
    </form>
  );
}

type Act = (work: (signal: AbortSignal) => Promise<unknown>) => Promise<unknown>;

function DetailPanel({ detail, settlement, capabilities, actorId, busy, maintenanceApi, act, applyFilters }: {
  detail: WorkOrderDetail;
  settlement: WorkOrderSettlement | "denied" | undefined;
  capabilities: MaintenanceCapabilities;
  actorId: string | undefined;
  busy: boolean;
  maintenanceApi: ReturnType<typeof createMaintenanceApi>;
  act: Act;
  applyFilters: (filters: Record<string, string>) => void;
}) {
  const primaryAssignment = detail.assignments.at(0);
  const received = fmtWhen(detail.created_at);
  const targetDue = fmtWhen(detail.target_due_at);
  return (
    <article className="maintenance__detail">
      <header>
        <h2>{detail.request_no}</h2>
        <span className={chipClass(STATUS_TONE[detail.status])}>{statusLabel(detail.status)}</span>
        <span className={chipClass(PRIORITY_TONE[detail.priority])}>{priorityLabel(detail.priority)}</span>
        {detail.result_type !== "UNKNOWN" && (
          <span className="maintenance__chip">{text.resultType[detail.result_type]}</span>
        )}
        {detail.kpi_excluded && <span className="maintenance__chip">{text.kv.kpiExcluded}</span>}
        <span className={chipClass(detail.evidence_verified ? "ok" : "warn")}>
          {detail.evidence_verified ? text.evidence.verified : text.evidence.notVerified}
        </span>
      </header>
      <FlowStepper status={detail.status} settlement={settlement === "denied" ? undefined : settlement} />
      {(detail.maintenance_type ?? detail.maintenance_cause) && (
        <div className="maintenance__links">
          {detail.maintenance_type && (
            <button
              type="button"
              className="maintenance__chip maintenance__chip--info"
              onClick={() => { applyFilters({ maintenance_type: detail.maintenance_type ?? "" }); }}
            >
              {`${text.maintenanceType.label} · ${typeLabel(detail.maintenance_type)}`}
            </button>
          )}
          {detail.maintenance_cause && (
            <button
              type="button"
              className="maintenance__chip maintenance__chip--info"
              onClick={() => { applyFilters({ maintenance_cause: detail.maintenance_cause ?? "" }); }}
            >
              {`${text.maintenanceCause.label} · ${causeLabel(detail.maintenance_cause)}`}
            </button>
          )}
        </div>
      )}
      <dl className="maintenance__kv">
        {received && (
          <>
            <dt>{text.kv.receivedAt}</dt>
            <dd>{received}</dd>
          </>
        )}
        <dt>{text.kv.targetDue}</dt>
        <dd>
          {targetDue ?? text.lanes.empty}
          {isOverdue(detail) && <span className={chipClass("danger")}>{text.kv.overdue}</span>}
        </dd>
        <dt>{text.kv.equipment}</dt>
        <dd>
          <button
            type="button"
            className="maintenance__chip"
            onClick={() => { applyFilters({ equipment_id: detail.equipment.id }); }}
          >
            {`${detail.equipment.equipment_no} · ${detail.equipment.model ?? detail.equipment.specification}`}
          </button>
        </dd>
        <dt>{text.kv.customer}</dt>
        <dd>
          <button
            type="button"
            className="maintenance__chip"
            onClick={() => { applyFilters({ customer_id: detail.customer.id }); }}
          >
            {detail.customer.name}
          </button>
        </dd>
        <dt>{text.kv.site}</dt>
        <dd>
          <button
            type="button"
            className="maintenance__chip"
            onClick={() => { applyFilters({ site_id: detail.site.id }); }}
          >
            {detail.site.name}
          </button>
        </dd>
        <dt>{text.kv.assignee}</dt>
        <dd>
          {detail.assignments.length ? detail.assignments.map((assignment) => (
            <button
              key={assignment.id}
              type="button"
              className="maintenance__chip"
              onClick={() => { applyFilters({ assigned_to: assignment.mechanic_id }); }}
            >
              {`${assignment.mechanic_name} · ${text.assignRole[assignment.role]}`}
            </button>
          )) : text.kv.unassigned}
        </dd>
        {detail.site_contact?.name && (
          <>
            <dt>{text.kv.contact}</dt>
            <dd>{[detail.site_contact.name, detail.site_contact.phone].filter(Boolean).join(" · ")}</dd>
          </>
        )}
        <dt>{text.kv.symptom}</dt>
        <dd>{detail.symptom}</dd>
        {detail.customer_request && (
          <>
            <dt>{text.kv.customerRequest}</dt>
            <dd>{detail.customer_request}</dd>
          </>
        )}
        {detail.diagnosis && (
          <>
            <dt>{text.kv.diagnosis}</dt>
            <dd>{detail.diagnosis}</dd>
          </>
        )}
        {detail.action_taken && (
          <>
            <dt>{text.kv.actionTaken}</dt>
            <dd>{detail.action_taken}</dd>
          </>
        )}
        {detail.delay_reason && (
          <>
            <dt>{text.kv.delayReason}</dt>
            <dd>{[detail.delay_reason, detail.delay_note].filter(Boolean).join(" · ")}</dd>
          </>
        )}
      </dl>
      <div className="maintenance__links">
        <button
          type="button"
          className="maintenance__chip"
          onClick={() => { applyFilters({ equipment_id: detail.equipment.id }); }}
        >
          {text.links.assetHistory}
        </button>
        <button
          type="button"
          className="maintenance__chip"
          onClick={() => { applyFilters({ customer_id: detail.customer.id }); }}
        >
          {text.links.customerOrders}
        </button>
        <button
          type="button"
          className="maintenance__chip"
          onClick={() => { applyFilters({ site_id: detail.site.id }); }}
        >
          {text.links.siteOrders}
        </button>
        {primaryAssignment && (
          <button
            type="button"
            className="maintenance__chip"
            onClick={() => { applyFilters({ assigned_to: primaryAssignment.mechanic_id }); }}
          >
            {text.links.assigneeOrders}
          </button>
        )}
        <button
          type="button"
          className="maintenance__chip"
          onClick={() => { applyFilters({ around_work_order_id: detail.id }); }}
        >
          {text.links.related}
        </button>
      </div>
      <ActionsSection detail={detail} capabilities={capabilities} busy={busy} maintenanceApi={maintenanceApi} act={act} />
      {SETTLEMENT_STATUSES.has(detail.status) && settlement !== "denied" && (
        <SettlementSection
          detail={detail}
          settlement={settlement}
          capabilities={capabilities}
          actorId={actorId}
          busy={busy}
          maintenanceApi={maintenanceApi}
          act={act}
        />
      )}
      <EvidenceSection detail={detail} capabilities={capabilities} busy={busy} maintenanceApi={maintenanceApi} act={act} />
      <section className="maintenance__section" aria-label={text.approval.heading}>
        <h3>{text.approval.heading}</h3>
        {detail.approval_line.length ? (
          <ul className="maintenance__timeline">
            {detail.approval_line.map((step) => (
              <li key={step.id}>
                <span className="maintenance__chip">
                  {step.role in text.approval.role
                    ? text.approval.role[step.role as keyof typeof text.approval.role]
                    : step.role}
                </span>
                <span className={chipClass(step.status === "APPROVED" ? "ok" : step.status === "REJECTED" ? "danger" : undefined)}>
                  {step.status in text.approval.step
                    ? text.approval.step[step.status as keyof typeof text.approval.step]
                    : step.status}
                </span>
                <span>{step.approved_by_name ?? step.approver_name ?? text.lanes.empty}</span>
                {step.approved_at && <time dateTime={step.approved_at}>{fmtWhen(step.approved_at)}</time>}
                {step.decision_comment && <span>{step.decision_comment}</span>}
              </li>
            ))}
          </ul>
        ) : <p role="status">{text.approval.empty}</p>}
      </section>
      <section className="maintenance__section" aria-label={text.history.heading}>
        <h3>{text.history.heading}</h3>
        <ul className="maintenance__timeline">
          {detail.status_history.map((entry) => (
            <li key={entry.id}>
              <time dateTime={entry.occurred_at}>{fmtWhen(entry.occurred_at)}</time>
              {entry.from_status && (
                <span className={chipClass(STATUS_TONE[entry.from_status as WorkOrderStatus])}>
                  {statusLabel(entry.from_status)}
                </span>
              )}
              <span className={chipClass(STATUS_TONE[entry.to_status])}>{statusLabel(entry.to_status)}</span>
              <span>{entry.action}</span>
            </li>
          ))}
        </ul>
      </section>
    </article>
  );
}

function FlowStepper({ status, settlement }: {
  status: WorkOrderStatus;
  settlement: WorkOrderSettlement | undefined;
}) {
  if (TERMINAL_STATUSES.has(status)) {
    return <span className={chipClass(STATUS_TONE[status])}>{statusLabel(status)}</span>;
  }
  const at = FLOW_STEP_STATUSES.findIndex((statuses) => statuses.includes(status));
  const voucherDone = settlement?.status === "APPROVED";
  return (
    <ol className="maintenance__flow" aria-label={text.detail}>
      {FLOW_STEP_LABELS.map((label, index) => {
        const step = index === 4 && voucherDone
          ? "done"
          : index < at ? "done" : index === at ? "cur" : "next";
        return <li key={label} data-step={step}>{label}</li>;
      })}
    </ol>
  );
}

function ActionsSection({ detail, capabilities, busy, maintenanceApi, act }: {
  detail: WorkOrderDetail;
  capabilities: MaintenanceCapabilities;
  busy: boolean;
  maintenanceApi: ReturnType<typeof createMaintenanceApi>;
  act: Act;
}) {
  const id = useId();
  const [mechanicError, setMechanicError] = useState(false);
  const [reviewError, setReviewError] = useState<"approve" | "reject">();
  const mechanicRef = useRef<HTMLInputElement>(null);
  const commentRef = useRef<HTMLTextAreaElement>(null);
  const priorityRef = useRef<HTMLSelectElement>(null);
  const canAssignNow = capabilities.canAssign && PRE_EXECUTION_STATUSES.has(detail.status);
  const canStartNow = capabilities.canStart && detail.status === "ASSIGNED";
  const canReportNow = capabilities.canSubmitReport && EXECUTION_STATUSES.has(detail.status);
  const canReviewNow = capabilities.canReview && REVIEW_STATUSES.has(detail.status);
  if (!canAssignNow && !canStartNow && !canReportNow && !canReviewNow && !capabilities.canManagePriority) {
    return null;
  }

  const assign = async () => {
    const mechanicId = mechanicRef.current?.value.trim() ?? "";
    if (!mechanicId) {
      setMechanicError(true);
      return;
    }
    setMechanicError(false);
    await act((signal) => maintenanceApi.assign(detail.id, {
      assignments: [{ mechanic_id: mechanicId, role: "PRIMARY" }],
    }, signal));
  };

  const report = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    await act((signal) => maintenanceApi.report(detail.id, {
      result_type: formText(data, "result_type") as WorkResultType,
      diagnosis: formText(data, "diagnosis"),
      action_taken: formText(data, "action_taken"),
    }, signal));
  };

  const review = async (decision: "approve" | "reject") => {
    const comment = commentRef.current?.value.trim() ?? "";
    if (!comment) {
      setReviewError(decision);
      return;
    }
    setReviewError(undefined);
    await act((signal) => decision === "approve"
      ? maintenanceApi.approve(detail.id, comment, signal)
      : maintenanceApi.reject(detail.id, comment, signal));
  };

  return (
    <section className="maintenance__section maintenance__acts" aria-label={text.actions.assign}>
      {capabilities.canManagePriority && (
        <div>
          <label htmlFor={`${id}-priority`}>
            {text.actions.priority}
            <select id={`${id}-priority`} ref={priorityRef} defaultValue={detail.priority}>
              {(["P1", "P2", "P3", "OUTSOURCE", "UNSET"] as const).map((value) => (
                <option key={value} value={value}>{priorityLabel(value)}</option>
              ))}
            </select>
          </label>
          <button
            type="button"
            className="maintenance__action maintenance__action--quiet"
            disabled={busy}
            onClick={() => {
              const priority = priorityRef.current?.value as PriorityLevel | undefined;
              if (priority && priority !== detail.priority) {
                void act((signal) => maintenanceApi.setPriority(detail.id, priority, signal));
              }
            }}
          >
            {text.actions.priorityApply}
          </button>
        </div>
      )}
      {canAssignNow && (
        <div>
          <label htmlFor={`${id}-mechanic`}>
            {text.actions.mechanicId}
            <input id={`${id}-mechanic`} ref={mechanicRef} aria-invalid={mechanicError} />
          </label>
          <button type="button" className="maintenance__action" disabled={busy} onClick={() => void assign()}>
            {text.actions.assign}
          </button>
          {mechanicError && <span className="maintenance__field-error" role="alert">{text.actions.mechanicRequired}</span>}
        </div>
      )}
      {canStartNow && (
        <div>
          <button type="button" className="maintenance__action" disabled={busy} onClick={() => void act((signal) => maintenanceApi.start(detail.id, signal))}>
            {text.actions.start}
          </button>
        </div>
      )}
      {canReportNow && (
        <form className="maintenance__form" aria-label={text.actions.report} onSubmit={(event) => void report(event)}>
          <label htmlFor={`${id}-result`}>
            {text.actions.resultType}
            <select id={`${id}-result`} name="result_type" defaultValue="COMPLETED">
              {(["COMPLETED", "TEMPORARY_ACTION", "INCOMPLETE", "REVISIT_REQUIRED"] as const).map((value) => (
                <option key={value} value={value}>{text.resultType[value]}</option>
              ))}
            </select>
          </label>
          <label htmlFor={`${id}-diagnosis`}>
            {text.actions.diagnosis}
            <textarea id={`${id}-diagnosis`} name="diagnosis" maxLength={4000} required />
          </label>
          <label htmlFor={`${id}-action-taken`}>
            {text.actions.actionTaken}
            <textarea id={`${id}-action-taken`} name="action_taken" maxLength={4000} required />
          </label>
          <div>
            <button type="submit" className="maintenance__action" disabled={busy}>{text.actions.report}</button>
          </div>
        </form>
      )}
      {canReviewNow && (
        <div>
          <label htmlFor={`${id}-comment`}>
            {text.actions.reviewComment}
            <textarea id={`${id}-comment`} ref={commentRef} maxLength={2000} aria-invalid={reviewError !== undefined} />
          </label>
          <button type="button" className="maintenance__action" disabled={busy} onClick={() => void review("approve")}>
            {text.actions.approve}
          </button>
          <button type="button" className="maintenance__action maintenance__action--danger" disabled={busy} onClick={() => void review("reject")}>
            {text.actions.reject}
          </button>
          {reviewError && (
            <span className="maintenance__field-error" role="alert">
              {reviewError === "approve" ? text.actions.approveCommentRequired : text.actions.rejectMemoRequired}
            </span>
          )}
        </div>
      )}
    </section>
  );
}

function SettlementSection({ detail, settlement, capabilities, actorId, busy, maintenanceApi, act }: {
  detail: WorkOrderDetail;
  settlement: WorkOrderSettlement | undefined;
  capabilities: MaintenanceCapabilities;
  actorId: string | undefined;
  busy: boolean;
  maintenanceApi: ReturnType<typeof createMaintenanceApi>;
  act: Act;
}) {
  const id = useId();
  const [lines, setLines] = useState<{ kind: SettlementLineKind; label: string; amount: string; source_ref: string }[]>([
    { kind: "LABOR", label: "", amount: "", source_ref: "" },
  ]);
  const [lineError, setLineError] = useState<"empty" | "label">();
  const [commentError, setCommentError] = useState<"return" | "void">();
  const commentRef = useRef<HTMLTextAreaElement>(null);
  const total = settlement?.total_amount_krw ?? 0;

  const createSettlement = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    const filled = lines.filter((line) => line.amount.trim() !== "" && Number.isFinite(Number(line.amount)));
    if (!filled.length) {
      setLineError("empty");
      return;
    }
    // The server requires a non-empty label on every line (422 otherwise).
    if (filled.some((line) => line.label.trim() === "")) {
      setLineError("label");
      return;
    }
    const payload: SettlementLine[] = filled.map((line, index) => ({
      kind: line.kind,
      label: line.label.trim(),
      amount_krw: Number(line.amount),
      sort_order: index,
      ...(line.source_ref.trim() ? { source_ref: line.source_ref.trim() } : {}),
    }));
    setLineError(undefined);
    await act((signal) => maintenanceApi.createSettlement(detail.id, payload, signal));
  };

  const reviewSettlement = async (decision: "APPROVED" | "RETURNED") => {
    if (!settlement) return;
    const comment = commentRef.current?.value.trim() ?? "";
    if (decision === "RETURNED" && !comment) {
      setCommentError("return");
      return;
    }
    setCommentError(undefined);
    await act((signal) => maintenanceApi.reviewSettlement(
      settlement.id,
      { decision, ...(comment ? { comment } : {}) },
      signal,
    ));
  };

  const voidSettlement = async () => {
    if (!settlement) return;
    const reason = commentRef.current?.value.trim() ?? "";
    if (!reason) {
      setCommentError("void");
      return;
    }
    setCommentError(undefined);
    await act((signal) => maintenanceApi.voidSettlement(settlement.id, reason, signal));
  };

  return (
    <section className="maintenance__section" aria-label={text.settlement.heading}>
      <h3>{text.settlement.heading}</h3>
      {settlement ? (
        <>
          <div className="maintenance__links">
            <span className={chipClass(SETTLEMENT_TONE[settlement.status])}>
              {text.settlement.status[settlement.status]}
            </span>
            {settlement.voucher_ref && (
              <span className="maintenance__chip maintenance__chip--info">
                {`${text.settlement.voucherRef} · ${settlement.voucher_ref}`}
              </span>
            )}
          </div>
          <table className="maintenance__table">
            <thead>
              <tr>
                <th scope="col">{text.settlement.lineKind}</th>
                <th scope="col">{text.settlement.lineLabel}</th>
                <th scope="col" className="maintenance__amount">{text.settlement.amount}</th>
                <th scope="col">{text.settlement.sourceRef}</th>
              </tr>
            </thead>
            <tbody>
              {settlement.lines.map((line) => (
                <tr key={line.id}>
                  <td>{text.settlement.kind[line.kind]}</td>
                  <td>{line.label}</td>
                  <td className="maintenance__amount">{krw.format(line.amount_krw)}</td>
                  <td>{line.source_ref ?? text.lanes.empty}</td>
                </tr>
              ))}
              <tr>
                <td>{text.settlement.total}</td>
                <td />
                <td className="maintenance__amount">{krw.format(total)}</td>
                <td />
              </tr>
            </tbody>
          </table>
          {settlement.note && <p>{`${text.settlement.note} · ${settlement.note}`}</p>}
          <div className="maintenance__acts">
            {settlement.status === "DRAFT" && capabilities.canSettle && (
              <div>
                <button type="button" className="maintenance__action" disabled={busy} onClick={() => void act((signal) => maintenanceApi.submitSettlement(settlement.id, signal))}>
                  {text.settlement.submit}
                </button>
              </div>
            )}
            {settlement.status === "SUBMITTED" && capabilities.canReviewSettlement && (
              <div>
                <label htmlFor={`${id}-settle-comment`}>
                  {text.settlement.reviewComment}
                  <textarea id={`${id}-settle-comment`} ref={commentRef} maxLength={2000} aria-invalid={commentError !== undefined} />
                </label>
                <button type="button" className="maintenance__action" disabled={busy} onClick={() => void reviewSettlement("APPROVED")}>
                  {text.settlement.approve}
                </button>
                <button type="button" className="maintenance__action maintenance__action--danger" disabled={busy} onClick={() => void reviewSettlement("RETURNED")}>
                  {text.settlement.return}
                </button>
                <button type="button" className="maintenance__action maintenance__action--danger" disabled={busy} onClick={() => void voidSettlement()}>
                  {text.settlement.void}
                </button>
                {commentError && (
                  <span className="maintenance__field-error" role="alert">
                    {commentError === "return" ? text.settlement.returnCommentRequired : text.settlement.voidReasonRequired}
                  </span>
                )}
              </div>
            )}
          </div>
        </>
      ) : capabilities.canSettle && actorId ? (
        <form className="maintenance__form" aria-label={text.settlement.create} onSubmit={(event) => void createSettlement(event)}>
          {lines.map((line, index) => (
            <div key={String(index)} className="maintenance__form-grid">
              <label htmlFor={`${id}-kind-${String(index)}`}>
                {text.settlement.lineKind}
                <select
                  id={`${id}-kind-${String(index)}`}
                  value={line.kind}
                  onChange={(event) => {
                    const kind = event.currentTarget.value as SettlementLineKind;
                    setLines((current) => current.map((entry, i) => i === index ? { ...entry, kind } : entry));
                  }}
                >
                  {(["LABOR", "PART", "OUTSOURCE", "OTHER"] as const).map((value) => (
                    <option key={value} value={value}>{text.settlement.kind[value]}</option>
                  ))}
                </select>
              </label>
              <label htmlFor={`${id}-amount-${String(index)}`}>
                {text.settlement.amount}
                <input
                  id={`${id}-amount-${String(index)}`}
                  type="number"
                  min={0}
                  step={1}
                  value={line.amount}
                  onChange={(event) => {
                    const amount = event.currentTarget.value;
                    setLines((current) => current.map((entry, i) => i === index ? { ...entry, amount } : entry));
                  }}
                />
              </label>
              <label htmlFor={`${id}-source-${String(index)}`}>
                {text.settlement.sourceRef}
                <input
                  id={`${id}-source-${String(index)}`}
                  value={line.source_ref}
                  onChange={(event) => {
                    const ref = event.currentTarget.value;
                    setLines((current) => current.map((entry, i) => i === index ? { ...entry, source_ref: ref } : entry));
                  }}
                />
              </label>
              <label htmlFor={`${id}-label-${String(index)}`}>
                {text.settlement.lineLabel}
                <input
                  id={`${id}-label-${String(index)}`}
                  value={line.label}
                  onChange={(event) => {
                    const label = event.currentTarget.value;
                    setLines((current) => current.map((entry, i) => i === index ? { ...entry, label } : entry));
                  }}
                />
              </label>
            </div>
          ))}
          <div className="maintenance__acts">
            <div>
              <button
                type="button"
                className="maintenance__action maintenance__action--quiet"
                onClick={() => { setLines((current) => [...current, { kind: "PART", label: "", amount: "", source_ref: "" }]); }}
              >
                {text.settlement.addLine}
              </button>
              <button type="submit" className="maintenance__action" disabled={busy}>{text.settlement.create}</button>
            </div>
            {lineError && (
              <span className="maintenance__field-error" role="alert">
                {lineError === "label" ? text.settlement.lineLabelRequired : text.settlement.lineRequired}
              </span>
            )}
          </div>
        </form>
      ) : <p role="status">{text.settlement.none}</p>}
    </section>
  );
}

function EvidenceSection({ detail, capabilities, busy, maintenanceApi, act }: {
  detail: WorkOrderDetail;
  capabilities: MaintenanceCapabilities;
  busy: boolean;
  maintenanceApi: ReturnType<typeof createMaintenanceApi>;
  act: Act;
}) {
  const id = useId();
  const [fileError, setFileError] = useState(false);
  const attach = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    const file = data.get("evidence_file");
    const stage = formText(data, "stage");
    if (!(file instanceof File) || file.size === 0) {
      setFileError(true);
      return;
    }
    setFileError(false);
    const applied = await act(async (signal) => {
      const presigned = await maintenanceApi.presignEvidence({
        work_order_id: detail.id,
        stage: stage as WorkOrderDetail["evidence"][number]["stage"],
        content_type: file.type || "application/octet-stream",
        size_bytes: file.size,
      }, signal);
      const uploadHeaders = new Headers();
      for (const [name, value] of presigned.upload.headers) {
        if (name && value) uploadHeaders.set(name, value);
      }
      const upload = await fetch(presigned.upload.url, {
        method: presigned.upload.method,
        headers: uploadHeaders,
        body: file,
        signal,
      });
      if (!upload.ok) throw new MaintenanceApiError(text.evidence.uploadFailed, upload.status);
      return maintenanceApi.confirmEvidence(presigned.id, signal);
    });
    if (applied) form.reset();
  };

  return (
    <section className="maintenance__section" aria-label={text.evidence.heading}>
      <h3>{text.evidence.heading}</h3>
      {detail.evidence.length ? (
        <ul className="maintenance__timeline">
          {detail.evidence.map((entry) => (
            <li key={entry.id}>
              <span className="maintenance__chip">{text.evidence.stage[entry.stage]}</span>
              <span className={chipClass(WORM_TONE[entry.worm_replica_status])}>
                {text.evidence.worm[entry.worm_replica_status]}
              </span>
              <span>{entry.content_type}</span>
              {entry.verified_at && <time dateTime={entry.verified_at}>{fmtWhen(entry.verified_at)}</time>}
            </li>
          ))}
        </ul>
      ) : <p role="status">{text.evidence.empty}</p>}
      {capabilities.canAttachEvidence && !CLOSED_STATUSES.has(detail.status) && (
        <form className="maintenance__acts" aria-label={text.evidence.attach} onSubmit={(event) => void attach(event)}>
          <div>
            <label htmlFor={`${id}-stage`}>
              {text.evidence.stageLabel}
              <select id={`${id}-stage`} name="stage" defaultValue="DURING">
                {(["REQUEST", "BEFORE", "DURING", "AFTER", "REPORT", "OUTSOURCE_RESULT"] as const).map((value) => (
                  <option key={value} value={value}>{text.evidence.stage[value]}</option>
                ))}
              </select>
            </label>
            <label htmlFor={`${id}-file`}>
              {text.evidence.fileLabel}
              <input id={`${id}-file`} name="evidence_file" type="file" aria-invalid={fileError} />
            </label>
            <button type="submit" className="maintenance__action" disabled={busy}>{text.evidence.attach}</button>
          </div>
          {fileError && <span className="maintenance__field-error" role="alert">{text.evidence.fileRequired}</span>}
        </form>
      )}
    </section>
  );
}
