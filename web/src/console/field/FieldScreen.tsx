import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type SyntheticEvent,
} from "react";

import type { ConsoleApiClient } from "../../api/client";
import {
  allowedTransitions,
  categoryLabel,
  formatDateTime,
  originLabel,
  priorityLabel,
  statusLabel,
  SUPPORT_CATEGORIES,
  SUPPORT_PRIORITIES,
  ticketCode,
  transitionActionLabel,
} from "../../features/support/support-format";
import { fieldStrings as text } from "../../i18n/field";
import { ko } from "../../i18n/ko";
import { StatusChip } from "../components";
import { priorityTone, statusTone } from "../screens/support/model";
import {
  createFieldApi,
  FieldApiError,
  type AcceptanceChannel,
  type AcceptanceKind,
  type FieldSiteDetail,
  type FieldSiteRow,
  type FieldSlaState,
  type TicketCategory,
  type TicketDetail,
  type TicketPriority,
  type TicketStatus,
} from "./fieldApi";
import type { FieldCapabilities } from "./fieldCapabilities";
import "./field.css";

type Props = {
  api: ConsoleApiClient;
  branchId: string;
  actorId: string | undefined;
  capabilities: FieldCapabilities;
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

function formText(data: FormData, name: string): string {
  const value = data.get(name);
  return typeof value === "string" ? value : "";
}

interface OpSlot {
  current: AbortController | undefined;
}

/**
 * An async op may settle state only while it is still its pane's latest
 * un-aborted op. Each pane owns its slot, so concurrent panes never invalidate
 * each other (a shared counter would strand `reconcile`'s parallel reloads),
 * and session changes remount the whole screen via the fence key.
 */
function settles(slot: OpSlot, controller: AbortController): boolean {
  return slot.current === controller && !controller.signal.aborted;
}

const SLA_STATES: FieldSlaState[] = ["OK", "AT_RISK", "BREACHED"];
const ACCEPTANCE_KINDS: AcceptanceKind[] = ["CUSTOMER_ACCEPTED", "CUSTOMER_DECLINED"];
const ACCEPTANCE_CHANNELS: AcceptanceChannel[] = ["IN_PERSON", "PHONE", "EMAIL", "MESSENGER"];

function slaTone(state: FieldSlaState): "ok" | "warn" | "danger" {
  if (state === "BREACHED") return "danger";
  if (state === "AT_RISK") return "warn";
  return "ok";
}

/** WO- object code from the ref (request_no when assigned, id tail otherwise —
 * same derivation pattern as `ticketCode`). */
function workOrderCode(ref: { id: string; request_no: string | null }): string {
  if (ref.request_no) return ref.request_no;
  const cleaned = ref.id.replaceAll(/[^0-9A-Za-z]/gu, "");
  return `WO-${cleaned.slice(-4).toUpperCase()}`;
}

function workOrderStatusLabel(status: string): string {
  const labels = ko.status as Record<string, string>;
  return labels[status] ?? status;
}

interface IntakeDraft {
  category: TicketCategory;
  priority: TicketPriority;
  title: string;
  body: string;
}

const EMPTY_DRAFT: IntakeDraft = {
  category: "OPERATIONAL",
  priority: "MEDIUM",
  title: "",
  body: "",
};

interface StoredState {
  selectedSiteId?: string;
  draft?: IntakeDraft;
}

function readStore(key: string): StoredState {
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as StoredState | null;
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

function writeStore(key: string, value: StoredState) {
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Storage quota/denied — persistence is best-effort, never blocking.
  }
}

/**
 * Re-mount synchronously whenever effective authority changes. Effects run too
 * late to fence an old tenant/session's selected site, error, or busy state.
 */
export function FieldScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.branchId,
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <FieldScreenInner key={sessionFence} {...props} />;
}

function FieldScreenInner({ api, branchId, actorId, capabilities, sessionKey }: Props) {
  const storeKey = `mnt.field.${sessionKey ?? "anon"}.${branchId}`;

  const [rows, setRows] = useState<FieldSiteRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [listError, setListError] = useState<string>();
  const [query, setQuery] = useState("");
  const [slaFilter, setSlaFilter] = useState<FieldSlaState>();
  const [customerFilter, setCustomerFilter] = useState<{ id: string; name: string }>();
  const [issuesDrill, setIssuesDrill] = useState(false);
  const [cursorId, setCursorId] = useState<string>();
  const [selectedSiteId, setSelectedSiteId] = useState<string | undefined>(
    () => readStore(storeKey).selectedSiteId,
  );

  const [detail, setDetail] = useState<FieldSiteDetail>();
  const [detailState, setDetailState] = useState<"idle" | "loading" | "absent" | "error">("idle");

  const [selectedTicketId, setSelectedTicketId] = useState<string>();
  const [ticket, setTicket] = useState<TicketDetail>();
  const [ticketState, setTicketState] = useState<"idle" | "loading" | "error">("idle");

  const [intakeOpen, setIntakeOpen] = useState(false);
  const [draft, setDraft] = useState<IntakeDraft>(() => readStore(storeKey).draft ?? EMPTY_DRAFT);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string>();

  const fieldApi = useMemo(() => createFieldApi(api), [api]);
  const listOp = useRef<AbortController>(undefined);
  const detailOp = useRef<AbortController>(undefined);
  const ticketOp = useRef<AbortController>(undefined);
  const mutateOp = useRef<AbortController>(undefined);

  const categoryId = useId();
  const priorityId = useId();
  const titleId = useId();
  const bodyId = useId();
  const acceptKindId = useId();
  const acceptChannelId = useId();
  const acceptById = useId();
  const acceptNoteId = useId();
  const commentId = useId();

  useEffect(() => {
    writeStore(storeKey, { selectedSiteId, draft });
  }, [storeKey, selectedSiteId, draft]);

  const loadList = useCallback(async () => {
    if (!capabilities.canRead) {
      setRows([]);
      setLoading(false);
      return;
    }
    listOp.current?.abort();
    const controller = new AbortController();
    listOp.current = controller;
    setLoading(true);
    setListError(undefined);
    try {
      const page = await fieldApi.listSites(
        {
          ...(query.trim() ? { q: query.trim() } : {}),
          ...(slaFilter ? { sla: slaFilter } : {}),
          ...(customerFilter ? { customer_id: customerFilter.id } : {}),
          limit: 100,
        },
        controller.signal,
      );
      if (settles(listOp, controller)) setRows(page.items);
    } catch (cause) {
      if (settles(listOp, controller)) {
        setListError(message(cause, text.loadError));
      }
    } finally {
      if (settles(listOp, controller)) setLoading(false);
    }
  }, [capabilities.canRead, customerFilter, fieldApi, query, slaFilter]);

  useEffect(() => {
    listOp.current?.abort();
    // 200ms defer doubles as the search-typing debounce.
    const start = window.setTimeout(() => {
      void loadList();
    }, 200);
    return () => {
      window.clearTimeout(start);
      listOp.current?.abort();
    };
  }, [loadList, sessionKey]);

  const loadDetail = useCallback(async () => {
    if (!capabilities.canRead || !selectedSiteId) {
      setDetail(undefined);
      setDetailState("idle");
      return;
    }
    detailOp.current?.abort();
    const controller = new AbortController();
    detailOp.current = controller;
    setDetailState("loading");
    try {
      const next = await fieldApi.getSite(selectedSiteId, controller.signal);
      if (settles(detailOp, controller)) {
        setDetail(next);
        setDetailState("idle");
      }
    } catch (cause) {
      if (!settles(detailOp, controller)) return;
      // Out-of-scope reads deny by omission (404); render absence, not failure.
      if (cause instanceof FieldApiError && (cause.status === 404 || cause.status === 403)) {
        setDetail(undefined);
        setDetailState("absent");
      } else {
        setDetailState("error");
      }
    }
  }, [capabilities.canRead, fieldApi, selectedSiteId]);

  useEffect(() => {
    const start = window.setTimeout(() => {
      setDetail(undefined);
      setSelectedTicketId(undefined);
      void loadDetail();
    }, 0);
    return () => {
      window.clearTimeout(start);
      detailOp.current?.abort();
    };
  }, [loadDetail]);

  const loadTicket = useCallback(async () => {
    if (!selectedTicketId) {
      setTicket(undefined);
      setTicketState("idle");
      return;
    }
    ticketOp.current?.abort();
    const controller = new AbortController();
    ticketOp.current = controller;
    setTicketState("loading");
    try {
      const next = await fieldApi.getTicket(selectedTicketId, controller.signal);
      if (settles(ticketOp, controller)) {
        setTicket(next);
        setTicketState("idle");
      }
    } catch {
      if (settles(ticketOp, controller)) setTicketState("error");
    }
  }, [fieldApi, selectedTicketId]);

  useEffect(() => {
    const start = window.setTimeout(() => {
      setTicket(undefined);
      void loadTicket();
    }, 0);
    return () => {
      window.clearTimeout(start);
      ticketOp.current?.abort();
    };
  }, [loadTicket]);

  /** Run a mutation, then reconcile every open surface from the server. */
  const mutate = useCallback(
    async (work: (signal: AbortSignal) => Promise<unknown>, fallback: string) => {
      mutateOp.current?.abort();
      const controller = new AbortController();
      mutateOp.current = controller;
      setBusy(true);
      setActionError(undefined);
      try {
        await work(controller.signal);
        return settles(mutateOp, controller);
      } catch (cause) {
        if (settles(mutateOp, controller)) {
          setActionError(message(cause, fallback));
        }
        return false;
      } finally {
        if (settles(mutateOp, controller)) setBusy(false);
      }
    },
    [],
  );

  const reconcile = useCallback(async () => {
    await Promise.all([loadList(), loadDetail(), loadTicket()]);
  }, [loadDetail, loadList, loadTicket]);

  const visibleRows = useMemo(
    () => (issuesDrill ? rows.filter((row) => row.open_ticket_count > 0) : rows),
    [issuesDrill, rows],
  );

  const stats = useMemo(
    () => ({
      breached: rows.filter((row) => row.sla === "BREACHED").length,
      openIssues: rows.filter((row) => row.open_ticket_count > 0).length,
      total: rows.length,
    }),
    [rows],
  );

  const filtersActive =
    slaFilter !== undefined || customerFilter !== undefined || issuesDrill || query.trim() !== "";

  const clearFilters = () => {
    setSlaFilter(undefined);
    setCustomerFilter(undefined);
    setIssuesDrill(false);
    setQuery("");
  };

  const onKeyNav = (event: KeyboardEvent<HTMLDivElement>) => {
    const target = event.target as HTMLElement;
    if (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT") {
      return;
    }
    const key = event.key;
    const ids = visibleRows.map((row) => row.site_id);
    if (key === "j" || key === "J" || key === "ArrowDown" || key === "k" || key === "K" || key === "ArrowUp") {
      if (ids.length === 0) return;
      const down = key === "j" || key === "J" || key === "ArrowDown";
      const cur = cursorId ? ids.indexOf(cursorId) : -1;
      const next = cur === -1 ? 0 : down ? Math.min(cur + 1, ids.length - 1) : Math.max(cur - 1, 0);
      setCursorId(ids[next]);
      event.preventDefault();
    } else if (key === "Enter") {
      if (cursorId) {
        setSelectedSiteId(cursorId);
        event.preventDefault();
      }
    }
  };

  const submitIntake = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!capabilities.canIntake || busy) return;
    const title = draft.title.trim();
    const body = draft.body.trim();
    if (!title || !body) return;
    const applied = await mutate(async (signal) => {
      const created = await fieldApi.createTicket(
        {
          branch_id: branchId,
          category: draft.category,
          priority: draft.priority,
          title,
          body,
        },
        signal,
      );
      setSelectedTicketId(created.id);
      if (capabilities.canTriage && selectedSiteId) {
        // The ticket exists once created — a failed link must not fail the
        // intake (resubmitting would duplicate it). The ticket pane keeps the
        // manual link action for retry.
        try {
          await fieldApi.linkTicket(created.id, { site_id: selectedSiteId }, signal);
        } catch (cause) {
          setActionError(message(cause, text.ticket.linkFailed));
        }
      }
    }, text.intakeForm.failed);
    if (applied) {
      setDraft(EMPTY_DRAFT);
      setIntakeOpen(false);
      await reconcile();
    }
  };

  const runTransition = async (to: TicketStatus) => {
    if (!selectedTicketId || !capabilities.canTriage) return;
    const applied = await mutate(
      (signal) => fieldApi.transitionTicket(selectedTicketId, to, signal),
      text.actionError,
    );
    if (applied) await reconcile();
  };

  const runAssignSelf = async () => {
    if (!selectedTicketId || !capabilities.canTriage || !actorId) return;
    const applied = await mutate(
      (signal) =>
        fieldApi.assignTicket(
          selectedTicketId,
          { assignee_user_id: actorId, branch_id: branchId },
          signal,
        ),
      text.actionError,
    );
    if (applied) await reconcile();
  };

  const runLinkSite = async () => {
    if (!selectedTicketId || !capabilities.canTriage || !selectedSiteId) return;
    const applied = await mutate(
      (signal) => fieldApi.linkTicket(selectedTicketId, { site_id: selectedSiteId }, signal),
      text.ticket.linkFailed,
    );
    if (applied) await reconcile();
  };

  const submitAcceptance = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedTicketId || !capabilities.canAccept || busy) return;
    const form = event.currentTarget;
    const data = new FormData(form);
    const acceptedBy = formText(data, "accepted_by").trim();
    const note = formText(data, "note").trim();
    const kind = formText(data, "kind") as AcceptanceKind;
    const channel = formText(data, "channel") as AcceptanceChannel;
    if (!acceptedBy) return;
    const applied = await mutate(
      (signal) =>
        fieldApi.recordAcceptance(
          selectedTicketId,
          { kind, channel, accepted_by: acceptedBy, ...(note ? { note } : {}) },
          signal,
        ),
      text.acceptance.failed,
    );
    if (applied) {
      form.reset();
      await reconcile();
    }
  };

  const submitComment = async (event: SyntheticEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!selectedTicketId || !capabilities.canComment || busy) return;
    const form = event.currentTarget;
    const data = new FormData(form);
    const body = formText(data, "body").trim();
    if (!body) return;
    const applied = await mutate(
      (signal) =>
        fieldApi.addComment(
          selectedTicketId,
          { body, is_internal_note: data.get("internal") === "on" },
          signal,
        ),
      text.actionError,
    );
    if (applied) {
      form.reset();
      await reconcile();
    }
  };

  if (!capabilities.canRead) {
    return (
      <main className="field">
        <header className="field__head">
          <h1>{text.title}</h1>
        </header>
        <p className="field__note" role="status">{text.denied}</p>
      </main>
    );
  }

  return (
    <main className="field" aria-busy={loading || busy}>
      <header className="field__head">
        <h1>{text.title}</h1>
        <div className="field__stats" role="group" aria-label={text.statsLabel}>
          <button
            type="button"
            className="field__stat"
            aria-pressed={slaFilter === "BREACHED"}
            onClick={() => {
              setSlaFilter((current) => (current === "BREACHED" ? undefined : "BREACHED"));
            }}
          >
            <span>{text.stats.breached}</span>
            <strong className="field__stat-value field__stat-value--danger">
              {stats.breached === 0 ? "—" : stats.breached}
            </strong>
          </button>
          <button
            type="button"
            className="field__stat"
            aria-pressed={issuesDrill}
            onClick={() => {
              setIssuesDrill((current) => !current);
            }}
          >
            <span>{text.stats.openIssues}</span>
            <strong className="field__stat-value field__stat-value--warn">
              {stats.openIssues === 0 ? "—" : stats.openIssues}
            </strong>
          </button>
          <button
            type="button"
            className="field__stat"
            aria-pressed={!filtersActive}
            onClick={clearFilters}
          >
            <span>{text.stats.total}</span>
            <strong className="field__stat-value">{stats.total === 0 ? "—" : stats.total}</strong>
          </button>
        </div>
        <input
          type="search"
          className="field__search"
          value={query}
          aria-label={text.searchLabel}
          onChange={(event) => {
            setQuery(event.currentTarget.value);
          }}
        />
        {capabilities.canIntake && (
          <button
            type="button"
            className="field__primary"
            aria-expanded={intakeOpen}
            onClick={() => {
              setIntakeOpen((current) => !current);
            }}
          >
            {text.intake}
          </button>
        )}
      </header>

      <div className="field__filters" role="group" aria-label={text.slaFilterLabel}>
        <button
          type="button"
          className="field__chipbtn"
          aria-pressed={slaFilter === undefined}
          onClick={() => {
            setSlaFilter(undefined);
          }}
        >
          {text.allFilter}
        </button>
        {SLA_STATES.map((state) => (
          <button
            key={state}
            type="button"
            className="field__chipbtn"
            aria-pressed={slaFilter === state}
            onClick={() => {
              setSlaFilter((current) => (current === state ? undefined : state));
            }}
          >
            {text.sla[state]}
          </button>
        ))}
        {customerFilter && (
          <button
            type="button"
            className="field__chipbtn"
            aria-pressed={true}
            onClick={() => {
              setCustomerFilter(undefined);
            }}
          >
            {customerFilter.name}
          </button>
        )}
      </div>

      {intakeOpen && capabilities.canIntake && (
        <form className="field__form" aria-label={text.intake} onSubmit={(event) => void submitIntake(event)}>
          <div className="field__form-grid">
            <label htmlFor={categoryId}>
              {ko.support.form.category}
              <select
                id={categoryId}
                value={draft.category}
                onChange={(event) => {
                  const category = event.currentTarget.value as TicketCategory;
                  setDraft((current) => ({ ...current, category }));
                }}
              >
                {SUPPORT_CATEGORIES.map((category) => (
                  <option key={category} value={category}>{categoryLabel(category)}</option>
                ))}
              </select>
            </label>
            <label htmlFor={priorityId}>
              {ko.support.form.priority}
              <select
                id={priorityId}
                value={draft.priority}
                onChange={(event) => {
                  const priority = event.currentTarget.value as TicketPriority;
                  setDraft((current) => ({ ...current, priority }));
                }}
              >
                {SUPPORT_PRIORITIES.map((priority) => (
                  <option key={priority} value={priority}>{priorityLabel(priority)}</option>
                ))}
              </select>
            </label>
          </div>
          <label htmlFor={titleId}>
            {ko.support.form.ticketTitle}
            <input
              id={titleId}
              value={draft.title}
              maxLength={200}
              required
              onChange={(event) => {
                const title = event.currentTarget.value;
                setDraft((current) => ({ ...current, title }));
              }}
            />
          </label>
          <label htmlFor={bodyId}>
            {ko.support.form.body}
            <textarea
              id={bodyId}
              value={draft.body}
              maxLength={4000}
              required
              onChange={(event) => {
                const body = event.currentTarget.value;
                setDraft((current) => ({ ...current, body }));
              }}
            />
          </label>
          <div className="field__form-row">
            <StatusChip tone="neutral">
              {`${text.intakeForm.site} ${detail?.site.name ?? text.intakeForm.noSite}`}
            </StatusChip>
            <button
              type="submit"
              className="field__primary"
              disabled={busy || !draft.title.trim() || !draft.body.trim()}
            >
              {busy ? text.intakeForm.submitting : text.intakeForm.submit}
            </button>
            <button
              type="button"
              className="field__secondary"
              onClick={() => {
                setIntakeOpen(false);
              }}
            >
              {ko.common.cancel}
            </button>
          </div>
        </form>
      )}

      {actionError && (
        <div className="field__alert" role="alert">
          <span>{actionError}</span>
          <button
            type="button"
            onClick={() => {
              setActionError(undefined);
            }}
          >
            {ko.common.cancel}
          </button>
        </div>
      )}

      <div className="field__content">
        <section className="field__list-pane">
          {listError ? (
            <div className="field__alert" role="alert">
              <span>{listError}</span>
              <button
                type="button"
                onClick={() => {
                  void loadList();
                }}
              >
                {text.retry}
              </button>
            </div>
          ) : loading ? (
            <p className="field__note" role="status">{text.loading}</p>
          ) : (
            <div
              className="field__table"
              role="grid"
              tabIndex={0}
              aria-label={text.listLabel}
              onKeyDown={onKeyNav}
            >
              <div className="field__row field__row--head" role="row">
                <span role="columnheader">{text.cols.site}</span>
                <span role="columnheader" className="field__cell--wide">{text.cols.customer}</span>
                <span role="columnheader" className="field__cell--wide">{text.cols.load}</span>
                <span role="columnheader">{text.cols.sla}</span>
              </div>
              {visibleRows.length === 0 ? (
                <div className="field__empty">
                  <span role="status">{filtersActive ? text.emptyFiltered : text.empty}</span>
                  {filtersActive && (
                    <button type="button" className="field__secondary" onClick={clearFilters}>
                      {text.clearFilters}
                    </button>
                  )}
                </div>
              ) : (
                visibleRows.map((row) => {
                  const active = row.site_id === selectedSiteId;
                  const cursor = row.site_id === cursorId;
                  return (
                    <div
                      key={row.site_id}
                      role="row"
                      aria-selected={active || cursor}
                      className={
                        active
                          ? "field__row field__row--selected"
                          : cursor
                            ? "field__row field__row--cursor"
                            : "field__row"
                      }
                      onClick={() => {
                        setCursorId(row.site_id);
                        setSelectedSiteId(row.site_id);
                      }}
                    >
                      <span role="gridcell" className="field__site-name">{row.site_name}</span>
                      <span role="gridcell" className="field__cell--wide">{row.customer_name}</span>
                      <span role="gridcell" className="field__cell--wide">
                        {row.open_ticket_count === 0 && row.active_work_order_count === 0
                          ? "—"
                          : `${text.issueCount(row.open_ticket_count)} · ${text.workOrderCount(row.active_work_order_count)}`}
                      </span>
                      <span role="gridcell">
                        <StatusChip tone={slaTone(row.sla)}>{text.sla[row.sla]}</StatusChip>
                      </span>
                    </div>
                  );
                })
              )}
            </div>
          )}
        </section>

        <aside className="field__detail" aria-live="polite" aria-label={text.detail.label}>
          {selectedTicketId ? (
            <TicketPane
              ticket={ticket}
              state={ticketState}
              busy={busy}
              actorId={actorId}
              siteId={selectedSiteId}
              capabilities={capabilities}
              ids={{
                kind: acceptKindId,
                channel: acceptChannelId,
                by: acceptById,
                note: acceptNoteId,
                comment: commentId,
              }}
              onBack={() => {
                setSelectedTicketId(undefined);
              }}
              onRetry={() => {
                void loadTicket();
              }}
              onTransition={(to) => void runTransition(to)}
              onAssignSelf={() => void runAssignSelf()}
              onLinkSite={() => void runLinkSite()}
              onSubmitAcceptance={(event) => void submitAcceptance(event)}
              onSubmitComment={(event) => void submitComment(event)}
            />
          ) : !selectedSiteId ? (
            <p className="field__note">{text.detail.select}</p>
          ) : detailState === "loading" ? (
            <p className="field__note" role="status">{text.detail.loading}</p>
          ) : detailState === "absent" ? (
            <p className="field__note" role="status">{text.detail.absent}</p>
          ) : detailState === "error" ? (
            <div className="field__alert" role="alert">
              <span>{text.detail.loadError}</span>
              <button
                type="button"
                onClick={() => {
                  void loadDetail();
                }}
              >
                {text.retry}
              </button>
            </div>
          ) : detail ? (
            <SitePane
              detail={detail}
              onFilterCustomer={(id, name) => {
                setCustomerFilter({ id, name });
              }}
              onSearchContact={(name) => {
                setQuery(name);
              }}
              onOpenTicket={(id) => {
                setSelectedTicketId(id);
              }}
            />
          ) : null}
        </aside>
      </div>
    </main>
  );
}

function SitePane({
  detail,
  onFilterCustomer,
  onSearchContact,
  onOpenTicket,
}: {
  detail: FieldSiteDetail;
  onFilterCustomer: (id: string, name: string) => void;
  onSearchContact: (name: string) => void;
  onOpenTicket: (id: string) => void;
}) {
  const { site, sla } = detail;
  // The server carries the site contact as a name and a phone; the chip shows
  // whichever are set and searches on that same string.
  const contact = [site.contact_name, site.contact_phone].filter(Boolean).join(" · ");
  return (
    <>
      <header className="field__detail-head">
        <h2>{site.name}</h2>
        <StatusChip tone={slaTone(sla.state)}>{text.sla[sla.state]}</StatusChip>
        {sla.open > 0 && <StatusChip tone="warn">{text.issueCount(sla.open)}</StatusChip>}
        {sla.breached > 0 && (
          <StatusChip tone="danger">{`${text.sla.BREACHED} ${String(sla.breached)}`}</StatusChip>
        )}
      </header>
      <dl className="field__kv">
        <dt>{text.detail.customer}</dt>
        <dd>
          <button
            type="button"
            className="field__linkchip"
            aria-label={text.detail.filterByCustomer(site.customer_name)}
            onClick={() => {
              onFilterCustomer(site.customer_id, site.customer_name);
            }}
          >
            {site.customer_name}
          </button>
        </dd>
        {site.address && (
          <>
            <dt>{text.detail.address}</dt>
            <dd>{site.address}</dd>
          </>
        )}
        {contact && (
          <>
            <dt>{text.detail.contact}</dt>
            <dd>
              <button
                type="button"
                className="field__linkchip"
                aria-label={text.detail.searchContact(contact)}
                onClick={() => {
                  onSearchContact(contact);
                }}
              >
                {contact}
              </button>
            </dd>
          </>
        )}
        {site.geofence_radius_m !== null && (
          <>
            <dt>{text.detail.geofence}</dt>
            <dd>{text.detail.meters(site.geofence_radius_m)}</dd>
          </>
        )}
        {sla.next_due_at && (
          <>
            <dt>{text.detail.nextDue}</dt>
            <dd>{formatDateTime(sla.next_due_at)}</dd>
          </>
        )}
        <dt>{text.detail.sla90d}</dt>
        <dd>{text.detail.sla90dValue(sla.resolved_within_sla_90d, sla.resolved_breached_90d)}</dd>
      </dl>

      <section className="field__section" aria-label={text.detail.tickets}>
        <h3>{text.detail.tickets}</h3>
        {detail.tickets.length === 0 ? (
          <p className="field__note">{text.detail.sectionEmpty}</p>
        ) : (
          <ul className="field__items">
            {detail.tickets.map((item) => (
              <li key={item.id}>
                <button
                  type="button"
                  className="field__item"
                  aria-label={text.ticket.open(item.title)}
                  onClick={() => {
                    onOpenTicket(item.id);
                  }}
                >
                  <span className="field__item-title">{item.title}</span>
                  <StatusChip tone={priorityTone(item.priority)}>
                    {priorityLabel(item.priority)}
                  </StatusChip>
                  <StatusChip tone={statusTone(item.status)}>{statusLabel(item.status)}</StatusChip>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="field__section" aria-label={text.detail.workOrders}>
        <h3>{text.detail.workOrders}</h3>
        {detail.work_orders.length === 0 ? (
          <p className="field__note">{text.detail.sectionEmpty}</p>
        ) : (
          <ul className="field__items">
            {detail.work_orders.map((ref) => {
              const code = workOrderCode(ref);
              return (
                <li key={ref.id} className="field__item">
                  <a
                    className="field__linkchip"
                    aria-label={text.workOrder.open(code)}
                    href={`/dispatch?around_work_order_id=${encodeURIComponent(ref.id)}`}
                  >
                    {code}
                  </a>
                  <StatusChip tone="neutral">{workOrderStatusLabel(ref.status)}</StatusChip>
                  {ref.report_submitted_at && (
                    <StatusChip tone="ok">{text.workOrder.reportSubmitted}</StatusChip>
                  )}
                  {ref.target_due_at && <span>{formatDateTime(ref.target_due_at)}</span>}
                </li>
              );
            })}
          </ul>
        )}
      </section>

      <section className="field__section" aria-label={text.detail.attendance}>
        <h3>{text.detail.attendance}</h3>
        {detail.attendance.length === 0 ? (
          <p className="field__note">{text.detail.sectionEmpty}</p>
        ) : (
          <ul className="field__items">
            {detail.attendance.map((event) => (
              <li
                key={`${event.user_id}:${event.kind}:${event.occurred_at}`}
                className="field__item"
              >
                <span>{event.user_name ?? ko.common.unknown}</span>
                <StatusChip tone={event.kind === "ARRIVAL" ? "ok" : "neutral"}>
                  {/* kind is the compliance crate's open vocabulary, not a closed enum */}
                  {(text.attendanceKind as Record<string, string>)[event.kind] ?? event.kind}
                </StatusChip>
                <span>{formatDateTime(event.occurred_at)}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="field__section" aria-label={text.detail.acceptances}>
        <h3>{text.detail.acceptances}</h3>
        {detail.acceptances.length === 0 ? (
          <p className="field__note">{text.detail.sectionEmpty}</p>
        ) : (
          <ul className="field__items">
            {detail.acceptances.map((view) => (
              <li key={view.id} className="field__item">
                <StatusChip tone={view.kind === "CUSTOMER_ACCEPTED" ? "ok" : "danger"}>
                  {text.acceptance.kinds[view.kind]}
                </StatusChip>
                <span>{view.accepted_by}</span>
                <StatusChip tone="neutral">{text.acceptance.channels[view.channel]}</StatusChip>
                <span>{formatDateTime(view.occurred_at)}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}

function TicketPane({
  ticket,
  state,
  busy,
  actorId,
  siteId,
  capabilities,
  ids,
  onBack,
  onRetry,
  onTransition,
  onAssignSelf,
  onLinkSite,
  onSubmitAcceptance,
  onSubmitComment,
}: {
  ticket: TicketDetail | undefined;
  state: "idle" | "loading" | "error";
  busy: boolean;
  actorId: string | undefined;
  siteId: string | undefined;
  capabilities: FieldCapabilities;
  ids: { kind: string; channel: string; by: string; note: string; comment: string };
  onBack: () => void;
  onRetry: () => void;
  onTransition: (to: TicketStatus) => void;
  onAssignSelf: () => void;
  onLinkSite: () => void;
  onSubmitAcceptance: (event: SyntheticEvent<HTMLFormElement>) => void;
  onSubmitComment: (event: SyntheticEvent<HTMLFormElement>) => void;
}) {
  return (
    <section className="field__ticket" aria-label={text.ticket.label}>
      <button type="button" className="field__secondary" onClick={onBack}>
        {text.detail.back}
      </button>
      {state === "loading" ? (
        <p className="field__note" role="status">{text.detail.loading}</p>
      ) : state === "error" || !ticket ? (
        <div className="field__alert" role="alert">
          <span>{text.ticket.loadError}</span>
          <button type="button" onClick={onRetry}>{text.retry}</button>
        </div>
      ) : (
        <>
          <div className="field__chips">
            <StatusChip tone="neutral">{ticketCode(ticket.ticket.id)}</StatusChip>
            <StatusChip tone={statusTone(ticket.ticket.status)}>
              {statusLabel(ticket.ticket.status)}
            </StatusChip>
            <StatusChip tone={priorityTone(ticket.ticket.priority)}>
              {priorityLabel(ticket.ticket.priority)}
            </StatusChip>
            <StatusChip tone="neutral">{categoryLabel(ticket.ticket.category)}</StatusChip>
            <StatusChip tone="neutral">{originLabel(ticket.ticket.origin)}</StatusChip>
            {ticket.ticket.site_name && (
              <StatusChip tone="accent">{ticket.ticket.site_name}</StatusChip>
            )}
          </div>
          <h2>{ticket.ticket.title}</h2>
          <dl className="field__kv">
            <dt>{ko.support.requester}</dt>
            <dd>{ticket.ticket.requester_name ?? ko.common.unknown}</dd>
            <dt>{ko.support.assignee}</dt>
            <dd>
              {ticket.ticket.assignee_user_id
                ? (ticket.ticket.assignee_name ?? ko.common.unknown)
                : ko.support.unassigned}
            </dd>
            <dt>{ko.support.dueAt}</dt>
            <dd>{formatDateTime(ticket.ticket.due_at)}</dd>
            <dt>{ko.support.createdAt}</dt>
            <dd>{formatDateTime(ticket.ticket.created_at)}</dd>
          </dl>

          {capabilities.canTriage && (
            <div className="field__actions">
              {allowedTransitions(ticket.ticket.status).map((to) => (
                <button
                  key={to}
                  type="button"
                  className="field__secondary"
                  disabled={busy}
                  onClick={() => {
                    onTransition(to);
                  }}
                >
                  {transitionActionLabel(ticket.ticket.status, to)}
                </button>
              ))}
              {actorId && ticket.ticket.assignee_user_id !== actorId && (
                <button
                  type="button"
                  className="field__secondary"
                  disabled={busy}
                  onClick={onAssignSelf}
                >
                  {ko.support.assignSelf}
                </button>
              )}
              {!ticket.ticket.site_id && siteId && (
                <button
                  type="button"
                  className="field__secondary"
                  disabled={busy}
                  onClick={onLinkSite}
                >
                  {busy ? text.ticket.linking : text.ticket.linkSite}
                </button>
              )}
            </div>
          )}

          {capabilities.canAccept && ticket.ticket.status === "RESOLVED" && (
            <form
              className="field__form"
              aria-label={text.detail.acceptances}
              onSubmit={onSubmitAcceptance}
            >
              <h3>{text.detail.acceptances}</h3>
              <div className="field__form-grid">
                <label htmlFor={ids.kind}>
                  {text.acceptance.kind}
                  <select id={ids.kind} name="kind" defaultValue="CUSTOMER_ACCEPTED">
                    {ACCEPTANCE_KINDS.map((kind) => (
                      <option key={kind} value={kind}>{text.acceptance.kinds[kind]}</option>
                    ))}
                  </select>
                </label>
                <label htmlFor={ids.channel}>
                  {text.acceptance.channel}
                  <select id={ids.channel} name="channel" defaultValue="IN_PERSON">
                    {ACCEPTANCE_CHANNELS.map((channel) => (
                      <option key={channel} value={channel}>
                        {text.acceptance.channels[channel]}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <label htmlFor={ids.by}>
                {text.acceptance.acceptedBy}
                <input
                  id={ids.by}
                  name="accepted_by"
                  maxLength={200}
                  required
                  placeholder={text.acceptance.acceptedByPlaceholder}
                />
              </label>
              <label htmlFor={ids.note}>
                {text.acceptance.note}
                <textarea id={ids.note} name="note" maxLength={2000} />
              </label>
              <button type="submit" className="field__primary" disabled={busy}>
                {busy ? text.acceptance.recording : text.acceptance.record}
              </button>
            </form>
          )}

          <section className="field__section" aria-label={ko.support.comments.title}>
            <h3>{ko.support.comments.title}</h3>
            {ticket.comments.length === 0 ? (
              <p className="field__note">{ko.support.comments.empty}</p>
            ) : (
              <ul className="field__items">
                {ticket.comments.map((comment) => (
                  <li key={comment.id} className="field__comment">
                    <div className="field__chips">
                      {comment.is_internal_note && (
                        <StatusChip tone="warn">{ko.support.comments.internalNote}</StatusChip>
                      )}
                      <strong>
                        {comment.author_user_id
                          ? (comment.author_name ?? ko.common.unknown)
                          : ko.support.comments.systemAuthor}
                      </strong>
                      <span>{formatDateTime(comment.created_at)}</span>
                    </div>
                    <p>{comment.body}</p>
                  </li>
                ))}
              </ul>
            )}
            {capabilities.canComment && (
              <form className="field__form" onSubmit={onSubmitComment}>
                <label htmlFor={ids.comment}>
                  {ko.support.comments.title}
                  <textarea id={ids.comment} name="body" maxLength={4000} required />
                </label>
                <label className="field__check">
                  <input type="checkbox" name="internal" />
                  {ko.support.comments.markInternal}
                </label>
                <button type="submit" className="field__secondary" disabled={busy}>
                  {busy ? ko.support.comments.adding : ko.support.comments.add}
                </button>
              </form>
            )}
          </section>
        </>
      )}
    </section>
  );
}
