import { useCallback, useEffect, useId, useRef, useState } from "react";

import { orgStrings as text } from "../../i18n/org";
import type {
  OrgApi,
  OrgChangeDetail,
  OrgChangeKind,
  OrgChangeStatus,
  OrgChangeTarget,
  OrgProposalOp,
} from "./orgApi";
import type { OrgCapabilities } from "./orgCapabilities";

export interface OrgChangeDraftSeed {
  kind: OrgChangeKind;
  target: OrgChangeTarget;
  proposal: OrgProposalOp[];
}

export type OrgChangeModalMode =
  | { kind: "existing"; id: string }
  | { kind: "new"; seed: OrgChangeDraftSeed };

type Props = {
  api: OrgApi;
  capabilities: OrgCapabilities;
  mode: OrgChangeModalMode;
  onClose: () => void;
  /** Fired with every fresh detail so the owner can refresh its list state. */
  onChanged: (detail: OrgChangeDetail) => void;
  onNavigate: (screen: string) => void;
};

const STEP_KEYS_BASE = ["draft", "precheck", "approval", "effect"] as const;
const STEP_KEYS_DISSOLVE = [...STEP_KEYS_BASE, "settle", "archive"] as const;

function stageIndex(status: OrgChangeStatus): number {
  switch (status) {
    case "DRAFT":
      return 0;
    case "PRECHECKED":
      return 1;
    case "IN_APPROVAL":
    case "APPROVED":
    case "REJECTED":
    case "CANCELLED":
      return 2;
    case "APPLIED":
      return 3;
    case "SETTLING":
      return 4;
    case "ARCHIVED":
      return 5;
  }
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : text.actionError;
}

function today(): string {
  return new Date().toISOString().slice(0, 10);
}

function opLabel(op: OrgProposalOp): string {
  const kind = text.ocOps[op.op];
  switch (op.op) {
    case "CREATE_REGION":
    case "RENAME_REGION":
    case "CREATE_BRANCH":
    case "CREATE_SITE":
      return `${kind} · ${op.name}`;
    case "RENAME_BRANCH":
      return op.name ? `${kind} · ${op.name}` : kind;
    case "DEACTIVATE_REGION":
    case "DEACTIVATE_BRANCH":
    case "UPDATE_SITE":
      return kind;
    case "REASSIGN_ORG_UNIT":
      return `${kind} · ${op.fromOrgUnit} → ${op.toOrgUnit}`;
  }
}

export function OrgChangeModal({ api, capabilities, mode, onClose, onChanged, onNavigate }: Props) {
  const [detail, setDetail] = useState<OrgChangeDetail>();
  const [loading, setLoading] = useState(mode.kind === "existing");
  const [loadFailed, setLoadFailed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [kind, setKind] = useState<OrgChangeKind>(mode.kind === "new" ? mode.seed.kind : "REORG");
  const [effectiveDate, setEffectiveDate] = useState(today());
  const [reason, setReason] = useState("");
  const [draftDirty, setDraftDirty] = useState(false);
  const [rejecting, setRejecting] = useState<string>();
  const [rejectMemo, setRejectMemo] = useState("");
  const [cancelling, setCancelling] = useState(false);
  const [cancelReason, setCancelReason] = useState("");
  const dialog = useRef<HTMLDivElement | null>(null);
  const generation = useRef(0);
  const dateId = useId();
  const reasonId = useId();
  const rejectId = useId();
  const cancelId = useId();
  const titleId = useId();

  const adopt = useCallback((next: OrgChangeDetail) => {
    setDetail(next);
    setKind(next.kind);
    setEffectiveDate(next.effectiveDate);
    setReason(next.reason);
    setDraftDirty(false);
    onChanged(next);
  }, [onChanged]);

  useEffect(() => {
    dialog.current?.focus();
  }, []);

  const loadExisting = useCallback((id: string, signal?: AbortSignal) => {
    const token = ++generation.current;
    setLoading(true);
    setLoadFailed(false);
    setError(undefined);
    api.getChange(id, signal)
      .then((next) => {
        if (generation.current !== token) return;
        adopt(next);
        setLoading(false);
      })
      .catch((cause: unknown) => {
        if (generation.current !== token || signal?.aborted) return;
        setError(message(cause));
        setLoadFailed(true);
        setLoading(false);
      });
  }, [adopt, api]);

  useEffect(() => {
    if (mode.kind !== "existing") return;
    const controller = new AbortController();
    const start = window.setTimeout(() => {
      loadExisting(mode.id, controller.signal);
    }, 0);
    return () => {
      window.clearTimeout(start);
      controller.abort();
    };
  }, [loadExisting, mode]);

  const run = useCallback(async (work: () => Promise<OrgChangeDetail>) => {
    const token = ++generation.current;
    setBusy(true);
    setError(undefined);
    try {
      const next = await work();
      if (generation.current === token) adopt(next);
      return true;
    } catch (cause) {
      if (generation.current === token) setError(message(cause));
      return false;
    } finally {
      if (generation.current === token) setBusy(false);
    }
  }, [adopt]);

  const seed = mode.kind === "new" ? mode.seed : undefined;
  const status: OrgChangeStatus | undefined = detail?.status;
  const target = detail?.target ?? seed?.target;
  const proposal = detail?.proposal ?? seed?.proposal ?? [];
  const stepKeys = (detail?.kind ?? kind) === "DISSOLVE" ? STEP_KEYS_DISSOLVE : STEP_KEYS_BASE;
  const stage = status ? stageIndex(status) : 0;
  const editableDraft = !detail || status === "DRAFT" || status === "PRECHECKED";
  const report = detail?.preflight ?? undefined;
  const reportBlocked = !!report && report.blockers.length > 0;
  const reportUsable = !!report && !report.stale && !draftDirty;
  const pendingSteps = detail?.approvalSteps.filter((step) => step.decision === "PENDING") ?? [];
  const nextStep = pendingSteps.length
    ? pendingSteps.reduce((low, step) => (step.stepOrder < low.stepOrder ? step : low))
    : undefined;
  const approvedCount = detail ? detail.approvalSteps.length - pendingSteps.length : 0;
  const unsettled = detail?.settlementItems.filter((item) => !item.done) ?? [];
  const terminal = status === "APPLIED" || status === "ARCHIVED" || status === "REJECTED" || status === "CANCELLED";

  const saveDraftIfDirty = async (): Promise<boolean> => {
    if (!detail || !draftDirty) return true;
    return run(() => api.updateDraft(detail.id, { kind, effectiveDate: effectiveDate, reason }));
  };

  const create = async () => {
    if (!seed || !capabilities.canDraft) return;
    if (!reason.trim()) {
      setError(text.ocReasonRequired);
      return;
    }
    await run(() => api.createChange({
      kind,
      target: seed.target,
      effectiveDate: effectiveDate,
      reason: reason.trim(),
      proposal: seed.proposal,
    }));
  };

  let ctaKind: "create" | "precheck" | "submit" | "waiting" | "effectuate" | "archive" | undefined;
  let ctaLabel = "";
  let ctaDisabled = true;
  if (!detail) {
    ctaKind = "create";
    ctaLabel = text.ocCreate;
    ctaDisabled = busy || !capabilities.canDraft;
  } else if (status === "DRAFT" || (status === "PRECHECKED" && (!reportUsable || reportBlocked))) {
    ctaKind = "precheck";
    ctaLabel = text.ocPrecheckRun;
    ctaDisabled = busy || !capabilities.canDraft;
  } else if (status === "PRECHECKED") {
    ctaKind = "submit";
    ctaLabel = text.ocSubmit;
    ctaDisabled = busy || !capabilities.canDraft || reportBlocked;
  } else if (status === "IN_APPROVAL") {
    ctaKind = "waiting";
    ctaLabel = `${text.ocWaiting} (${String(approvedCount)}/${String(detail.approvalSteps.length)})`;
  } else if (status === "APPROVED") {
    ctaKind = "effectuate";
    ctaLabel = text.ocEffectuate;
    ctaDisabled = busy || !capabilities.canApply;
  } else if (status === "SETTLING") {
    ctaKind = "archive";
    ctaLabel = text.ocArchive;
    ctaDisabled = busy || !capabilities.canApply || unsettled.length > 0;
  }

  const runCta = async () => {
    if (ctaKind === "create") {
      await create();
      return;
    }
    if (!detail) return;
    if (ctaKind === "precheck") {
      if (!(await saveDraftIfDirty())) return;
      await run(async () => {
        await api.preflight(detail.id);
        return api.getChange(detail.id);
      });
    } else if (ctaKind === "submit") {
      if (!(await saveDraftIfDirty())) return;
      await run(() => api.submit(detail.id));
    } else if (ctaKind === "effectuate") {
      await run(() => api.effectuate(detail.id));
    } else if (ctaKind === "archive") {
      await run(() => api.archive(detail.id));
    }
  };

  return (
    <div className="org-overlay" onClick={onClose}>
      <div
        ref={dialog}
        className="org-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        onClick={(event) => {
          event.stopPropagation();
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") onClose();
        }}
      >
        <header className="org-modal-head">
          <span className="org-modal-glyph" aria-hidden="true">
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 3v18h18 M7 16V9 M12 16V5 M17 16v-3" /></svg>
          </span>
          <span id={titleId} className="org-modal-title">
            {text.ocTitle}{target ? ` · ${target.label}` : ""}
          </span>
          {detail && <span className="org-mono-chip">{detail.code}</span>}
          <span className="org-stage-chip">{status ? text.ocStage[status] : text.ocStage.DRAFT}</span>
          <button type="button" className="org-icon-button" aria-label={text.close} onClick={onClose}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18 M6 6l12 12" /></svg>
          </button>
        </header>

        <div className="org-modal-steps">
          {stepKeys.map((key, index) => (
            <span key={key} className="org-modal-step-cell">
              <span className={index === stage ? "org-step-chip org-step-chip--current" : index < stage ? "org-step-chip org-step-chip--done" : "org-step-chip"}>
                {text.ocSteps[key]}
              </span>
              {index < stepKeys.length - 1 && <span className="org-step-sep" aria-hidden="true">{"›"}</span>}
            </span>
          ))}
          {(status === "REJECTED" || status === "CANCELLED") && (
            <span className="org-chip org-chip--danger">{text.ocStage[status]}</span>
          )}
        </div>

        <div className="org-modal-body">
          {error && (
            <div className="org-alert" role="alert">
              <span>{error}</span>
              {loadFailed && mode.kind === "existing" && (
                <button
                  type="button"
                  onClick={() => {
                    loadExisting(mode.id);
                  }}
                >
                  {text.retry}
                </button>
              )}
            </div>
          )}
          {loading && <p role="status">{text.loading}</p>}

          {!loading && !loadFailed && (
            <>
              {editableDraft ? (
                <>
                  <div className="org-field">
                    <span className="org-field-label">{text.ocKind}</span>
                    <div className="org-kind-row" role="radiogroup" aria-label={text.ocKind}>
                      {(Object.keys(text.ocKinds) as OrgChangeKind[]).map((option) => (
                        <button
                          key={option}
                          type="button"
                          role="radio"
                          aria-checked={kind === option}
                          className={kind === option ? "org-kind-option org-kind-option--on" : "org-kind-option"}
                          disabled={busy || !capabilities.canDraft}
                          onClick={() => {
                            setKind(option);
                            setDraftDirty(true);
                          }}
                        >
                          {text.ocKinds[option]}
                        </button>
                      ))}
                    </div>
                  </div>
                  <div className="org-field">
                    <label className="org-field-label" htmlFor={dateId}>{text.ocEffectiveDate}</label>
                    <input
                      id={dateId}
                      className="org-date-input"
                      type="date"
                      value={effectiveDate}
                      min={today()}
                      disabled={busy || !capabilities.canDraft}
                      onChange={(event) => {
                        setEffectiveDate(event.target.value);
                        setDraftDirty(true);
                      }}
                    />
                  </div>
                  <div className="org-field">
                    <label className="org-field-label" htmlFor={reasonId}>{text.ocReason}</label>
                    <textarea
                      id={reasonId}
                      className="org-reason-input"
                      value={reason}
                      maxLength={4000}
                      disabled={busy || !capabilities.canDraft}
                      onChange={(event) => {
                        setReason(event.target.value);
                        setDraftDirty(true);
                      }}
                    />
                  </div>
                </>
              ) : (
                <div className="org-field">
                  <span className="org-field-label">{text.ocReason}</span>
                  <p className="org-reason-text">{detail.reason}</p>
                </div>
              )}

              <div className="org-stat-strip">
                <div className="org-stat"><span className="org-stat-label">{text.ocStatTarget}</span><span className="org-stat-value">{target ? text.ocTargetKind[target.kind] : "-"}</span></div>
                <div className="org-stat"><span className="org-stat-label">{text.ocStatHeadcount}</span><span className="org-stat-value org-mono">{detail ? String(detail.headcount) : "-"}</span></div>
                <div className="org-stat"><span className="org-stat-label">{text.ocStatSites}</span><span className="org-stat-value org-mono">{detail ? String(detail.siteCount) : "-"}</span></div>
                <div className="org-stat"><span className="org-stat-label">{text.ocStatTeams}</span><span className="org-stat-value org-mono">{detail ? String(detail.teamCount) : "-"}</span></div>
              </div>

              <div className="org-section">
                <span className="org-section-label">{text.ocProposal}</span>
                {proposal.length === 0 && <p className="org-empty-line" role="status">{text.ocProposalEmpty}</p>}
                {proposal.map((op, index) => (
                  <div key={`${op.op}-${String(index)}`} className="org-proposal-row">
                    <span className="org-chip">{text.ocOps[op.op]}</span>
                    <span className="org-proposal-text">{opLabel(op)}</span>
                  </div>
                ))}
              </div>

              {report && (
                <div className="org-section">
                  <span className="org-section-label">{text.ocPrecheck}</span>
                  {(report.stale || draftDirty) && <div className="org-banner org-banner--warn">{text.ocPrecheckStale}</div>}
                  {report.blockers.map((blocker) => (
                    <div key={blocker.code} className="org-banner org-banner--danger" role="alert">
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M12 9v4 M12 17h.01 M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" /></svg>
                      <span>{blocker.label}</span>
                      <span className="org-mono-chip">{String(blocker.count)}</span>
                    </div>
                  ))}
                  {report.warnings.map((warning) => (
                    <div key={warning.code} className="org-banner org-banner--warn">{warning.label}</div>
                  ))}
                  <div className="org-report-meta">
                    <span className="org-chip">{text.ocStatHeadcount}<span className="org-mono"> {String(report.headcount)}</span></span>
                    <span className="org-chip">{text.ocDependents}<span className="org-mono"> {String(report.dependentsTotal)}</span></span>
                  </div>
                </div>
              )}

              {detail && detail.approvalSteps.length > 0 && (
                <div className="org-section">
                  <span className="org-section-label">{text.ocApprovals}</span>
                  {detail.approvalSteps.map((step) => (
                    <div key={step.id} className="org-approval-row">
                      <span className="org-chip">{text.ocApprovalRole[step.roleKey]}</span>
                      <span className="org-approval-who">{step.decidedBy ?? ""}</span>
                      {step.decision === "APPROVED" && (
                        <span className="org-decision org-decision--ok">
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                          {text.ocDecisionApproved}
                        </span>
                      )}
                      {step.decision === "REJECTED" && <span className="org-decision org-decision--danger">{text.ocDecisionRejected}</span>}
                      {status === "IN_APPROVAL" && capabilities.canApprove && nextStep?.id === step.id && (
                        <span className="org-approval-actions">
                          <button
                            type="button"
                            disabled={busy}
                            onClick={() => {
                              void run(() => api.decide(detail.id, step.id, { decision: "APPROVED" }));
                            }}
                          >
                            {text.ocApprove}
                          </button>
                          <button
                            type="button"
                            className="org-danger-button"
                            disabled={busy}
                            onClick={() => {
                              setRejecting((current) => (current === step.id ? undefined : step.id));
                              setRejectMemo("");
                            }}
                          >
                            {text.ocRejectAction}
                          </button>
                        </span>
                      )}
                      {rejecting === step.id && (
                        <span className="org-reject-row">
                          <label className="org-field-label" htmlFor={rejectId}>{text.ocRejectMemo}</label>
                          <input
                            id={rejectId}
                            value={rejectMemo}
                            maxLength={500}
                            onChange={(event) => {
                              setRejectMemo(event.target.value);
                            }}
                          />
                          <button
                            type="button"
                            className="org-danger-button"
                            disabled={busy || !rejectMemo.trim()}
                            onClick={() => {
                              void run(() => api.decide(detail.id, step.id, { decision: "REJECTED", memo: rejectMemo.trim() })).then((done) => {
                                if (done) setRejecting(undefined);
                              });
                            }}
                          >
                            {text.ocRejectAction}
                          </button>
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              )}

              {detail && detail.settlementItems.length > 0 && (status === "SETTLING" || status === "ARCHIVED") && (
                <div className="org-section">
                  <span className="org-section-label">{text.ocSettle}</span>
                  {detail.settlementItems.map((item) => (
                    <div key={item.id} className="org-approval-row">
                      <span className="org-settle-text">{text.ocSettleItems[item.itemKey]}</span>
                      {item.done ? (
                        <span className="org-decision org-decision--ok">
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
                          {text.ocSettleDone}
                        </span>
                      ) : (
                        status === "SETTLING" && capabilities.canApply && (
                          <button
                            type="button"
                            disabled={busy}
                            onClick={() => {
                              void run(() => api.completeSettlement(detail.id, item.id));
                            }}
                          >
                            {text.ocSettleAction}
                          </button>
                        )
                      )}
                    </div>
                  ))}
                </div>
              )}

              {detail && detail.events.length > 0 && (
                <div className="org-section">
                  <span className="org-section-label">{text.ocEvents}</span>
                  {detail.events.map((event, index) => (
                    <div key={`${event.at}-${String(index)}`} className="org-event-row">
                      <span className="org-mono org-event-at">{event.at.slice(0, 16).replace("T", " ")}</span>
                      <span className="org-event-action">{event.action}</span>
                      <span className="org-event-reason">{event.reason}</span>
                    </div>
                  ))}
                  <button
                    type="button"
                    className="org-link-button"
                    onClick={() => {
                      onNavigate("audit");
                    }}
                  >
                    {text.ocEventsAudit}
                  </button>
                </div>
              )}
            </>
          )}
        </div>

        <footer className="org-modal-foot">
          {ctaKind && !terminal && (
            <button
              type="button"
              className="org-cta"
              disabled={ctaDisabled}
              onClick={() => {
                void runCta();
              }}
            >
              {ctaLabel}
            </button>
          )}
          {detail && (status === "DRAFT" || status === "PRECHECKED") && capabilities.canDraft && (
            cancelling ? (
              <span className="org-reject-row">
                <label className="org-field-label" htmlFor={cancelId}>{text.ocCancelReason}</label>
                <input
                  id={cancelId}
                  value={cancelReason}
                  maxLength={500}
                  onChange={(event) => {
                    setCancelReason(event.target.value);
                  }}
                />
                <button
                  type="button"
                  className="org-danger-button"
                  disabled={busy || !cancelReason.trim()}
                  onClick={() => {
                    void run(() => api.cancel(detail.id, cancelReason.trim())).then((done) => {
                      if (done) setCancelling(false);
                    });
                  }}
                >
                  {text.ocCancel}
                </button>
              </span>
            ) : (
              <button
                type="button"
                className="org-danger-button"
                disabled={busy}
                onClick={() => {
                  setCancelling(true);
                }}
              >
                {text.ocCancel}
              </button>
            )
          )}
          <span className="org-spacer" />
          <button type="button" className="org-secondary" onClick={onClose}>{text.close}</button>
        </footer>
      </div>
    </div>
  );
}
