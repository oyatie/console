import { useCallback, useEffect, useId, useRef, useState, type SyntheticEvent } from "react";

import type { components } from "@maintenance/api-client-ts";

import type {
  AssetLifecycleCostSummary,
  CreateEquipmentRequest,
  EquipmentListItem,
  EquipmentTimelineGraph,
  SubstituteAssignment,
  SubstituteCandidate,
} from "../../api/types";
import type { ConsoleApiClient } from "../../api/client";
import { EquipmentImportPanel } from "../../features/equipment/EquipmentImportPanel";
import { assetStrings as text } from "../../i18n/asset";
import { SiteGeographyPanel } from "../../features/equipment/SiteGeographyPanel";
import { formatKoreanDateTime } from "../../lib/datetime";
import { Won } from "../../lib/format";
import "./assetWorkspace.css";

type Props = {
  api: ConsoleApiClient;
  sessionKey: string;
  capabilities: {
    canRead: boolean;
    canManage: (branch: string) => boolean;
    canReadCost: (branch: string) => boolean;
    canImport: (branch: string) => boolean;
  };
};

type EquipmentVersion = components["schemas"]["EquipmentVersion"];
type OwnershipTransfer = components["schemas"]["OwnershipTransfer"];

type Detail = {
  row: EquipmentListItem;
  timeline?: EquipmentTimelineGraph;
  versions: EquipmentVersion[];
  candidates: SubstituteCandidate[];
  transfers: OwnershipTransfer[];
  cost?: AssetLifecycleCostSummary;
};

type SessionSubstitution = {
  assignment: SubstituteAssignment;
  sourceEquipmentId: string;
  sessionKey: string;
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : text.genericRequestFailure;
}

function responseData<T>(response: { data?: T; error?: unknown }, operation: string): T {
  if (response.data) return response.data;
  if (response.error && typeof response.error === "object" && "message" in response.error) {
    const message = (response.error as { message?: unknown }).message;
    if (typeof message === "string") throw new Error(message);
  }
  throw new Error(text.responseUnavailable(operation));
}

function value(data: FormData, name: string): string {
  const raw = data.get(name);
  return typeof raw === "string" ? raw.trim() : "";
}

export function AssetWorkspace({ api, sessionKey, capabilities }: Props) {
  const [rows, setRows] = useState<EquipmentListItem[]>([]);
  const [selected, setSelected] = useState<Detail>();
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [notice, setNotice] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [substitution, setSubstitution] = useState<SessionSubstitution>();
  const [showRegister, setShowRegister] = useState(false);
  const [showImport, setShowImport] = useState(false);
  const [showSites, setShowSites] = useState(false);
  const listGeneration = useRef(0);
  const detailGeneration = useRef(0);
  const listAbort = useRef(new AbortController());
  const detailAbort = useRef(new AbortController());
  const retryListQuery = useRef<string | undefined>(undefined);
  const retryDetailRow = useRef<EquipmentListItem | undefined>(undefined);

  const load = useCallback(async (q = "") => {
    listGeneration.current += 1;
    listAbort.current.abort();
    const controller = new AbortController();
    listAbort.current = controller;
    const generation = listGeneration.current;
    setLoading(true);
    setError(undefined);
    try {
      const response = await api.GET("/api/v1/equipment/list", {
        params: { query: { q: q || undefined, limit: 50, offset: 0 } }, signal: controller.signal,
      });
      const page = responseData(response, text.operations.equipmentList);
      if (controller.signal.aborted || generation !== listGeneration.current) return;
      setRows(page.items);
    } catch (cause) {
      if (!controller.signal.aborted && generation === listGeneration.current) {
        retryListQuery.current = q;
        setError(errorMessage(cause));
      }
    } finally {
      if (generation === listGeneration.current) setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    void Promise.resolve().then(() => {
      setRows([]); setSelected(undefined); setSubstitution(undefined); setError(undefined); setNotice(undefined);
      void load("");
    });
    return () => { listAbort.current.abort(); detailAbort.current.abort(); };
  }, [api, load, sessionKey]);

  async function open(row: EquipmentListItem) {
    detailGeneration.current += 1;
    detailAbort.current.abort();
    const controller = new AbortController();
    detailAbort.current = controller;
    const generation = detailGeneration.current;
    setSubstitution(undefined);
    setSelected(undefined);
    setBusy(true);
    setError(undefined);
    try {
      const [timeline, versions, candidates, transfers, cost] = await Promise.all([
        api.GET("/api/v1/equipment/{id}/timeline-graph", { params: { path: { id: row.equipment_id } }, signal: controller.signal }),
        api.GET("/api/v1/equipment/{id}/versions", { params: { path: { id: row.equipment_id } }, signal: controller.signal }),
        capabilities.canManage(row.branch_id)
          ? api.GET("/api/v1/equipment/{id}/substitutes", { params: { path: { id: row.equipment_id } }, signal: controller.signal })
          : Promise.resolve({ data: undefined }),
        capabilities.canManage(row.branch_id)
          ? api.GET("/api/v1/equipment/{id}/ownership-transfer-requests", { params: { path: { id: row.equipment_id } }, signal: controller.signal })
          : Promise.resolve({ data: undefined }),
        capabilities.canReadCost(row.branch_id)
          ? api.GET("/api/v1/financial/equipment/{equipmentId}/lifecycle-cost", { params: { path: { equipmentId: row.equipment_id } }, signal: controller.signal })
          : Promise.resolve({ data: undefined }),
      ]);
      if (controller.signal.aborted || generation !== detailGeneration.current) return;
      const manage = capabilities.canManage(row.branch_id);
      const readCost = capabilities.canReadCost(row.branch_id);
      setSelected({
        row,
        timeline: responseData(timeline, text.operations.lifecycle),
        versions: responseData(versions, text.operations.versionHistory).items,
        candidates: manage ? responseData(candidates, text.operations.substituteCandidates).items : [],
        transfers: manage ? responseData(transfers, text.operations.ownershipTransferHistory).items : [],
        cost: readCost ? responseData(cost, text.operations.lifecycleCost) : undefined,
      });
    } catch (cause) {
      if (!controller.signal.aborted && generation === detailGeneration.current) {
        retryDetailRow.current = row;
        setError(errorMessage(cause));
      }
    } finally {
      if (generation === detailGeneration.current) setBusy(false);
    }
  }

  async function refreshSelected() {
    if (selected) await open(selected.row);
    await load(query);
  }

  async function register(event: SyntheticEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    const body: CreateEquipmentRequest = {
      equipment_no: value(data, "equipment_no"),
      customer_name: value(data, "customer_name"),
      site_name: value(data, "site_name"),
      status: value(data, "status") as CreateEquipmentRequest["status"],
      specification: value(data, "specification"),
      ton_text: value(data, "ton_text"),
      management_no: value(data, "management_no") || null,
    };
    setBusy(true);
    setError(undefined);
    try {
      const response = await api.POST("/api/v1/equipment", { body });
      responseData(response, text.operations.equipmentRegistration);
      form.reset();
      setShowRegister(false);
      setNotice(text.notices.equipmentRegistered);
      await load();
    } catch (cause) {
      setError(errorMessage(cause));
    } finally { setBusy(false); }
  }

  async function rollback(version: number) {
    if (!selected) return;
    setBusy(true); setError(undefined);
    try {
      const response = await api.POST("/api/v1/equipment/{id}/versions/{version}/rollback", {
        params: { path: { id: selected.row.equipment_id, version } },
      });
      const result = responseData(response, text.operations.rollback);
      setNotice(text.notices.rollbackRecorded(result.version));
      await refreshSelected();
    } catch (cause) { setError(errorMessage(cause)); } finally { setBusy(false); }
  }

  async function assign(candidate: SubstituteCandidate) {
    if (!selected) return;
    const location = window.prompt(text.prompts.substitutionLocation);
    if (!location?.trim()) return;
    setBusy(true); setError(undefined);
    try {
      const response = await api.POST("/api/v1/equipment-substitutions", {
        body: { source_equipment_id: selected.row.equipment_id, substitute_equipment_id: candidate.equipment_id, assignment_location: location.trim() },
      });
      const assignment = responseData(response, text.operations.substitutionAssignment);
      setSubstitution({ assignment, sourceEquipmentId: selected.row.equipment_id, sessionKey });
      setNotice(text.notices.substitutionAssigned);
      await load(query);
    } catch (cause) { setError(errorMessage(cause)); } finally { setBusy(false); }
  }

  async function returnSubstitution() {
    if (!substitution || !selected || substitution.sourceEquipmentId !== selected.row.equipment_id || substitution.sessionKey !== sessionKey) return;
    setBusy(true); setError(undefined);
    try {
      const response = await api.POST("/api/v1/equipment-substitutions/{id}/return", { params: { path: { id: substitution.assignment.id } }, body: {} });
      responseData(response, text.operations.substitutionReturn);
      setSubstitution(undefined);
      setNotice(text.notices.substitutionReturned);
      await refreshSelected();
    } catch (cause) { setError(errorMessage(cause)); } finally { setBusy(false); }
  }

  async function createTransfer(event: SyntheticEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected) return;
    const data = new FormData(event.currentTarget);
    setBusy(true); setError(undefined);
    try {
      const response = await api.POST("/api/v1/equipment/{id}/ownership-transfer-requests", {
        params: { path: { id: selected.row.equipment_id } },
        body: { to_owner: value(data, "to_owner"), reason: value(data, "reason") },
      });
      responseData(response, text.operations.ownershipTransferRequest);
      setNotice(text.notices.ownershipTransferRequested);
      await refreshSelected();
    } catch (cause) { setError(errorMessage(cause)); } finally { setBusy(false); }
  }

  async function decideTransfer(id: string, decision: "approve" | "reject") {
    const comment = window.prompt(decision === "approve" ? text.prompts.approveTransferComment : text.prompts.rejectTransferComment);
    if (comment === null) return;
    setBusy(true); setError(undefined);
    try {
      const response = await api.POST("/api/v1/equipment/ownership-transfer-requests/{id}/decisions", { params: { path: { id } }, body: { decision, comment } });
      responseData(response, text.operations.ownershipTransferDecision);
      setNotice(decision === "approve" ? text.notices.ownershipTransferApproved : text.notices.ownershipTransferRejected);
      await refreshSelected();
    } catch (cause) { setError(errorMessage(cause)); } finally { setBusy(false); }
  }

  return <main className="asset-workspace" aria-busy={loading || busy}>
    <div className="asset-workspace__content">
      <header className="asset-workspace__header">
        <div><h1>{text.title}</h1><p>{text.subtitle}</p></div>
        {selected && capabilities.canManage(selected.row.branch_id) ? <div className="asset-workspace__actions">
          <button type="button" className="asset-workspace__button asset-workspace__button--secondary" onClick={() => { setShowSites((open) => !open); }}>{text.siteCoordinates}</button>
          {capabilities.canImport(selected.row.branch_id) ? <button type="button" className="asset-workspace__button asset-workspace__button--secondary" onClick={() => { setShowImport((open) => !open); }}>{text.importSpreadsheet}</button> : null}
          <button type="button" className="asset-workspace__button" onClick={() => { setShowRegister((open) => !open); }}>{text.registerEquipment}</button>
        </div> : null}
      </header>
      {notice ? <p role="status" className="asset-workspace__notice">{notice}</p> : null}
      {error ? <div role="alert" className="asset-workspace__alert"><span>{error}</span><button type="button" className="asset-workspace__button asset-workspace__button--secondary asset-workspace__button--compact" onClick={() => { const row = retryDetailRow.current; if (row !== undefined) { void open(row); } else { void load(retryListQuery.current || ""); } }}>{text.retry}</button></div> : null}
      {showRegister ? <RegisterForm busy={busy} onSubmit={(event) => { void register(event); }} /> : null}
      {showImport ? <EquipmentImportPanel api={api} onImported={() => { void load(); }} /> : null}
      {showSites ? <SiteGeographyPanel api={api} /> : null}
      <section className="asset-workspace__grid">
        <section className="asset-workspace__panel asset-workspace__list-panel">
          <form className="asset-workspace__search" onSubmit={(event) => { event.preventDefault(); void load(query); }}>
            <label className="asset-workspace__sr-only" htmlFor="asset-search">{text.searchLabel}</label>
            <input id="asset-search" value={query} onChange={(event) => { setQuery(event.target.value); }} placeholder={text.searchPlaceholder} />
            <button type="submit" className="asset-workspace__button asset-workspace__button--secondary">{text.search}</button>
          </form>
          <div className="asset-workspace__table-wrap"><table><thead><tr><th>{text.columns.equipmentNumber}</th><th>{text.columns.status}</th><th>{text.columns.customerSite}</th><th>{text.columns.updated}</th></tr></thead><tbody>
            {rows.map((row) => <tr key={row.equipment_id}><td><button type="button" className="asset-workspace__equipment-link" onClick={() => void open(row)} aria-label={text.detailOpenAriaLabel(row.equipment_no)}>{row.equipment_no}</button><div className="asset-workspace__meta">{row.management_no ?? row.model ?? "—"}</div></td><td>{row.status}</td><td>{row.customer_name}<div className="asset-workspace__meta">{row.site_name}</div></td><td className="asset-workspace__updated">{formatKoreanDateTime(row.updated_at)}</td></tr>)}
          </tbody></table></div>
          {!loading && rows.length === 0 ? <p role="status" className="asset-workspace__empty">{text.noMatchingEquipment}</p> : null}
        </section>
        {selected ? <AssetDetail detail={selected} capabilities={capabilities} busy={busy} substitution={substitution} sessionKey={sessionKey} onRollback={rollback} onAssign={assign} onReturn={returnSubstitution} onTransfer={createTransfer} onDecide={decideTransfer} /> : <section className="asset-workspace__panel"><p role="status" className="asset-workspace__empty">{text.selectEquipment}</p></section>}
      </section>
    </div>
  </main>;
}

function RegisterForm({ busy, onSubmit }: { busy: boolean; onSubmit: (event: SyntheticEvent<HTMLFormElement>) => void }) {
  const id = useId();
  return <section className="asset-workspace__panel"><form className="asset-workspace__form asset-workspace__form--two-column" onSubmit={onSubmit}>
    <h2>{text.registerEquipment}</h2>
    <Field id={`${id}-no`} label={text.fields.equipmentNumber} name="equipment_no" required /><Field id={`${id}-management`} label={text.fields.managementNumber} name="management_no" />
    <Field id={`${id}-customer`} label={text.fields.customer} name="customer_name" required /><Field id={`${id}-site`} label={text.fields.site} name="site_name" required />
    <Field id={`${id}-spec`} label={text.fields.specification} name="specification" required /><Field id={`${id}-ton`} label={text.fields.capacity} name="ton_text" required />
    <label className="asset-workspace__field" htmlFor={`${id}-status`}>{text.fields.status}<select id={`${id}-status`} name="status" defaultValue={text.statusOptions[0]}>{text.statusOptions.map((status) => <option key={status} value={status}>{status}</option>)}</select></label>
    <div className="asset-workspace__form-action"><button type="submit" className="asset-workspace__button" disabled={busy}>{text.register}</button></div>
  </form></section>;
}

function Field({ id, label, name, required = false }: { id: string; label: string; name: string; required?: boolean }) { return <label className="asset-workspace__field" htmlFor={id}>{label}<input id={id} name={name} required={required} /></label>; }

function AssetDetail({ detail, capabilities, busy, substitution, sessionKey, onRollback, onAssign, onReturn, onTransfer, onDecide }: { detail: Detail; capabilities: Props["capabilities"]; busy: boolean; substitution?: SessionSubstitution; sessionKey: string; onRollback: (version: number) => Promise<void>; onAssign: (candidate: SubstituteCandidate) => Promise<void>; onReturn: () => Promise<void>; onTransfer: (event: SyntheticEvent<HTMLFormElement>) => Promise<void>; onDecide: (id: string, decision: "approve" | "reject") => Promise<void> }) {
  const { row, timeline, versions, candidates, transfers, cost } = detail;
  return <div className="asset-workspace__detail"><section className="asset-workspace__panel"><header><h2>{row.equipment_no}</h2><p>{row.customer_name} · {row.site_name}</p></header><dl className="asset-workspace__facts"><div><dt>{text.fields.status}</dt><dd>{row.status}</dd></div><div><dt>{text.fields.owner}</dt><dd>{row.asset_owner ?? "—"}</dd></div><div><dt>{text.fields.specification}</dt><dd>{row.specification} / {row.ton_text}</dd></div><div><dt>{text.fields.vin}</dt><dd className="asset-workspace__monospace">{row.vin ?? "—"}</dd></div></dl></section>
    <section className="asset-workspace__panel"><h3>{text.operations.lifecycle}</h3>{timeline ? <><ol className="asset-workspace__timeline">{timeline.lifecycle_events.map((event) => <li key={event.id}><div>{event.href ? <a href={event.href}>{event.label}</a> : event.label}</div><div className="asset-workspace__meta">{event.description ?? event.event_date ?? event.occurred_at ?? ""}</div></li>)}</ol><div className="asset-workspace__graph-nodes">{timeline.graph.nodes.map((node) => node.href ? <a key={node.id} href={node.href}>{node.label}</a> : <span key={node.id}>{node.label}</span>)}</div>{timeline.graph.edges.length ? <p className="asset-workspace__meta">{timeline.graph.edges.map((edge) => edge.label).join(" · ")}</p> : null}</> : <p className="asset-workspace__empty">{text.lifecycleUnavailable}</p>}</section>
    <section className="asset-workspace__panel"><h3>{text.operations.versionHistory}</h3>{versions.length ? <ul className="asset-workspace__stack-list">{versions.map((version) => <li key={version.version}><span>{text.versionSummary(version.version, version.status, formatKoreanDateTime(version.createdAt))}</span>{capabilities.canManage(row.branch_id) ? <button type="button" className="asset-workspace__button asset-workspace__button--secondary asset-workspace__button--compact" disabled={busy} onClick={() => { void onRollback(version.version); }}>{text.rollback}</button> : null}</li>)}</ul> : <p className="asset-workspace__empty">{text.noSavedVersions}</p>}</section>
    {capabilities.canReadCost(row.branch_id) && cost ? <section className="asset-workspace__panel"><h3>{text.lifecycleCost}</h3><dl className="asset-workspace__facts"><Money label={text.costLabels.totalCostOfOwnership} value={cost.tco_won} /><Money label={text.costLabels.maintenanceCost} value={cost.maintenance_total_won} /><Money label={text.costLabels.residualValue} value={cost.residual_value_won} /><Money label={text.costLabels.monthlyCost} value={cost.cost_per_month_won} /></dl>{cost.timeline.map((entry) => <div key={entry.id} className="asset-workspace__cost-entry"><span>{entry.memo || entry.source}</span><Won amount={entry.amount_won} /></div>)}</section> : null}
    {capabilities.canManage(row.branch_id) ? <section className="asset-workspace__panel"><h3>{text.substitution}</h3><p className="asset-workspace__meta">{text.substitutionReturnLimitation}</p>{substitution && substitution.sourceEquipmentId === row.equipment_id && substitution.sessionKey === sessionKey ? <div className="asset-workspace__split-row"><span>{text.substitutionAssigned(substitution.assignment.assignment_location)}</span><button type="button" className="asset-workspace__button asset-workspace__button--secondary asset-workspace__button--compact" disabled={busy} onClick={() => { void onReturn(); }}>{text.returnSubstitution}</button></div> : candidates.length ? <ul className="asset-workspace__stack-list">{candidates.map((candidate) => <li key={candidate.equipment_id}><span>{candidate.equipment_no} · {candidate.ton_text} · {candidate.match_kind}</span><button type="button" className="asset-workspace__button asset-workspace__button--secondary asset-workspace__button--compact" disabled={busy} onClick={() => { void onAssign(candidate); }}>{text.assignSubstitution}</button></li>)}</ul> : <p className="asset-workspace__empty">{text.noCompatibleSubstitutes}</p>}</section> : null}
    {capabilities.canManage(row.branch_id) ? <section className="asset-workspace__panel"><h3>{text.ownershipTransfer}</h3><form className="asset-workspace__form" onSubmit={(event) => { void onTransfer(event); }}><Field id={`owner-${row.equipment_id}`} label={text.fields.newLegalOwner} name="to_owner" required /><label className="asset-workspace__field">{text.fields.transferReason}<textarea name="reason" required /></label><button type="submit" className="asset-workspace__button" disabled={busy}>{text.requestTransfer}</button></form>{transfers.map((transfer) => <div key={transfer.id} className="asset-workspace__transfer"><span>{transfer.from_owner} → {transfer.to_owner} · {transfer.status}</span><span className="asset-workspace__meta">{transfer.current_step ?? text.completed}</span>{transfer.status === "PENDING" ? <div className="asset-workspace__actions"><button type="button" className="asset-workspace__button asset-workspace__button--compact" disabled={busy} onClick={() => { void onDecide(transfer.id, "approve"); }}>{text.approve}</button><button type="button" className="asset-workspace__button asset-workspace__button--secondary asset-workspace__button--compact" disabled={busy} onClick={() => { void onDecide(transfer.id, "reject"); }}>{text.reject}</button></div> : null}</div>)}</section> : null}
  </div>;
}

function Money({ label, value }: { label: string; value: number | null | undefined }) { return <div><dt>{label}</dt><dd><Won amount={value ?? 0} /></dd></div>; }
