import { useCallback, useEffect, useMemo, useRef, useState, type DragEvent, type KeyboardEvent } from "react";

import type { ConsoleApiClient } from "../../api/client";
import { directoryStrings as text } from "../../i18n/directory";
import {
  createDirectoryApi,
  isBlocked,
  isDenied,
  type DirectoryEmployee,
  type DirectoryLifecycleEvent,
  type DirectoryMember,
} from "./directoryApi";
import type { DirectoryCapabilities } from "./directoryCapabilities";
import "./directory.css";

type Segment = "members" | "employees";
type ListState = "loading" | "ready" | "error" | "denied";
type CardState = "idle" | "loading" | "ready" | "blocked" | "error";
type Selection = { kind: "member" | "employee"; id: string };

type Props = {
  api: ConsoleApiClient;
  branchId: string | undefined;
  actorId: string | undefined;
  capabilities: DirectoryCapabilities;
  /** Changes whenever auth replaces the effective tenant/session. */
  sessionKey: string | undefined;
  /** `m:<userId>` or `e:<employeeId>` — selection restored across refresh/Back. */
  initialPersonKey?: string;
  onPersonKeyChange?: (key: string | undefined) => void;
  onOpenThread?: (threadId: string) => void;
};

const EMPLOYEE_PAGE = 100;

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

function parsePersonKey(key: string | undefined): Selection | undefined {
  if (!key) return undefined;
  if (key.startsWith("m:")) return { kind: "member", id: key.slice(2) };
  if (key.startsWith("e:")) return { kind: "employee", id: key.slice(2) };
  return undefined;
}

function personKey(selection: Selection | undefined): string | undefined {
  if (!selection) return undefined;
  return `${selection.kind === "member" ? "m" : "e"}:${selection.id}`;
}

function employeeStatusChipClass(status: string): string {
  if (status === "ACTIVE") return "dir-chip dir-chip-ok";
  if (status === "EXITED") return "dir-chip dir-chip-warn";
  return "dir-chip";
}

function employeeStatusLabel(status: string): string {
  if (status in text.status) return text.status[status as keyof typeof text.status];
  return status;
}

function lifecycleTransition(event: DirectoryLifecycleEvent): string {
  const moves = [
    [event.from_company, event.to_company],
    [event.from_org_unit, event.to_org_unit],
    [event.from_position, event.to_position],
  ]
    .filter(([from, to]) => Boolean(to) && from !== to)
    .map(([from, to]) => (from ? `${from} → ${String(to)}` : String(to)));
  return [...moves, event.comment].filter(Boolean).join(" · ");
}

/**
 * Re-mount synchronously whenever effective authority changes: effects run too
 * late to fence an old tenant/session's roster, selection, or card state.
 */
export function DirectoryScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.branchId ?? "no-branch",
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <DirectoryScreenInstance key={sessionFence} {...props} />;
}

function DirectoryScreenInstance({
  api,
  branchId,
  actorId,
  capabilities,
  initialPersonKey,
  onPersonKeyChange,
  onOpenThread,
}: Props) {
  const directoryApi = useMemo(() => createDirectoryApi(api), [api]);

  const [segment, setSegment] = useState<Segment>(() => {
    const initial = parsePersonKey(initialPersonKey);
    if (initial?.kind === "employee" && capabilities.canReadHrDirectory) return "employees";
    return capabilities.canViewPerson ? "members" : "employees";
  });
  const [query, setQuery] = useState("");
  const [companyFilter, setCompanyFilter] = useState<string>();
  const [members, setMembers] = useState<DirectoryMember[]>([]);
  const [membersState, setMembersState] = useState<ListState>("loading");
  const [memberSort, setMemberSort] = useState<{ col: "name" | "team"; dir: 1 | -1 }>();
  const [employees, setEmployees] = useState<DirectoryEmployee[]>([]);
  const [employeesState, setEmployeesState] = useState<ListState>("loading");
  const [employeesTotal, setEmployeesTotal] = useState(0);
  const [employeesPending, setEmployeesPending] = useState(false);
  const [selected, setSelected] = useState<Selection | undefined>(() => {
    const initial = parsePersonKey(initialPersonKey);
    if (!initial) return undefined;
    if (initial.kind === "member" && !capabilities.canViewPerson) return undefined;
    if (initial.kind === "employee" && !capabilities.canReadHrDirectory) return undefined;
    return initial;
  });
  const [profile, setProfile] = useState<DirectoryMember>();
  const [profileState, setProfileState] = useState<CardState>("idle");
  const [events, setEvents] = useState<DirectoryLifecycleEvent[]>([]);
  const [eventsState, setEventsState] = useState<CardState>("idle");
  const [dmState, setDmState] = useState<"idle" | "busy" | "error">("idle");

  const memberRequest = useRef(0);
  const employeeRequest = useRef(0);
  const profileRequest = useRef(0);
  const eventsRequest = useRef(0);
  const dmRequest = useRef(0);
  const employeesRef = useRef<DirectoryEmployee[]>([]);
  const firstEmployeeLoad = useRef(true);
  const announcedKey = useRef<string | undefined>(initialPersonKey);
  const searchRef = useRef<HTMLInputElement>(null);
  const rowRefs = useRef(new Map<string, HTMLButtonElement>());

  // --- roster (messenger tier, active branch scope) -------------------------
  const loadMembers = useCallback(async () => {
    if (!capabilities.canViewPerson || !branchId) return;
    const token = ++memberRequest.current;
    setMembersState("loading");
    try {
      const items = await directoryApi.listMembers(branchId);
      if (token !== memberRequest.current) return;
      setMembers(items);
      setMembersState("ready");
    } catch (cause) {
      if (token !== memberRequest.current) return;
      setMembersState(isDenied(cause) ? "denied" : "error");
    }
  }, [branchId, capabilities.canViewPerson, directoryApi]);

  // --- HR register (employee_directory_read, org-wide, server typeahead) ----
  const loadEmployees = useCallback(async (append: boolean, search: string, company: string | undefined) => {
    if (!capabilities.canReadHrDirectory) return;
    const token = ++employeeRequest.current;
    const offset = append ? employeesRef.current.length : 0;
    if (!append) setEmployeesState("loading");
    setEmployeesPending(true);
    try {
      const page = await directoryApi.listEmployees({
        search: search.trim() || undefined,
        company,
        limit: EMPLOYEE_PAGE,
        offset,
      });
      if (token !== employeeRequest.current) return;
      const next = append ? [...employeesRef.current, ...page.items] : page.items;
      employeesRef.current = next;
      setEmployees(next);
      setEmployeesTotal(page.total);
      setEmployeesState("ready");
    } catch (cause) {
      if (token !== employeeRequest.current) return;
      setEmployeesState(isDenied(cause) ? "denied" : "error");
    } finally {
      if (token === employeeRequest.current) setEmployeesPending(false);
    }
  }, [capabilities.canReadHrDirectory, directoryApi]);

  useEffect(() => {
    const start = window.setTimeout(() => {
      void loadMembers();
    }, 0);
    return () => {
      window.clearTimeout(start);
      memberRequest.current += 1;
    };
  }, [loadMembers]);

  useEffect(() => {
    if (!capabilities.canReadHrDirectory) return;
    const delay = firstEmployeeLoad.current ? 0 : 250;
    firstEmployeeLoad.current = false;
    const start = window.setTimeout(() => {
      void loadEmployees(false, query, companyFilter);
    }, delay);
    return () => {
      window.clearTimeout(start);
    };
  }, [capabilities.canReadHrDirectory, loadEmployees, query, companyFilter]);

  useEffect(() => () => {
    memberRequest.current += 1;
    employeeRequest.current += 1;
    profileRequest.current += 1;
    eventsRequest.current += 1;
    dmRequest.current += 1;
  }, []);

  // --- person card ----------------------------------------------------------
  const selectedMemberId = selected?.kind === "member" ? selected.id : undefined;
  const selectedEmployee = useMemo(
    () => (selected?.kind === "employee" ? employees.find((row) => row.id === selected.id) : undefined),
    [employees, selected],
  );

  /**
   * Every non-self member card open goes through the read-audited profile
   * endpoint (server records `person.view`; 404 = no-leak). Never bypassed.
   */
  const loadProfile = useCallback(async () => {
    if (!selectedMemberId || !branchId || !capabilities.canViewPerson) return;
    const token = ++profileRequest.current;
    setProfile(undefined);
    setProfileState("loading");
    setDmState("idle");
    try {
      const summary = await directoryApi.getMember(selectedMemberId, branchId);
      if (token !== profileRequest.current) return;
      setProfile(summary);
      setProfileState("ready");
    } catch (cause) {
      if (token !== profileRequest.current) return;
      setProfileState(isBlocked(cause) ? "blocked" : "error");
    }
  }, [branchId, capabilities.canViewPerson, directoryApi, selectedMemberId]);

  useEffect(() => {
    dmRequest.current += 1;
    if (!selectedMemberId) {
      // The member card is not rendered without a member selection; stale card
      // state stays invisible and the next load resets it before showing.
      profileRequest.current += 1;
      return;
    }
    const start = window.setTimeout(() => {
      void loadProfile();
    }, 0);
    return () => {
      window.clearTimeout(start);
    };
  }, [loadProfile, selectedMemberId]);

  const loadEvents = useCallback(async () => {
    const employeeId = selected?.kind === "employee" ? selected.id : undefined;
    if (!employeeId || !capabilities.canReadHrDirectory) return;
    const token = ++eventsRequest.current;
    setEvents([]);
    setEventsState("loading");
    try {
      const items = await directoryApi.listLifecycleEvents(employeeId);
      if (token !== eventsRequest.current) return;
      setEvents(items);
      setEventsState("ready");
    } catch (cause) {
      if (token !== eventsRequest.current) return;
      // Deny-by-omission: a denied ledger renders as absent, not as an error.
      setEventsState(isDenied(cause) ? "idle" : "error");
    }
  }, [capabilities.canReadHrDirectory, directoryApi, selected]);

  useEffect(() => {
    if (selected?.kind !== "employee") {
      // The employee card is not rendered without an employee selection; the
      // next load resets the ledger before it becomes visible again.
      eventsRequest.current += 1;
      return;
    }
    const start = window.setTimeout(() => {
      void loadEvents();
    }, 0);
    return () => {
      window.clearTimeout(start);
    };
  }, [loadEvents, selected]);

  useEffect(() => {
    const key = personKey(selected);
    if (key === announcedKey.current) return;
    announcedKey.current = key;
    onPersonKeyChange?.(key);
  }, [onPersonKeyChange, selected]);

  // A selected employee that the (re)loaded, server-filtered register no longer
  // contains — or that a server-denied register can no longer address — is no
  // longer selectable; selection follows the authorized list.
  useEffect(() => {
    if (employeesState !== "ready" && employeesState !== "denied") return;
    setSelected((current) => {
      if (current?.kind !== "employee") return current;
      if (employeesState === "denied") return undefined;
      return employeesRef.current.some((row) => row.id === current.id) ? current : undefined;
    });
  }, [employees, employeesState]);

  // --- actions --------------------------------------------------------------
  const startDm = useCallback(async () => {
    if (!capabilities.canMessage || !branchId || !selectedMemberId || selectedMemberId === actorId) return;
    const token = ++dmRequest.current;
    setDmState("busy");
    try {
      const thread = await directoryApi.createDmThread(branchId, selectedMemberId);
      if (token !== dmRequest.current) return;
      setDmState("idle");
      onOpenThread?.(thread.id);
    } catch {
      if (token !== dmRequest.current) return;
      setDmState("error");
    }
  }, [actorId, branchId, capabilities.canMessage, directoryApi, onOpenThread, selectedMemberId]);

  // --- derived rows ---------------------------------------------------------
  const filteredMembers = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const matched = needle
      ? members.filter((member) =>
          member.display_name.toLowerCase().includes(needle) ||
          (member.team ?? "").toLowerCase().includes(needle))
      : members;
    if (!memberSort) return matched;
    const { col, dir } = memberSort;
    return [...matched].sort((a, b) => {
      const left = col === "name" ? a.display_name : a.team ?? "";
      const right = col === "name" ? b.display_name : b.team ?? "";
      return left.localeCompare(right, "ko") * dir;
    });
  }, [memberSort, members, query]);

  const hrLive = capabilities.canReadHrDirectory && employeesState !== "denied";
  const activeSegment: Segment = segment === "employees" && hrLive ? "employees" : "members";

  const clearFilters = useCallback(() => {
    setQuery("");
    setCompanyFilter(undefined);
  }, []);

  const cycleMemberSort = useCallback((col: "name" | "team") => {
    setMemberSort((current) => {
      if (current?.col !== col) return { col, dir: 1 };
      if (current.dir === 1) return { col, dir: -1 };
      return undefined;
    });
  }, []);

  const moveWithin = useCallback((keys: readonly string[], currentId: string, event: KeyboardEvent) => {
    const index = keys.indexOf(currentId);
    if (index < 0) return;
    const nextIndex =
      event.key === "Home" ? 0 :
      event.key === "End" ? keys.length - 1 :
      event.key === "ArrowDown" || event.key === "j" ? Math.min(index + 1, keys.length - 1) :
      event.key === "ArrowUp" || event.key === "k" ? Math.max(index - 1, 0) :
      index;
    if (nextIndex === index && event.key !== "Home" && event.key !== "End") return;
    event.preventDefault();
    return keys[nextIndex];
  }, []);

  const registerRow = useCallback((id: string) => (node: HTMLButtonElement | null) => {
    if (node) rowRefs.current.set(id, node);
    else rowRefs.current.delete(id);
  }, []);

  const focusRow = useCallback((id: string) => {
    queueMicrotask(() => rowRefs.current.get(id)?.focus());
  }, []);

  const dragPayload = useCallback((token: string) => (event: DragEvent) => {
    event.dataTransfer.setData("text/plain", token);
  }, []);

  // Denied before any fetch (no capability), or fully reconciled-denied: the
  // only granted tier was the HR register and the server refused it.
  if (!capabilities.canRead || (!capabilities.canViewPerson && employeesState === "denied")) {
    return (
      <section aria-label={text.title} className="dir-screen">
        <p className="dir-denied" role="status">{text.denied}</p>
      </section>
    );
  }

  const memberKeys = filteredMembers.map((member) => member.id);
  const employeeKeys = employees.map((row) => row.id);
  const selfSelected = Boolean(selectedMemberId) && selectedMemberId === actorId;

  return (
    <section aria-label={text.title} className="dir-screen">
      <header className="dir-bar">
        <h1 className="dir-title">{text.title}</h1>
        {capabilities.canViewPerson ? (
          <button
            type="button"
            className="dir-stat"
            aria-pressed={activeSegment === "members"}
            onClick={() => { setSegment("members"); }}
          >
            <span className="dir-stat-label">{text.statMembers}</span>
            <span className="dir-stat-value">{members.length.toLocaleString("ko-KR")}</span>
          </button>
        ) : null}
        {hrLive ? (
          <button
            type="button"
            className="dir-stat"
            aria-pressed={activeSegment === "employees"}
            onClick={() => { setSegment("employees"); }}
          >
            <span className="dir-stat-label">{text.statEmployees}</span>
            <span className="dir-stat-value">{employeesTotal.toLocaleString("ko-KR")}</span>
          </button>
        ) : null}
        <span className="dir-bar-spacer" />
        {companyFilter ? (
          <span className="dir-filter">
            <span>{`${text.kv.company} · ${companyFilter}`}</span>
            <button type="button" aria-label={text.removeFilter} onClick={() => { setCompanyFilter(undefined); }}>×</button>
          </span>
        ) : null}
        <label className="dir-search">
          <input
            ref={searchRef}
            value={query}
            aria-label={text.search}
            placeholder={text.search}
            onChange={(event) => { setQuery(event.target.value); }}
          />
        </label>
        {capabilities.canMessage ? (
          <button type="button" className="dir-action" onClick={() => searchRef.current?.focus()}>
            {text.newConversation}
          </button>
        ) : null}
      </header>

      <div className="dir-body">
        <div className="dir-list-panel" aria-busy={activeSegment === "employees" ? employeesPending : membersState === "loading"}>
          {activeSegment === "members" ? (
            <>
              <div className="dir-head dir-grid-members">
                <button type="button" className="dir-sort" title={text.sortHint} onClick={() => { cycleMemberSort("name"); }}>
                  {text.cols.name}{memberSort?.col === "name" ? (memberSort.dir === 1 ? " ↑" : " ↓") : ""}
                </button>
                <button type="button" className="dir-sort" title={text.sortHint} onClick={() => { cycleMemberSort("team"); }}>
                  {text.cols.team}{memberSort?.col === "team" ? (memberSort.dir === 1 ? " ↑" : " ↓") : ""}
                </button>
                <span />
              </div>
              <div className="dir-scroll">
                {membersState === "loading" ? <p className="dir-state" role="status">{text.loading}</p> : null}
                {membersState === "denied" ? <p className="dir-state" role="status">{text.denied}</p> : null}
                {membersState === "error" ? (
                  <div className="dir-alert" role="alert">
                    <p>{text.membersLoadError}</p>
                    <button type="button" className="dir-ghost" onClick={() => { void loadMembers(); }}>{text.retry}</button>
                  </div>
                ) : null}
                {membersState === "ready" && filteredMembers.length === 0 ? (
                  <div className="dir-empty">
                    <span className="dir-empty-title">{text.empty}</span>
                    <button type="button" className="dir-ghost" onClick={clearFilters}>{text.clearFilter}</button>
                  </div>
                ) : null}
                <div role="listbox" aria-label={text.membersLabel}>
                  {filteredMembers.map((member, index) => {
                    const isSelected = selectedMemberId === member.id;
                    return (
                      <button
                        key={member.id}
                        ref={registerRow(member.id)}
                        type="button"
                        role="option"
                        aria-selected={isSelected}
                        tabIndex={isSelected || (!selectedMemberId && index === 0) ? 0 : -1}
                        className="dir-row dir-grid-members"
                        title={text.rowHint}
                        draggable
                        onDragStart={dragPayload(`@${member.display_name}`)}
                        onClick={() => { setSelected({ kind: "member", id: member.id }); }}
                        onKeyDown={(event) => {
                          const next = moveWithin(memberKeys, member.id, event);
                          if (next) { setSelected({ kind: "member", id: next }); focusRow(next); }
                        }}
                      >
                        <span className="dir-name">{member.display_name}</span>
                        <span className="dir-cell">{member.team ?? text.none}</span>
                        <span className="dir-chevron" aria-hidden="true">›</span>
                      </button>
                    );
                  })}
                </div>
              </div>
            </>
          ) : (
            <>
              <div className="dir-head dir-grid-employees">
                <span className="dir-col">{text.cols.code}</span>
                <span className="dir-col">{text.cols.name}</span>
                <span className="dir-col">{text.cols.position}</span>
                <span className="dir-col">{text.cols.team}</span>
                <span className="dir-col">{text.kv.company}</span>
                <span />
              </div>
              <div className="dir-scroll">
                {employeesState === "loading" ? <p className="dir-state" role="status">{text.loading}</p> : null}
                {employeesState === "error" ? (
                  <div className="dir-alert" role="alert">
                    <p>{text.employeesLoadError}</p>
                    <button type="button" className="dir-ghost" onClick={() => { void loadEmployees(false, query, companyFilter); }}>{text.retry}</button>
                  </div>
                ) : null}
                {employeesState === "ready" && employees.length === 0 ? (
                  <div className="dir-empty">
                    <span className="dir-empty-title">{text.empty}</span>
                    <button type="button" className="dir-ghost" onClick={clearFilters}>{text.clearFilter}</button>
                  </div>
                ) : null}
                <div role="listbox" aria-label={text.employeesLabel}>
                  {employees.map((row, index) => {
                    const isSelected = selected?.kind === "employee" && selected.id === row.id;
                    const refToken = `[${row.employee_number ?? text.none} ${row.name}]`;
                    return (
                      <button
                        key={row.id}
                        ref={registerRow(row.id)}
                        type="button"
                        role="option"
                        aria-selected={isSelected}
                        tabIndex={isSelected || (selected?.kind !== "employee" && index === 0) ? 0 : -1}
                        className="dir-row dir-grid-employees"
                        title={text.rowHint}
                        draggable
                        onDragStart={dragPayload(refToken)}
                        onClick={() => { setSelected({ kind: "employee", id: row.id }); }}
                        onKeyDown={(event) => {
                          const next = moveWithin(employeeKeys, row.id, event);
                          if (next) { setSelected({ kind: "employee", id: next }); focusRow(next); }
                        }}
                      >
                        <span className="dir-code">{row.employee_number ?? text.none}</span>
                        <span className="dir-name">{row.name}</span>
                        <span className="dir-cell">{row.position ?? text.none}</span>
                        <span className="dir-cell">{row.org_unit ?? text.none}</span>
                        <span><span className="dir-chip">{row.company}</span></span>
                        <span className="dir-chevron" aria-hidden="true">›</span>
                      </button>
                    );
                  })}
                </div>
                {employeesState === "ready" && employees.length < employeesTotal ? (
                  <button type="button" className="dir-more" disabled={employeesPending} onClick={() => { void loadEmployees(true, query, companyFilter); }}>
                    {text.loadMore}
                  </button>
                ) : null}
              </div>
            </>
          )}
        </div>

        <aside className="dir-detail" aria-label={text.detail} aria-busy={profileState === "loading" || eventsState === "loading"}>
          {!selected ? <p className="dir-state">{text.detailEmpty}</p> : null}

          {selected?.kind === "member" ? (
            <>
              {profileState === "loading" ? <p className="dir-state" role="status">{text.detailLoading}</p> : null}
              {profileState === "blocked" ? <p className="dir-state" role="status">{text.detailBlocked}</p> : null}
              {profileState === "error" ? (
                <div className="dir-alert" role="alert">
                  <p>{text.detailError}</p>
                  <button type="button" className="dir-ghost" onClick={() => { void loadProfile(); }}>{text.retry}</button>
                </div>
              ) : null}
              {profileState === "ready" && profile ? (
                <>
                  <div className="dir-detail-head">
                    <div className="dir-detail-chips">
                      {profile.team ? <span className="dir-chip">{profile.team}</span> : null}
                      {selfSelected ? <span className="dir-chip">{text.self}</span> : (
                        <span className="dir-chip dir-chip-info">{text.viewLogged}</span>
                      )}
                    </div>
                    <div className="dir-detail-title">{profile.display_name}</div>
                  </div>
                  <div className="dir-detail-body">
                    <dl>
                      <div className="dir-kv"><dt>{text.kv.team}</dt><dd>{profile.team ?? text.none}</dd></div>
                    </dl>
                    {profile.team ? (
                      <div className="dir-en-row">
                        <button
                          type="button"
                          className="dir-en"
                          title={text.filterByTeam}
                          onClick={() => { setSegment("members"); setQuery(profile.team ?? ""); }}
                        >
                          <span className="dir-en-k">{text.kv.team}</span>
                          <span className="dir-en-v">{profile.team}</span>
                        </button>
                      </div>
                    ) : null}
                    {dmState === "error" ? (
                      <div className="dir-alert" role="alert">
                        <p>{text.messageError}</p>
                        <button type="button" className="dir-ghost" onClick={() => { void startDm(); }}>{text.retry}</button>
                      </div>
                    ) : null}
                    {!selfSelected && capabilities.canMessage ? (
                      <button type="button" className="dir-cta" disabled={dmState === "busy"} onClick={() => { void startDm(); }}>
                        {dmState === "busy" ? text.messageBusy : text.message}
                      </button>
                    ) : null}
                  </div>
                </>
              ) : null}
            </>
          ) : null}

          {selected?.kind === "employee" && !selectedEmployee ? (
            employeesState === "error" ? (
              <div className="dir-alert" role="alert">
                <p>{text.detailError}</p>
                <button type="button" className="dir-ghost" onClick={() => { void loadEmployees(false, query, companyFilter); }}>{text.retry}</button>
              </div>
            ) : employeesState !== "ready" ? (
              <p className="dir-state" role="status">{text.detailLoading}</p>
            ) : null
          ) : null}

          {selectedEmployee ? (
            <>
              <div className="dir-detail-head">
                <div className="dir-detail-chips">
                  {selectedEmployee.employee_number ? <span className="dir-code">{selectedEmployee.employee_number}</span> : null}
                  <span className="dir-chip">{selectedEmployee.company}</span>
                  {selectedEmployee.status ? (
                    <span className={employeeStatusChipClass(selectedEmployee.status)}>
                      {employeeStatusLabel(selectedEmployee.status)}
                    </span>
                  ) : null}
                </div>
                <div className="dir-detail-title">{selectedEmployee.name}</div>
              </div>
              <div className="dir-detail-body">
                <dl>
                  <div className="dir-kv"><dt>{text.kv.position}</dt><dd>{selectedEmployee.position ?? text.none}</dd></div>
                  <div className="dir-kv"><dt>{text.kv.job}</dt><dd>{selectedEmployee.job ?? text.none}</dd></div>
                  <div className="dir-kv"><dt>{text.kv.team}</dt><dd>{selectedEmployee.org_unit ?? text.none}</dd></div>
                  <div className="dir-kv"><dt>{text.kv.worksite}</dt><dd>{selectedEmployee.worksite_name ?? selectedEmployee.worksite ?? text.none}</dd></div>
                  <div className="dir-kv"><dt>{text.kv.hireDate}</dt><dd>{selectedEmployee.hire_date ?? text.none}</dd></div>
                  {selectedEmployee.exit_date ? (
                    <div className="dir-kv"><dt>{text.kv.exitDate}</dt><dd>{selectedEmployee.exit_date}</dd></div>
                  ) : null}
                  <div className="dir-kv"><dt>{text.kv.homeBranch}</dt><dd>{selectedEmployee.home_branch_name ?? text.none}</dd></div>
                </dl>
                <div className="dir-en-row">
                  <button
                    type="button"
                    className="dir-en"
                    title={text.filterByCompany}
                    onClick={() => { setSegment("employees"); setCompanyFilter(selectedEmployee.company); }}
                  >
                    <span className="dir-en-k">{text.kv.company}</span>
                    <span className="dir-en-v">{selectedEmployee.company}</span>
                  </button>
                  {selectedEmployee.org_unit ? (
                    <button
                      type="button"
                      className="dir-en"
                      title={text.filterByTeam}
                      onClick={() => { setSegment("employees"); setQuery(selectedEmployee.org_unit ?? ""); }}
                    >
                      <span className="dir-en-k">{text.kv.team}</span>
                      <span className="dir-en-v">{selectedEmployee.org_unit}</span>
                    </button>
                  ) : null}
                </div>
                <div className="dir-section">
                  <h3>{text.history}</h3>
                  {eventsState === "loading" ? <p className="dir-state" role="status">{text.loading}</p> : null}
                  {eventsState === "error" ? (
                    <div className="dir-alert" role="alert">
                      <p>{text.historyError}</p>
                      <button type="button" className="dir-ghost" onClick={() => { void loadEvents(); }}>{text.retry}</button>
                    </div>
                  ) : null}
                  {eventsState === "ready" && events.length === 0 ? <p className="dir-state">{text.historyEmpty}</p> : null}
                  {events.map((event) => (
                    <div key={event.id} className="dir-event">
                      <span className="dir-event-date">{event.effective_date}</span>
                      <span className="dir-chip">{text.event[event.event_type]}</span>
                      <span className="dir-event-body">{lifecycleTransition(event)}</span>
                    </div>
                  ))}
                </div>
              </div>
            </>
          ) : null}
        </aside>
      </div>
    </section>
  );
}
