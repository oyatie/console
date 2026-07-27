import { useEffect, useRef } from "react";

import { orgStrings as text } from "../../i18n/org";
import type { OrgTreeColumn, OrgTreeSite } from "./orgTree";
import type { HrOrgChartUnit } from "./orgApi";
import { deriveUnitHead } from "./orgTree";

function statusChipLabel(status: string): string {
  const key = status.toUpperCase();
  if (key in text.statusChip) return text.statusChip[key as keyof typeof text.statusChip];
  return text.statusChip.unknown;
}

function CardShell({ label, onClose, children }: {
  label: string;
  onClose: () => void;
  children: React.ReactNode;
}) {
  const dialog = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    dialog.current?.focus();
  }, []);
  return (
    <div className="org-overlay" onClick={onClose}>
      <div
        ref={dialog}
        className="org-card-modal"
        role="dialog"
        aria-modal="true"
        aria-label={label}
        tabIndex={-1}
        onClick={(event) => {
          event.stopPropagation();
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") onClose();
        }}
      >
        {children}
      </div>
    </div>
  );
}

export function OrgEntityCard({ column, onClose, onReorg, onNavigate }: {
  column: OrgTreeColumn;
  onClose: () => void;
  /** Absent = viewer lacks org_change_draft (deny-by-omission). */
  onReorg?: () => void;
  onNavigate: (screen: string) => void;
}) {
  return (
    <CardShell label={text.entityInfo} onClose={onClose}>
      <header className="org-card-head">
        <span className="org-card-avatar" aria-hidden="true">{column.company.slice(0, 1)}</span>
        <div className="org-card-head-main">
          <span className="org-card-name">{column.company}</span>
          {column.entity && <span className="org-mono-chip">{column.entity.slug}</span>}
        </div>
        {column.entity && <span className="org-chip">{statusChipLabel(column.entity.status)}</span>}
        <button type="button" className="org-icon-button" aria-label={text.close} onClick={onClose}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18 M6 6l12 12" /></svg>
        </button>
      </header>
      <dl className="org-card-grid">
        <div>
          <dt>{text.entityHeadcount}</dt>
          <dd className="org-mono">{String(column.active)}</dd>
        </div>
        <div>
          <dt>{text.entityOrg}</dt>
          <dd>
            {`${String(column.sites.length)}${text.entityOrgSites} · ${String(column.units.length)}${text.entityOrgTeams}`}
          </dd>
        </div>
      </dl>
      <div className="org-card-actions">
        {onReorg && (
          <button type="button" className="org-secondary" onClick={onReorg}>{text.entityReorg}</button>
        )}
        <button
          type="button"
          className="org-primary"
          onClick={() => {
            onNavigate("people");
          }}
        >
          {text.entityRoster}
        </button>
      </div>
    </CardShell>
  );
}

export function OrgSiteCard({ site, onClose }: {
  site: OrgTreeSite;
  onClose: () => void;
}) {
  return (
    <CardShell label={site.branch.name} onClose={onClose}>
      <header className="org-card-head">
        <span className="org-card-avatar org-card-avatar--team" aria-hidden="true">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 21h18 M5 21V7l7-4 7 4v14 M9 21v-6h6v6" /></svg>
        </span>
        <div className="org-card-head-main">
          <span className="org-card-name">{site.branch.name}</span>
          {site.regionName && <span className="org-card-path">{site.regionName}</span>}
        </div>
        <span className={site.branch.deactivated_at || site.pendingOff ? "org-chip org-chip--danger" : "org-chip org-chip--ok"}>
          {site.branch.deactivated_at || site.pendingOff ? text.siteDeactivated : text.siteActive}
        </span>
        <button type="button" className="org-icon-button" aria-label={text.close} onClick={onClose}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18 M6 6l12 12" /></svg>
        </button>
      </header>
      <dl className="org-card-grid">
        <div>
          <dt>{text.siteRegion}</dt>
          <dd>{site.regionName || "-"}</dd>
        </div>
      </dl>
    </CardShell>
  );
}

export function OrgTeamCard({ column, unit, onClose, onNavigate }: {
  column: OrgTreeColumn;
  unit: HrOrgChartUnit;
  onClose: () => void;
  onNavigate: (screen: string) => void;
}) {
  const head = deriveUnitHead(unit);
  return (
    <CardShell label={text.teamInfo} onClose={onClose}>
      <header className="org-card-head">
        <span className="org-card-avatar org-card-avatar--team" aria-hidden="true">
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2 M5 7a4 4 0 1 0 8 0a4 4 0 1 0 -8 0 M22 21v-2a4 4 0 0 0-3-3.87 M16 3.13a4 4 0 0 1 0 7.75" /></svg>
        </span>
        <div className="org-card-head-main">
          <span className="org-card-name">{unit.name}</span>
          <span className="org-card-path">{column.company}</span>
        </div>
        <button type="button" className="org-icon-button" aria-label={text.close} onClick={onClose}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18 M6 6l12 12" /></svg>
        </button>
      </header>
      <dl className="org-card-grid">
        <div>
          <dt>{text.teamHead}</dt>
          <dd>
            {head ? (
              <button
                type="button"
                className="org-link-button"
                title={text.personCard}
                onClick={() => {
                  onNavigate("people");
                }}
              >
                {head.name}
                <span className="org-chip org-chip--info">{text.headDerived}</span>
              </button>
            ) : (
              <span role="status">{text.headUnassigned}</span>
            )}
          </dd>
        </div>
        <div>
          <dt>{text.teamHeadcount}</dt>
          <dd className="org-mono">{String(unit.total)}</dd>
        </div>
      </dl>
      <div className="org-card-positions">
        <span className="org-section-label">{text.teamPositions}</span>
        {unit.positions.map((position) => (
          <div key={position.title} className="org-position-row">
            <span className="org-chip">{position.title}</span>
            <span className="org-mono">{String(position.total)}</span>
            <span className="org-position-names">
              {position.employees.map((employee) => (
                <button
                  key={employee.id}
                  type="button"
                  className="org-link-button"
                  title={text.personCard}
                  onClick={() => {
                    onNavigate("people");
                  }}
                >
                  {employee.name}
                </button>
              ))}
            </span>
          </div>
        ))}
      </div>
      <div className="org-card-actions">
        <button
          type="button"
          className="org-secondary"
          onClick={() => {
            onNavigate("people");
          }}
        >
          {text.teamRoster}
        </button>
        <button
          type="button"
          className="org-primary"
          onClick={() => {
            onNavigate("messenger");
          }}
        >
          {text.teamChannel}
        </button>
      </div>
    </CardShell>
  );
}
