import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useNavigate } from "react-router";

import type { ConsoleApiClient } from "../../api/client";
import { notifCategoryTone, notifScreenLabel, notifStrings as text } from "../../i18n/notif";
import { categoryLabel } from "../../i18n/notificationCategories";
import { StatusChip } from "../components";
import { TokenText, type ObjectKind, type ObjectRef } from "../composer";
import {
  NotifApiError,
  createNotifApi,
  type NotificationCountsSummary,
  type NotificationLink,
  type NotificationObjectGroup,
  type NotificationPolicySummary,
  type NotificationSummary,
  type ObjectHead,
} from "./notifApi";
import type { NotifCapabilities } from "./notifCapabilities";
import { linkKey, rowTarget, sameLink, timeLabel } from "./notifModel";
import "./notif.css";

type Props = {
  api: ConsoleApiClient;
  actorId: string | undefined;
  capabilities: NotifCapabilities;
  /** Changes whenever auth replaces the effective tenant/session. */
  sessionKey: string | undefined;
};

type View = "time" | "object";
type Filter = "all" | "unread";
type Failure = "load" | "action" | "denied";

const PAGE_LIMIT = 50;
/** Distinct source-object heads resolved per page (bounded fan-out). */
const HEAD_RESOLVE_CAP = 24;

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

function failureOf(cause: unknown, kind: Exclude<Failure, "denied">): Failure {
  if (cause instanceof NotifApiError && (cause.status === 401 || cause.status === 403)) return "denied";
  return kind;
}

function isAbort(cause: unknown): boolean {
  return cause instanceof DOMException && cause.name === "AbortError";
}

function CheckIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M20 6 9 17l-5-5" />
    </svg>
  );
}

function UndoIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
      <path d="M3 3v5h5" />
    </svg>
  );
}

function BellIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" />
      <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
    </svg>
  );
}

function BellOffIcon() {
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M8.7 3A6 6 0 0 1 18 8a21.3 21.3 0 0 0 .6 5" />
      <path d="M17 17H3s3-2 3-9a4.67 4.67 0 0 1 .3-1.7" />
      <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
      <path d="m2 2 20 20" />
    </svg>
  );
}

/**
 * Re-mount synchronously whenever effective authority changes: effects run too
 * late to fence an old session's rows, counts, or in-flight mutation state.
 */
export function NotifScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <NotifScreenInstance key={sessionFence} {...props} />;
}

function NotifScreenInstance({ api, capabilities }: Props) {
  const [view, setView] = useState<View>("time");
  const [filter, setFilter] = useState<Filter>("all");
  const [linkFilter, setLinkFilter] = useState<NotificationLink>();
  const [rows, setRows] = useState<NotificationSummary[]>([]);
  const [rowsCursor, setRowsCursor] = useState<string | null>(null);
  const [groups, setGroups] = useState<NotificationObjectGroup[]>([]);
  const [groupsCursor, setGroupsCursor] = useState<string | null>(null);
  const [summary, setSummary] = useState<NotificationCountsSummary>();
  const [heads, setHeads] = useState<Record<string, ObjectHead | null>>({});
  const [policies, setPolicies] = useState<NotificationPolicySummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<Failure>();
  const generation = useRef(0);
  const operation = useRef<AbortController | undefined>(undefined);
  const knownHeads = useRef(new Set<string>());
  const navigate = useNavigate();
  const notifApi = useMemo(() => createNotifApi(api), [api]);
  const isCurrent = useCallback((token: number) => generation.current === token, []);

  const resolveHeads = useCallback(async (links: NotificationLink[], token: number, signal: AbortSignal) => {
    const distinct = [...new Map(
      links.filter((link) => link.type === "object").map((link) => [linkKey(link), link] as const),
    ).values()].filter((link) => !knownHeads.current.has(linkKey(link))).slice(0, HEAD_RESOLVE_CAP);
    if (distinct.length === 0) return;
    for (const link of distinct) knownHeads.current.add(linkKey(link));
    try {
      const entries = await Promise.all(distinct.map(async (link) => {
        const head = await notifApi.resolveObject(link.kind, link.id, signal);
        return [linkKey(link), head ?? null] as const;
      }));
      if (isCurrent(token)) setHeads((current) => ({ ...current, ...Object.fromEntries(entries) }));
    } catch (cause) {
      // An aborted batch stored nothing — release the claims so a later load retries.
      for (const link of distinct) knownHeads.current.delete(linkKey(link));
      if (!isAbort(cause)) throw cause;
    }
  }, [isCurrent, notifApi]);

  const refreshSummary = useCallback(async (signal?: AbortSignal) => {
    // A failed count read keeps the last authoritative value; the header chip
    // never invents a number.
    try {
      setSummary(await notifApi.summary(signal));
    } catch (cause) {
      if (isAbort(cause)) throw cause;
    }
  }, [notifApi]);

  const refreshGroups = useCallback(async (unread: boolean | undefined, signal: AbortSignal) => {
    const [page, policyList] = await Promise.all([
      notifApi.listByObject({ ...(unread === undefined ? {} : { unread }), limit: PAGE_LIMIT }, signal),
      capabilities.canMute ? notifApi.listPolicies(signal) : Promise.resolve(undefined),
    ]);
    setGroups(page.items);
    setGroupsCursor(page.next_cursor);
    if (policyList) setPolicies(policyList.items);
    return page;
  }, [capabilities.canMute, notifApi]);

  const load = useCallback(async (nextView: View, nextFilter: Filter, before?: string) => {
    if (!capabilities.canRead) {
      setLoading(false);
      return;
    }
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    setLoading(true);
    setFailure(undefined);
    try {
      const unread = nextFilter === "unread" ? true : undefined;
      if (nextView === "time") {
        const page = await notifApi.list({ ...(unread === undefined ? {} : { unread }), ...(before === undefined ? {} : { before }), limit: PAGE_LIMIT }, controller.signal);
        if (!isCurrent(token)) return;
        setRows((current) => (before === undefined ? page.items : [...current, ...page.items]));
        setRowsCursor(page.next_cursor);
        void resolveHeads(page.items.map((row) => row.link), token, controller.signal).catch(() => undefined);
      } else if (before === undefined) {
        const page = await refreshGroups(unread, controller.signal);
        if (!isCurrent(token)) return;
        void resolveHeads(page.items.map((group) => group.link), token, controller.signal).catch(() => undefined);
      } else {
        const page = await notifApi.listByObject({ ...(unread === undefined ? {} : { unread }), before, limit: PAGE_LIMIT }, controller.signal);
        if (!isCurrent(token)) return;
        setGroups((current) => [...current, ...page.items]);
        setGroupsCursor(page.next_cursor);
        void resolveHeads(page.items.map((group) => group.link), token, controller.signal).catch(() => undefined);
      }
      await refreshSummary(controller.signal);
    } catch (cause) {
      if (isCurrent(token) && !controller.signal.aborted) setFailure(failureOf(cause, "load"));
    } finally {
      if (isCurrent(token)) setLoading(false);
    }
  }, [capabilities.canRead, isCurrent, notifApi, refreshGroups, refreshSummary, resolveHeads]);

  useEffect(() => {
    const start = window.setTimeout(() => {
      void load("time", "all");
    }, 0);
    return () => {
      window.clearTimeout(start);
      operation.current?.abort();
    };
  }, [load]);

  const mutate = useCallback(async (work: (signal: AbortSignal) => Promise<void>) => {
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    // The abort above may have cancelled an in-flight load whose finally can no
    // longer clear its flag (stale token) — clear it here or loading sticks true.
    setLoading(false);
    setBusy(true);
    setFailure(undefined);
    try {
      await work(controller.signal);
      return isCurrent(token);
    } catch (cause) {
      if (isCurrent(token) && !controller.signal.aborted) setFailure(failureOf(cause, "action"));
      return false;
    } finally {
      if (isCurrent(token)) setBusy(false);
    }
  }, [isCurrent]);

  const replaceRow = useCallback((next: NotificationSummary) => {
    setRows((current) => current.map((row) => (row.id === next.id ? next : row)));
  }, []);

  const toggleRead = (row: NotificationSummary) => {
    if (!capabilities.canAck) return;
    void mutate(async (signal) => {
      const next = row.unread
        ? await notifApi.markRead(row.id, signal)
        : await notifApi.markUnread(row.id, signal);
      replaceRow(next);
      await refreshSummary(signal);
    });
  };

  const activateRow = (row: NotificationSummary) => {
    if (capabilities.canAck && row.unread) {
      void mutate(async (signal) => {
        replaceRow(await notifApi.markRead(row.id, signal));
        await refreshSummary(signal);
      });
    }
    const target = rowTarget(row.link);
    if (target?.type === "screen") {
      void navigate(target.path);
    } else if (target?.type === "object") {
      setLinkFilter(row.link);
      setView("time");
    }
  };

  const activateGroup = (group: NotificationObjectGroup) => {
    setLinkFilter(group.link);
    setView("time");
  };

  const toggleMute = (group: NotificationObjectGroup) => {
    if (!capabilities.canMute) return;
    void mutate(async (signal) => {
      if (group.muted) {
        const owned = policies.find((policy) =>
          policy.scope === "object" && policy.link != null && sameLink(policy.link, group.link));
        const policy = owned ?? (await notifApi.listPolicies(signal)).items.find((candidate) =>
          candidate.scope === "object" && candidate.link != null && sameLink(candidate.link, group.link));
        if (policy) await notifApi.deletePolicy(policy.id, signal);
      } else {
        await notifApi.upsertPolicy({ scope: "object", link: group.link }, signal);
      }
      await refreshGroups(filter === "unread" ? true : undefined, signal);
      await refreshSummary(signal);
    });
  };

  const markAll = () => {
    if (!capabilities.canAck) return;
    void mutate(async (signal) => {
      await notifApi.markAllRead(signal);
      const unread = filter === "unread" ? true : undefined;
      if (view === "time") {
        const page = await notifApi.list({ ...(unread === undefined ? {} : { unread }), limit: PAGE_LIMIT }, signal);
        setRows(page.items);
        setRowsCursor(page.next_cursor);
      } else {
        await refreshGroups(unread, signal);
      }
      await refreshSummary(signal);
    });
  };

  const changeFilter = (next: Filter) => {
    setFilter(next);
    void load(view, next);
  };

  const changeView = (next: View) => {
    setView(next);
    setLinkFilter(undefined);
    void load(next, filter);
  };

  const codeRefs = useMemo(() => {
    const map: Record<string, ObjectRef> = {};
    for (const head of Object.values(heads)) {
      if (head?.code) map[head.code] = { id: head.id, code: head.code, name: head.title };
    }
    return map;
  }, [heads]);
  const resolveToken = useCallback((_kind: ObjectKind, code: string): ObjectRef | undefined => codeRefs[code], [codeRefs]);
  const openToken = useCallback((_kind: ObjectKind, code: string) => {
    const head = Object.values(heads).find((candidate) => candidate?.code === code);
    if (head) {
      setLinkFilter({ type: "object", kind: head.kind, id: head.id });
      setView("time");
    }
  }, [heads]);

  if (!capabilities.canRead) {
    return (
      <main className="notif">
        <header className="notif__head">
          <h1 className="notif__title">{text.title}</h1>
        </header>
        <section className="notif__card">
          <p className="notif__state" role="status">{text.denied}</p>
        </section>
      </main>
    );
  }

  const visibleRows = linkFilter ? rows.filter((row) => sameLink(row.link, linkFilter)) : rows;
  const cursor = view === "time" ? rowsCursor : groupsCursor;
  const hasData = view === "time" ? visibleRows.length > 0 : groups.length > 0;
  const filterHead = linkFilter ? heads[linkKey(linkFilter)] : undefined;
  const filterChipLabel = linkFilter
    ? linkFilter.type === "screen"
      ? notifScreenLabel(linkFilter.screen)
      : filterHead?.code ?? filterHead?.title ?? linkFilter.kind
    : undefined;

  return (
    <main className="notif" aria-busy={loading || busy}>
      <header className="notif__head">
        <h1 className="notif__title">{text.title}</h1>
        {summary !== undefined && (
          <span className="notif__unreadchip" role="status" aria-label={text.unreadBadge}>{summary.total_unread}</span>
        )}
        {summary?.muted_unread ? (
          <span className="notif__mutedchip" aria-label={text.mutedBadge}>{`${text.mutedShort} ${String(summary.muted_unread)}`}</span>
        ) : null}
        <div className="notif__seg" role="group" aria-label={text.filterLabel}>
          <button type="button" className={filter === "all" ? "notif__segbtn notif__segbtn--on" : "notif__segbtn"} aria-pressed={filter === "all"} onClick={() => { changeFilter("all"); }}>{text.filterAll}</button>
          <button type="button" className={filter === "unread" ? "notif__segbtn notif__segbtn--on" : "notif__segbtn"} aria-pressed={filter === "unread"} onClick={() => { changeFilter("unread"); }}>{text.filterUnread}</button>
        </div>
        <div className="notif__seg" role="group" aria-label={text.viewLabel}>
          <button type="button" className={view === "time" ? "notif__segbtn notif__segbtn--on" : "notif__segbtn"} aria-pressed={view === "time"} onClick={() => { changeView("time"); }}>{text.viewTimeline}</button>
          <button type="button" className={view === "object" ? "notif__segbtn notif__segbtn--on" : "notif__segbtn"} aria-pressed={view === "object"} onClick={() => { changeView("object"); }}>{text.viewByObject}</button>
        </div>
        {linkFilter && filterChipLabel !== undefined && (
          <button type="button" className="notif__filterchip" aria-label={text.objectFilterClear} onClick={() => { setLinkFilter(undefined); }}>
            <span className="notif__filtercode">{filterChipLabel}</span>
            <span aria-hidden="true">×</span>
          </button>
        )}
        <span className="notif__spring" />
        {capabilities.canAck && (
          <button type="button" className="notif__allread" disabled={busy} onClick={markAll}>{text.markAllRead}</button>
        )}
      </header>
      <section className="notif__card" aria-label={view === "time" ? text.list : text.groups}>
        <div className="notif__scroll">
          {failure === "denied" ? (
            <p className="notif__state" role="status">{text.denied}</p>
          ) : (
            <>
              {failure !== undefined && (
                <div className="notif__alert" role="alert">
                  <span>{failure === "action" ? text.actionError : text.loadError}</span>
                  <button type="button" onClick={() => { void load(view, filter); }}>{text.retry}</button>
                </div>
              )}
              {loading && !hasData ? (
                <p className="notif__state" role="status">{text.loading}</p>
              ) : view === "time" ? (
                visibleRows.length === 0 ? (
                  failure === undefined && !loading && (
                    <p className="notif__state" role="status">{filter === "unread" ? text.emptyUnread : text.empty}</p>
                  )
                ) : (
                  <ul className="notif__rows">
                    {visibleRows.map((row) => (
                      <li key={row.id} className={row.unread ? "notif__row notif__row--unread" : "notif__row"}>
                        <button type="button" className="notif__rowopen" aria-label={row.text} onClick={() => { activateRow(row); }} />
                        <span className="notif__dot" aria-hidden="true" />
                        <span className="notif__cat">{categoryLabel(row.category)}</span>
                        <span className="notif__body">
                          <TokenText text={row.text} resolveObject={resolveToken} onOpen={openToken} />
                        </span>
                        <time className="notif__time" dateTime={row.created_at}>{timeLabel(row.created_at)}</time>
                        {capabilities.canAck && (
                          <button type="button" className="notif__toggle" disabled={busy} aria-label={row.unread ? text.markRead : text.markUnread} onClick={() => { toggleRead(row); }}>
                            {row.unread ? <CheckIcon /> : <UndoIcon />}
                          </button>
                        )}
                      </li>
                    ))}
                  </ul>
                )
              ) : groups.length === 0 ? (
                failure === undefined && !loading && (
                  <p className="notif__state" role="status">{text.emptyGroups}</p>
                )
              ) : (
                <ul className="notif__rows">
                  {groups.map((group) => {
                    const key = linkKey(group.link);
                    const head = heads[key];
                    const title = group.link.type === "screen"
                      ? notifScreenLabel(group.link.screen)
                      : head?.code ?? head?.title ?? undefined;
                    return (
                      <li key={key} className={group.unread > 0 ? "notif__row notif__row--unread" : "notif__row"}>
                        <button type="button" className="notif__rowopen" aria-label={group.latest.text} onClick={() => { activateGroup(group); }} />
                        <span className="notif__dot" aria-hidden="true" />
                        {title !== undefined && <span className="notif__code">{title}</span>}
                        <span className="notif__gbody">
                          <span className="notif__gtext">{group.latest.text}</span>
                          <span className="notif__gchips">
                            {group.categories.map((entry) => (
                              <StatusChip key={entry.category} tone={notifCategoryTone(entry.category)}>
                                {`${categoryLabel(entry.category)} ${String(entry.unread)}`}
                              </StatusChip>
                            ))}
                          </span>
                        </span>
                        <span className="notif__count" aria-label={`${text.groupUnread} ${String(group.unread)} · ${text.groupTotal} ${String(group.total)}`}>
                          {`${String(group.unread)}/${String(group.total)}`}
                        </span>
                        <time className="notif__time" dateTime={group.latest.created_at}>{timeLabel(group.latest.created_at)}</time>
                        {capabilities.canMute && (
                          <button type="button" className="notif__toggle" disabled={busy} aria-pressed={group.muted} aria-label={group.muted ? text.unmute : text.mute} onClick={() => { toggleMute(group); }}>
                            {group.muted ? <BellOffIcon /> : <BellIcon />}
                          </button>
                        )}
                      </li>
                    );
                  })}
                </ul>
              )}
              {cursor !== null && !loading && failure === undefined && (
                <button type="button" className="notif__more" onClick={() => { void load(view, filter, cursor); }}>{text.loadMore}</button>
              )}
            </>
          )}
          <div className="notif__tail" aria-hidden="true" />
        </div>
      </section>
    </main>
  );
}
