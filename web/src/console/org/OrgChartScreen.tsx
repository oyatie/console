import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { ConsoleApiClient } from "../../api/client";
import { orgStrings as text } from "../../i18n/org";
import {
  createOrgApi,
  OrgApiError,
  type BranchSummary,
  type HrOrgChartResponse,
  type HrOrgChartUnit,
  type OrgChangeDetail,
  type OrgChangeSummary,
  type OrgEntitySummary,
  type OrgProposalOp,
  type RegionSummary,
} from "./orgApi";
import type { OrgCapabilities } from "./orgCapabilities";
import { applyPendingOps, buildOrgColumns, totalActive, type OrgTreeColumn, type OrgTreeSite } from "./orgTree";
import { OrgEntityCard, OrgSiteCard, OrgTeamCard } from "./OrgCards";
import { OrgChangeModal, type OrgChangeModalMode } from "./OrgChangeModal";
import "./org.css";

type Props = {
  api: ConsoleApiClient;
  actorId: string | undefined;
  capabilities: OrgCapabilities;
  /** Changes whenever auth replaces the effective tenant/session. */
  sessionKey: string | undefined;
  onNavigate: (screen: string) => void;
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

const SANDBOX_KEY = "console.org.sandbox";

interface Sandbox {
  actorId: string;
  ops: OrgProposalOp[];
  openChangeId?: string;
}

function readSandbox(actorId: string | undefined): Sandbox | undefined {
  if (!actorId) return undefined;
  try {
    const raw = window.sessionStorage.getItem(SANDBOX_KEY);
    if (!raw) return undefined;
    const parsed = JSON.parse(raw) as Sandbox;
    return parsed.actorId === actorId && Array.isArray(parsed.ops) ? parsed : undefined;
  } catch {
    return undefined;
  }
}

function writeSandbox(sandbox: Sandbox | undefined): void {
  try {
    if (!sandbox || (sandbox.ops.length === 0 && !sandbox.openChangeId)) {
      window.sessionStorage.removeItem(SANDBOX_KEY);
    } else {
      window.sessionStorage.setItem(SANDBOX_KEY, JSON.stringify(sandbox));
    }
  } catch {
    // Storage unavailable (private mode) — the sandbox simply won't survive refresh.
  }
}

function errorMessage(cause: unknown, fallback: string): string {
  return cause instanceof Error ? cause.message : fallback;
}

function isDenied(cause: unknown): boolean {
  return cause instanceof OrgApiError && (cause.status === 401 || cause.status === 403 || cause.status === 404);
}

type ChangesState =
  | { kind: "ready"; items: OrgChangeSummary[] }
  | { kind: "error"; message: string }
  | { kind: "unavailable" };

function changeStatusChipClass(status: OrgChangeSummary["status"]): string {
  switch (status) {
    case "REJECTED":
    case "CANCELLED":
      return "org-chip org-chip--danger";
    case "APPLIED":
    case "ARCHIVED":
      return "org-chip org-chip--ok";
    case "IN_APPROVAL":
    case "APPROVED":
    case "SETTLING":
      return "org-chip org-chip--warn";
    default:
      return "org-chip";
  }
}

/**
 * Re-mount synchronously whenever effective authority changes (production
 * exemplar): effects run too late to fence an old session's tree or sandbox.
 */
export function OrgChartScreen(props: Props) {
  const capabilityKey = Object.values(props.capabilities).join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.actorId ?? "no-actor",
    apiFenceKey(props.api),
    capabilityKey,
  ].join(":");
  return <OrgChartScreenBodyInner key={sessionFence} {...props} />;
}

function OrgChartScreenBodyInner({ api, actorId, capabilities, sessionKey, onNavigate }: Props) {
  const orgApi = useMemo(() => createOrgApi(api), [api]);
  const [loading, setLoading] = useState(true);
  const [treeError, setTreeError] = useState<string>();
  const [sideError, setSideError] = useState<string>();
  const [chart, setChart] = useState<HrOrgChartResponse>();
  const [regions, setRegions] = useState<RegionSummary[]>([]);
  const [branches, setBranches] = useState<BranchSummary[]>([]);
  const [entities, setEntities] = useState<OrgEntitySummary[]>([]);
  const [viewerCompany, setViewerCompany] = useState<string | null>(null);
  const [changes, setChanges] = useState<ChangesState>();
  const [openColumns, setOpenColumns] = useState<string[]>([]);
  const [edit, setEdit] = useState(false);
  const [pendingOps, setPendingOps] = useState<OrgProposalOp[]>(() => readSandbox(actorId)?.ops ?? []);
  const [guard, setGuard] = useState<string>();
  const [entityCard, setEntityCard] = useState<OrgTreeColumn>();
  const [siteCard, setSiteCard] = useState<OrgTreeSite>();
  const [teamCard, setTeamCard] = useState<{ column: OrgTreeColumn; unit: HrOrgChartUnit }>();
  const [modal, setModal] = useState<OrgChangeModalMode | undefined>(() => {
    const restored = capabilities.canReadChanges ? readSandbox(actorId)?.openChangeId : undefined;
    return restored ? { kind: "existing", id: restored } : undefined;
  });
  const [addSiteOpen, setAddSiteOpen] = useState(false);
  const [addSiteRegion, setAddSiteRegion] = useState("");
  const [addSiteName, setAddSiteName] = useState("");
  const generation = useRef(0);
  const operation = useRef<AbortController | undefined>(undefined);

  useEffect(() => {
    if (!actorId) return;
    writeSandbox({
      actorId,
      ops: pendingOps,
      openChangeId: modal?.kind === "existing" ? modal.id : undefined,
    });
  }, [actorId, modal, pendingOps]);

  const isCurrent = useCallback((token: number) => generation.current === token, []);

  const loadChanges = useCallback(async (signal?: AbortSignal) => {
    if (!capabilities.canReadChanges) {
      setChanges(undefined);
      return;
    }
    try {
      const page = await orgApi.listChanges({ limit: 50 }, signal);
      setChanges({ kind: "ready", items: page.items });
    } catch (cause) {
      if (signal?.aborted) return;
      if (cause instanceof OrgApiError && cause.status === 404) {
        setChanges({ kind: "unavailable" });
      } else if (isDenied(cause)) {
        setChanges(undefined);
      } else {
        setChanges({ kind: "error", message: errorMessage(cause, text.changesError) });
      }
    }
  }, [capabilities.canReadChanges, orgApi]);

  const load = useCallback(async () => {
    if (!capabilities.canReadTree) {
      setLoading(false);
      return;
    }
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    setLoading(true);
    setTreeError(undefined);
    setSideError(undefined);
    const [chartR, regionsR, branchesR, meR, entitiesR] = await Promise.allSettled([
      orgApi.orgChart(controller.signal),
      orgApi.regions(controller.signal),
      orgApi.branches(controller.signal),
      orgApi.me(controller.signal),
      orgApi.entities(controller.signal),
    ]);
    if (!isCurrent(token) || controller.signal.aborted) return;
    if (chartR.status === "fulfilled") {
      setChart(chartR.value);
    } else {
      setTreeError(errorMessage(chartR.reason, text.loadError));
    }
    let side = false;
    if (regionsR.status === "fulfilled") setRegions(regionsR.value);
    else if (!isDenied(regionsR.reason)) side = true;
    if (branchesR.status === "fulfilled") setBranches(branchesR.value);
    else if (!isDenied(branchesR.reason)) side = true;
    if (meR.status === "fulfilled") setViewerCompany(meR.value.employee_company);
    else if (!isDenied(meR.reason)) side = true;
    if (entitiesR.status === "fulfilled") setEntities(entitiesR.value);
    else if (!isDenied(entitiesR.reason)) side = true;
    if (side) setSideError(text.loadError);
    await loadChanges(controller.signal);
    if (isCurrent(token)) setLoading(false);
  }, [capabilities.canReadTree, isCurrent, loadChanges, orgApi]);

  useEffect(() => {
    generation.current += 1;
    operation.current?.abort();
    const start = window.setTimeout(() => {
      void load();
    }, 0);
    return () => {
      window.clearTimeout(start);
      operation.current?.abort();
    };
  }, [load, sessionKey]);

  const baseColumns = useMemo(
    () =>
      chart
        ? buildOrgColumns({ chart, entities, regions, branches, viewerCompany })
        : [],
    [branches, chart, entities, regions, viewerCompany],
  );
  const siteOwner = useMemo(
    () => baseColumns.find((column) => column.sites.length > 0)?.company
      ?? (viewerCompany && baseColumns.some((column) => column.company === viewerCompany) ? viewerCompany : null),
    [baseColumns, viewerCompany],
  );
  const columns = useMemo(
    () => applyPendingOps(baseColumns, pendingOps, siteOwner),
    [baseColumns, pendingOps, siteOwner],
  );
  const headcount = totalActive(columns);
  const siteCount = columns.reduce((sum, column) => sum + column.sites.length, 0);
  const teamCount = columns.reduce((sum, column) => sum + column.units.length, 0);

  const pushOp = useCallback((op: OrgProposalOp) => {
    setPendingOps((current) => [...current, op]);
    setGuard(undefined);
  }, []);

  const draftTarget = useCallback((column?: OrgTreeColumn) => {
    const owner = column ?? columns.find((candidate) => candidate.company === siteOwner) ?? columns.at(0);
    return owner
      ? {
        kind: "ENTITY" as const,
        ref: owner.entity?.orgId ?? owner.company,
        label: owner.company,
      }
      : { kind: "ENTITY" as const, ref: "", label: "" };
  }, [columns, siteOwner]);

  const openDraftModal = useCallback((column?: OrgTreeColumn, kind: "NEW" | "REORG" | "DISSOLVE" = "REORG") => {
    setEntityCard(undefined);
    setModal({ kind: "new", seed: { kind, target: draftTarget(column), proposal: pendingOps } });
  }, [draftTarget, pendingOps]);

  const closeEdit = useCallback(() => {
    setEdit(false);
    setAddSiteOpen(false);
    setGuard(undefined);
    if (pendingOps.length > 0) openDraftModal();
  }, [openDraftModal, pendingOps.length]);

  const onModalChanged = useCallback((detail: OrgChangeDetail) => {
    setModal((current) => {
      if (current?.kind === "new") {
        setPendingOps([]);
        return { kind: "existing", id: detail.id };
      }
      return current;
    });
    setChanges((current) =>
      current?.kind === "ready"
        ? {
          kind: "ready",
          items: current.items.some((item) => item.id === detail.id)
            ? current.items.map((item) => (item.id === detail.id ? { ...item, ...detail } : item))
            : [{ ...detail }, ...current.items],
        }
        : current,
    );
    if (detail.status === "APPLIED" || detail.status === "SETTLING" || detail.status === "ARCHIVED") {
      void load();
    }
  }, [load]);

  if (!capabilities.canReadTree) {
    return (
      <main className="org">
        <section className="org-panel" aria-labelledby="org-title">
          <h1 id="org-title">{text.title}</h1>
          <p role="status">{text.denied}</p>
        </section>
      </main>
    );
  }

  return (
    <main className="org" aria-busy={loading}>
      <header className="org-head">
        <div className="org-head-main">
          <h1 id="org-title">{text.title}</h1>
          {pendingOps.length > 0 && (
            <div className="org-dirty-banner" role="status">
              <span>{`${text.dirtyBanner} ${String(pendingOps.length)}${text.dirtyBannerUnit}`}</span>
              {capabilities.canDraft && (
                <button
                  type="button"
                  onClick={() => {
                    openDraftModal();
                  }}
                >
                  {text.dirtyCta}
                </button>
              )}
            </div>
          )}
        </div>
        <span className="org-spacer" />
        {capabilities.canDraft && !loading && !treeError && columns.length > 0 && (
          <button
            type="button"
            className="org-secondary"
            aria-pressed={edit}
            onClick={() => {
              if (edit) closeEdit();
              else setEdit(true);
            }}
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M21.17 2.83a2.83 2.83 0 0 0-4 0L4 16l-1 5 5-1L21.17 6.83a2.83 2.83 0 0 0 0-4z" /></svg>
            {edit ? text.editDone : text.edit}
          </button>
        )}
      </header>

      {capabilities.canReadChanges && changes && (
        <div className="org-changes-strip" aria-label={text.changesStrip}>
          <span className="org-section-label">{text.changesStrip}</span>
          {changes.kind === "unavailable" && <span className="org-empty-line" role="status">{text.changesUnavailable}</span>}
          {changes.kind === "error" && (
            <span className="org-empty-line" role="status">
              {changes.message}
              <button
                type="button"
                className="org-link-button"
                onClick={() => {
                  void loadChanges();
                }}
              >
                {text.retry}
              </button>
            </span>
          )}
          {changes.kind === "ready" && changes.items.length === 0 && (
            <span className="org-empty-line" role="status">{text.changesEmpty}</span>
          )}
          {changes.kind === "ready" && changes.items.map((item) => (
            <button
              key={item.id}
              type="button"
              className="org-change-chip"
              onClick={() => {
                setModal({ kind: "existing", id: item.id });
              }}
            >
              <span className="org-mono">{item.code}</span>
              <span>{item.target.label}</span>
              <span className="org-chip">{text.ocKinds[item.kind]}</span>
              <span className={changeStatusChipClass(item.status)}>{text.ocStage[item.status]}</span>
            </button>
          ))}
        </div>
      )}

      {guard && <div className="org-banner org-banner--danger" role="status">{guard}</div>}
      {(treeError ?? sideError) && (
        <div className="org-alert" role="alert">
          <span>{treeError ?? sideError}</span>
          <button
            type="button"
            onClick={() => {
              void load();
            }}
          >
            {text.retry}
          </button>
        </div>
      )}

      {loading ? (
        <p role="status">{text.loading}</p>
      ) : !treeError && (
        columns.length === 0 ? (
          <p role="status">{text.empty}</p>
        ) : (
          <div className="org-canvas">
            <div className="org-canvas-inner">
              <div className="org-root-row">
                <div className="org-root-card">
                  <span className="org-stat"><span className="org-stat-label">{text.rootMeta}</span><span className="org-stat-value org-mono">{String(headcount)}</span></span>
                  <span className="org-stat"><span className="org-stat-label">{text.entity}</span><span className="org-stat-value org-mono">{String(columns.length)}</span></span>
                  <span className="org-stat"><span className="org-stat-label">{text.ocStatSites}</span><span className="org-stat-value org-mono">{String(siteCount)}</span></span>
                  <span className="org-stat"><span className="org-stat-label">{text.ocStatTeams}</span><span className="org-stat-value org-mono">{String(teamCount)}</span></span>
                </div>
              </div>
              <div className="org-root-spine" aria-hidden="true" />
              <div className="org-columns" role="group" aria-label={text.tree}>
                {columns.map((column) => {
                  const open = openColumns.includes(column.company);
                  const editableSites = edit && column.company === siteOwner;
                  return (
                    <div key={column.company} className="org-column">
                      <div className="org-entity-card-row">
                        <button
                          type="button"
                          className={open ? "org-entity-card org-entity-card--open" : "org-entity-card"}
                          aria-expanded={open}
                          title={open ? text.collapseAll : text.expandAll}
                          onClick={() => {
                            setOpenColumns((current) =>
                              current.includes(column.company)
                                ? current.filter((name) => name !== column.company)
                                : [...current, column.company],
                            );
                          }}
                        >
                          <span className="org-entity-name">{column.company}</span>
                          <span className="org-entity-meta org-mono">{String(column.active)}</span>
                        </button>
                        <button
                          type="button"
                          className="org-info-button"
                          aria-label={`${column.company} · ${text.entityInfo}`}
                          onClick={() => {
                            setEntityCard(column);
                          }}
                        >
                          i
                        </button>
                      </div>
                      <div className="org-column-body">
                        {column.sites.map((site) => (
                          <div key={site.branch.id} className="org-site-row">
                            {editableSites && !site.pendingNew ? (
                              <>
                                <input
                                  className="org-inline-input"
                                  aria-label={`${text.ocOps.RENAME_BRANCH} · ${site.branch.name}`}
                                  defaultValue={site.branch.name}
                                  onBlur={(event) => {
                                    const value = event.target.value.trim();
                                    if (value && value !== site.branch.name) {
                                      pushOp({ op: "RENAME_BRANCH", branchId: site.branch.id, name: value });
                                    }
                                  }}
                                />
                                {!site.pendingOff && (
                                  <button
                                    type="button"
                                    className="org-remove-button"
                                    aria-label={`${text.removeSite} · ${site.branch.name}`}
                                    onClick={() => {
                                      pushOp({ op: "DEACTIVATE_BRANCH", branchId: site.branch.id });
                                    }}
                                  >
                                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M18 6 6 18 M6 6l12 12" /></svg>
                                  </button>
                                )}
                              </>
                            ) : (
                              <button
                                type="button"
                                className="org-site-button"
                                onClick={() => {
                                  setSiteCard(site);
                                }}
                              >
                                <span className="org-site-name">{site.branch.name}</span>
                                {site.regionName && <span className="org-chip">{site.regionName}</span>}
                                {(site.pendingOff ?? !!site.branch.deactivated_at) && (
                                  <span className="org-chip org-chip--danger">{text.siteDeactivated}</span>
                                )}
                                {site.pendingNew && <span className="org-chip org-chip--warn">{text.ocOps.CREATE_BRANCH}</span>}
                              </button>
                            )}
                          </div>
                        ))}
                        {editableSites && (
                          addSiteOpen ? (
                            <div className="org-add-site-form">
                              {regions.length > 0 ? (
                                <>
                                  <select
                                    aria-label={text.addSiteRegion}
                                    value={addSiteRegion}
                                    onChange={(event) => {
                                      setAddSiteRegion(event.target.value);
                                    }}
                                  >
                                    <option value="">{text.addSiteRegion}</option>
                                    {regions.map((region) => (
                                      <option key={region.id} value={region.id}>{region.name}</option>
                                    ))}
                                  </select>
                                  <input
                                    aria-label={text.addSiteName}
                                    value={addSiteName}
                                    onChange={(event) => {
                                      setAddSiteName(event.target.value);
                                    }}
                                  />
                                  <button
                                    type="button"
                                    disabled={!addSiteRegion || !addSiteName.trim()}
                                    onClick={() => {
                                      pushOp({ op: "CREATE_BRANCH", regionId: addSiteRegion, name: addSiteName.trim() });
                                      setAddSiteName("");
                                      setAddSiteOpen(false);
                                    }}
                                  >
                                    {text.addSiteConfirm}
                                  </button>
                                </>
                              ) : (
                                <>
                                  <input
                                    aria-label={text.addRegionName}
                                    value={addSiteName}
                                    onChange={(event) => {
                                      setAddSiteName(event.target.value);
                                    }}
                                  />
                                  <button
                                    type="button"
                                    disabled={!addSiteName.trim()}
                                    onClick={() => {
                                      pushOp({ op: "CREATE_REGION", name: addSiteName.trim() });
                                      setAddSiteName("");
                                      setAddSiteOpen(false);
                                    }}
                                  >
                                    {text.addSiteConfirm}
                                  </button>
                                </>
                              )}
                              <button
                                type="button"
                                onClick={() => {
                                  setAddSiteOpen(false);
                                }}
                              >
                                {text.addSiteCancel}
                              </button>
                            </div>
                          ) : (
                            <button
                              type="button"
                              className="org-add-button"
                              onClick={() => {
                                setAddSiteOpen(true);
                              }}
                            >
                              {text.addSite}
                            </button>
                          )
                        )}
                        {!open && column.units.length > 0 && (
                          <button
                            type="button"
                            className="org-units-toggle"
                            title={text.expandTeams}
                            onClick={() => {
                              setOpenColumns((current) => [...current, column.company]);
                            }}
                          >
                            <span className="org-chip">{`+${String(column.units.length)}`}</span>
                            <span>{text.ocStatTeams}</span>
                          </button>
                        )}
                        {open && column.units.map((unit) => (
                          <div key={unit.name} className="org-team-row">
                            {edit ? (
                              <input
                                className="org-inline-input"
                                aria-label={`${text.renameTeam} · ${unit.name}`}
                                title={text.renameTeam}
                                defaultValue={unit.name}
                                onBlur={(event) => {
                                  const value = event.target.value.trim();
                                  if (value && value !== unit.name) {
                                    pushOp({
                                      op: "REASSIGN_ORG_UNIT",
                                      fromOrgUnit: unit.name,
                                      toOrgUnit: value,
                                      scope: { company: column.company },
                                    });
                                  }
                                }}
                              />
                            ) : (
                              <button
                                type="button"
                                className="org-team-button"
                                title={text.teamInfo}
                                onClick={() => {
                                  setTeamCard({ column, unit });
                                }}
                              >
                                <span className="org-team-name">{unit.name}</span>
                                <span className="org-mono">{String(unit.total)}</span>
                              </button>
                            )}
                            {edit && (
                              <>
                                <span className="org-mono">{String(unit.total)}</span>
                                <button
                                  type="button"
                                  className="org-remove-button"
                                  aria-label={`${text.teamRemoveBlocked} · ${unit.name}`}
                                  onClick={() => {
                                    setGuard(`${text.teamRemoveBlocked} · ${unit.name} ${String(unit.total)}${text.peopleUnit}`);
                                  }}
                                >
                                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M18 6 6 18 M6 6l12 12" /></svg>
                                </button>
                              </>
                            )}
                          </div>
                        ))}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        )
      )}

      {entityCard && (
        <OrgEntityCard
          column={entityCard}
          onClose={() => {
            setEntityCard(undefined);
          }}
          onReorg={capabilities.canDraft
            ? () => {
              openDraftModal(entityCard);
            }
            : undefined}
          onNavigate={onNavigate}
        />
      )}
      {siteCard && (
        <OrgSiteCard
          site={siteCard}
          onClose={() => {
            setSiteCard(undefined);
          }}
        />
      )}
      {teamCard && (
        <OrgTeamCard
          column={teamCard.column}
          unit={teamCard.unit}
          onClose={() => {
            setTeamCard(undefined);
          }}
          onNavigate={onNavigate}
        />
      )}
      {modal && (
        <OrgChangeModal
          api={orgApi}
          capabilities={capabilities}
          mode={modal}
          onClose={() => {
            setModal(undefined);
          }}
          onChanged={onModalChanged}
          onNavigate={onNavigate}
        />
      )}
    </main>
  );
}
