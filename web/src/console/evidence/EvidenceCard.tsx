/* eslint-disable react-refresh/only-export-components -- evidenceWindowEntry is the §4.7-3 open-gesture factory, same pattern as objectCardWindowEntry */
// EvidenceCard — the EV- object detail (design (34) / EV-101~103): SHA-256
// fixity + TSA chips, WORM 원본/파생본 split, chain-of-custody timeline in the
// audit-stream shape, admissibility chip, and the legal-hold disposal gate.
// Opens as the right pin (§4.7-3) via evidenceWindowEntry; composes ObjectCard
// as the single object-detail substrate (§4-14).
//
// Real: hold apply / hold release (four-eyes via governance approvals-create +
// decide, fail-closed pending state) and fixity verify (per-copy verdict
// chips). Custody/holds render straight from `detail` — the caller (see
// EvidenceRecords) refetches the full object after every mutation, so the
// timeline is never client-synthesized. Operations without a typed backend
// authority are absent rather than shown as disabled previews.
import { useState, type CSSProperties } from "react";

import { ApiCallError } from "../../api/ontologyActions";
import { ko } from "../../i18n/ko";
import { StatusChip } from "../components";
import { ObjectCard } from "../objectcard";
import { PolicyGated } from "../policy";
import { type WindowEntry } from "../window";
import {
  admissibilityLabel,
  admissibilityTone,
  copyVerdictLabel,
  copyVerdictTone,
  custodyStageLabel,
  custodyStageOfAudit,
  derivativesOf,
  fixityTone,
  formatSize,
  originalOf,
  shortDigest,
  toObjectCardDescriptor,
  tsaTone,
  wormTone,
} from "./evidenceModel";
import {
  EVIDENCE_ACTIONS,
  type CopyFixityStatus,
  type CopyVerdictMap,
  EvidenceDetailRefreshError,
  type EvidenceObjectDetail,
  type ReleaseFlowState,
  type VerifyEvidence,
  type VerifyOutcome,
} from "./types";
import "../tokens.css";

const T = ko.console.evidence;
const TA = ko.console.audit;
const TP = ko.page;

const rootStyle: CSSProperties = {
  display: "grid",
  gap: "var(--sp-5)",
  padding: "var(--sp-5)",
  background: "var(--surface)",
  color: "var(--ink)",
  fontFamily: "var(--font-sans)",
};

const headerStyle: CSSProperties = { display: "grid", gap: "var(--sp-2)" };

const titleRowStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  alignItems: "baseline",
  justifyContent: "space-between",
  gap: "var(--sp-3)",
};

const titleStyle: CSSProperties = {
  margin: 0,
  fontSize: "var(--text-card-title)",
  fontWeight: "var(--fw-strong)",
};

const monoStyle: CSSProperties = {
  fontFamily: "var(--font-mono)",
  fontSize: "var(--text-xs)",
  color: "var(--steel)",
  overflowWrap: "anywhere",
};

const chipRowStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  alignItems: "center",
  gap: "var(--sp-2)",
};

const sectionStyle: CSSProperties = { display: "grid", gap: "var(--sp-2)" };

const sectionHeadingStyle: CSSProperties = {
  margin: 0,
  color: "var(--faint)",
  fontSize: "var(--text-xs)",
  fontWeight: "var(--fw-strong)",
  letterSpacing: "var(--tracking-label)",
};

const copyRowStyle: CSSProperties = {
  display: "grid",
  gap: "var(--sp-1)",
  padding: "var(--sp-3)",
  border: "1px solid var(--border-soft)",
  borderRadius: "var(--radius-md)",
  background: "var(--muted)",
};

const copyMetaStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  alignItems: "center",
  gap: "var(--sp-2)",
  fontSize: "var(--text-sm)",
};

const listStyle: CSSProperties = {
  display: "grid",
  gap: "var(--sp-2)",
  margin: 0,
  padding: 0,
  listStyle: "none",
};

const custodyRowStyle: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  alignItems: "center",
  gap: "var(--sp-2)",
  minHeight: 32,
  fontSize: "var(--text-sm)",
};

const buttonStyle: CSSProperties = {
  minHeight: 44,
  borderRadius: "var(--radius-md)",
  border: "1px solid var(--border)",
  background: "var(--surface)",
  color: "var(--ink)",
  padding: "0 var(--sp-4)",
  fontSize: "var(--text-sm)",
  fontWeight: "var(--fw-strong)",
  cursor: "pointer",
};

const inputStyle: CSSProperties = {
  minHeight: 44,
  minWidth: 0,
  borderRadius: "var(--radius-md)",
  border: "1px solid var(--border)",
  background: "var(--surface)",
  color: "var(--ink)",
  padding: "0 var(--sp-3)",
  fontFamily: "var(--font-sans)",
  fontSize: "var(--text-sm)",
};

const inlineFormStyle: CSSProperties = {
  display: "grid",
  gridTemplateColumns: "minmax(140px, 1fr) auto",
  gap: "var(--sp-2)",
  alignItems: "center",
};

const stackedFormStyle: CSSProperties = { display: "grid", gap: "var(--sp-2)" };

function timestampLabel(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : TA.datetime(date);
}

function verifyChip(outcome: VerifyOutcome | "running" | null) {
  if (outcome === null) return null;
  if (outcome === "running") return <StatusChip role="status" tone="info">{T.actions.verifying}</StatusChip>;
  switch (outcome.state) {
    case "verified":
      return <StatusChip role="status" tone="ok">{T.actions.verifyOk}</StatusChip>;
    case "processing":
      return <StatusChip role="status" tone="info">{T.actions.verifying}</StatusChip>;
    case "failed":
      return <StatusChip role="alert" tone="danger">{T.actions.verifyFail}</StatusChip>;
    case "denied":
      return <StatusChip role="alert" tone="danger">{TP.permissionDenied}</StatusChip>;
    case "error":
      return <StatusChip role="alert" tone="danger">{T.actions.verifyFail}</StatusChip>;
    case "unavailable":
      return <StatusChip role="status" tone="neutral">{T.actions.verifyPending}</StatusChip>;
  }
}

export interface HoldApplyBody {
  caseRef: string;
  basis: string;
  reason: string;
}

export interface HoldReleaseBody {
  holdId: string;
  reason: string;
  fourEyesRequestRef: string;
}

export interface EvidenceCardProps {
  detail: EvidenceObjectDetail;
  /** Real fixity check (POST /api/v1/evidence/objects/{id}/verify). */
  verify: VerifyEvidence;
  /** The signed-in user — blocks a self-decide in the UI (server also enforces it). */
  currentUserId?: string;
  /** POST /api/v1/evidence/objects/{id}/hold {op:"apply"}. */
  applyHold: (body: HoldApplyBody) => Promise<void>;
  /** POST /api/v1/governance/approvals — opens the release's four-eyes request. */
  requestHoldRelease: (holdId: string) => Promise<{ requestRef: string; requestedBy: string }>;
  /** POST /api/v1/governance/approvals/decide — a distinct approver decides it. */
  decideHoldRelease: (
    requestRef: string,
    requestedBy: string,
    decision: "approved" | "rejected",
  ) => Promise<void>;
  /** POST /api/v1/evidence/objects/{id}/hold {op:"release"}. */
  releaseHold: (body: HoldReleaseBody) => Promise<void>;
}

export function EvidenceCard({
  detail,
  verify,
  currentUserId,
  applyHold,
  requestHoldRelease,
  decideHoldRelease,
  releaseHold,
}: EvidenceCardProps) {
  const [outcome, setOutcome] = useState<VerifyOutcome | "running" | null>(null);
  const [copyVerdicts, setCopyVerdicts] = useState<CopyVerdictMap>(new Map());
  const [caseRef, setCaseRef] = useState("");
  const [basis, setBasis] = useState("");
  const [applyReason, setApplyReason] = useState("");
  const [applyError, setApplyError] = useState<string | null>(null);
  const [applyRefreshRetry, setApplyRefreshRetry] = useState<(() => Promise<void>) | null>(null);
  const [applying, setApplying] = useState(false);
  const [releaseFlow, setReleaseFlow] = useState<ReleaseFlowState>({ stage: "idle" });
  const [releaseReason, setReleaseReason] = useState("");
  const [releaseRefreshRetry, setReleaseRefreshRetry] = useState<(() => Promise<void>) | null>(null);

  const original = originalOf(detail.copies);
  const derivatives = derivativesOf(detail.copies);
  const custody = detail.custody;

  async function runVerify(): Promise<void> {
    // Clear any prior per-copy verdicts up front: a MATCH chip must never linger
    // as stale green when this run ends unavailable/processing/thrown — only a
    // fresh verified/failed result repopulates them.
    setCopyVerdicts(new Map());
    setOutcome("running");
    try {
      const result = await verify(detail);
      setOutcome(result);
      if (result.state === "verified" || result.state === "failed" || result.state === "unavailable") {
        setCopyVerdicts(result.copyVerdicts);
      }
    } catch (error) {
      // A denied action is not evidence corruption. Preserve the authorization
      // truth and suppress futile retries; all other transport/server failures
      // remain retryable through the same action control.
      if (error instanceof ApiCallError && (error.status === 401 || error.status === 403)) {
        setOutcome({ state: "denied" });
      } else {
        setOutcome({ state: "error" });
      }
    }
  }

  async function submitApplyHold(): Promise<void> {
    const ref = caseRef.trim();
    const basisTrimmed = basis.trim();
    const reasonTrimmed = applyReason.trim();
    if (!ref || !basisTrimmed || !reasonTrimmed) {
      setApplyError(T.hold.requiredFields);
      return;
    }
    setApplyError(null);
    setApplyRefreshRetry(null);
    setApplying(true);
    try {
      await applyHold({ caseRef: ref, basis: basisTrimmed, reason: reasonTrimmed });
      setCaseRef("");
      setBasis("");
      setApplyReason("");
    } catch (error) {
      if (error instanceof EvidenceDetailRefreshError) {
        setApplyError(T.hold.applyRefreshFailed);
        setApplyRefreshRetry(() => error.retry);
      } else {
        setApplyError(T.hold.applyFailed);
      }
    } finally {
      setApplying(false);
    }
  }

  async function retryApplyRefresh(): Promise<void> {
    if (!applyRefreshRetry) return;
    setApplying(true);
    try {
      await applyRefreshRetry();
      setApplyError(null);
      setApplyRefreshRetry(null);
      setCaseRef("");
      setBasis("");
      setApplyReason("");
    } catch {
      setApplyError(T.hold.applyRefreshFailed);
    } finally {
      setApplying(false);
    }
  }

  async function startReleaseRequest(holdId: string): Promise<void> {
    setReleaseRefreshRetry(null);
    setReleaseFlow({ stage: "requesting" });
    try {
      const { requestRef, requestedBy } = await requestHoldRelease(holdId);
      setReleaseFlow({ stage: "pending", holdId, requestRef, requestedBy });
    } catch {
      setReleaseFlow({ stage: "error", message: T.hold.releaseFailed });
    }
  }

  async function decideRelease(decision: "approved" | "rejected"): Promise<void> {
    if (releaseFlow.stage !== "pending") return;
    const { holdId, requestRef, requestedBy } = releaseFlow;
    setReleaseFlow({ stage: "deciding", holdId, requestRef, requestedBy });
    try {
      await decideHoldRelease(requestRef, requestedBy, decision);
      if (decision === "rejected") {
        setReleaseFlow({ stage: "idle" });
        return;
      }
      setReleaseFlow({ stage: "releasing", holdId, requestRef });
    } catch {
      setReleaseFlow({ stage: "error", message: T.hold.releaseFailed });
    }
  }

  async function finalizeRelease(): Promise<void> {
    if (releaseFlow.stage !== "releasing") return;
    const { holdId, requestRef } = releaseFlow;
    const reason = releaseReason.trim() || T.hold.defaultReleaseReason;
    setReleaseRefreshRetry(null);
    try {
      await releaseHold({ holdId, reason, fourEyesRequestRef: requestRef });
      setReleaseFlow({ stage: "idle" });
      setReleaseReason("");
    } catch (error) {
      if (error instanceof EvidenceDetailRefreshError) {
        setReleaseRefreshRetry(() => error.retry);
        setReleaseFlow({ stage: "error", message: T.hold.releaseRefreshFailed });
      } else {
        setReleaseFlow({ stage: "error", message: T.hold.releaseFailed });
      }
    }
  }

  async function retryReleaseRefresh(): Promise<void> {
    if (!releaseRefreshRetry) return;
    try {
      await releaseRefreshRetry();
      setReleaseRefreshRetry(null);
      setReleaseFlow({ stage: "idle" });
      setReleaseReason("");
    } catch {
      setReleaseFlow({ stage: "error", message: T.hold.releaseRefreshFailed });
    }
  }

  const activeHold = detail.holds.find((hold) => hold.status === "ACTIVE");
  const selfDecide = releaseFlow.stage === "pending" && currentUserId != null && releaseFlow.requestedBy === currentUserId;

  return (
    <article className="console" aria-label={T.detailAria(detail.code)} style={rootStyle}>
      <header style={headerStyle}>
        <div style={titleRowStyle}>
          <h2 style={titleStyle}>{detail.title}</h2>
          {/* Drag-reference lives on the keyboard-accessible EV- row button in
              EvidenceRecords; a non-focusable grab affordance here fails WCAG 2.1.1. */}
          <span style={monoStyle}>{detail.code}</span>
        </div>
        <div style={chipRowStyle}>
          <StatusChip
            tone={fixityTone(detail.fixity)}
            ariaLabel={T.fixity.aria(original ? original.digestSha256 : T.fixity.missing)}
          >
            {`SHA-256 ${original ? shortDigest(original.digestSha256) : T.fixity.missing}`}
          </StatusChip>
          <StatusChip tone={tsaTone(detail.tsa)}>{T.tsa[detail.tsa]}</StatusChip>
          <StatusChip
            tone={admissibilityTone(detail.admissibility)}
            ariaLabel={T.admissibilityAria(admissibilityLabel(detail.admissibility))}
          >
            {admissibilityLabel(detail.admissibility)}
          </StatusChip>
          {activeHold ? (
            <StatusChip tone="purple" ariaLabel={T.hold.activeAria(activeHold.caseRef)}>
              {T.hold.active}
            </StatusChip>
          ) : null}
          {detail.disposed ? <StatusChip tone="danger">{T.custody.stages.DISPOSED}</StatusChip> : null}
        </div>
      </header>

      {/* WORM split — immutable copies are inspectable as metadata only until
          a separately authorized read endpoint exists. */}
      <section aria-label={T.worm.originalSection} style={sectionStyle}>
        <h3 style={sectionHeadingStyle}>{T.worm.original}</h3>
        {original ? (
          <div style={copyRowStyle}>
            <div style={chipRowStyle}>
              <StatusChip tone="accent" ariaLabel={T.worm.sealedAria}>
                {T.worm.sealed}
              </StatusChip>
              <StatusChip tone={wormTone(original.wormStatus)}>
                {T.worm.status[original.wormStatus]}
              </StatusChip>
              <CopyVerdictChip status={copyVerdicts.get(original.id)} />
            </div>
            <span style={monoStyle}>{original.digestSha256}</span>
            <span style={copyMetaStyle}>
              <span>{original.contentType}</span>
              <span>{formatSize(original.sizeBytes)}</span>
            </span>
          </div>
        ) : (
          <StatusChip tone="danger">{T.worm.originalMissing}</StatusChip>
        )}
      </section>

      {derivatives.length > 0 ? (
        <section aria-label={T.worm.derivatives} style={sectionStyle}>
          <h3 style={sectionHeadingStyle}>{`${T.worm.derivatives} ${String(derivatives.length)}`}</h3>
          <ul style={listStyle}>
            {derivatives.map((copy) => (
              <li key={copy.id} style={copyRowStyle}>
                <div style={chipRowStyle}>
                  <StatusChip tone="neutral">
                    {copy.derivativeKind ? T.derivativeKinds[copy.derivativeKind] : T.worm.derivative}
                  </StatusChip>
                  <StatusChip tone={wormTone(copy.wormStatus)}>
                    {T.worm.status[copy.wormStatus]}
                  </StatusChip>
                  <CopyVerdictChip status={copyVerdicts.get(copy.id)} />
                </div>
                <span style={monoStyle}>{shortDigest(copy.digestSha256)}</span>
                <span style={copyMetaStyle}>
                  <span>{copy.contentType}</span>
                  <span>{formatSize(copy.sizeBytes)}</span>
                </span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      {/* Chain of custody — the audit stream shape (수집→봉인→열람…). */}
      <section aria-label={T.custody.title} style={sectionStyle}>
        <h3 style={sectionHeadingStyle}>{T.custody.title}</h3>
        <ol style={listStyle}>
          {custody.map((event) => {
            const stage = custodyStageOfAudit(event.action);
            return (
              <li key={event.id} style={custodyRowStyle}>
                <StatusChip tone={stage === "WORM_REPLICATED" || stage === "TSA_VERIFIED" ? "ok" : "neutral"}>
                  {stage ? custodyStageLabel(stage) : event.action}
                </StatusChip>
                <span>{event.actor ?? TA.values.systemActor}</span>
                <span style={monoStyle}>{timestampLabel(event.occurred_at)}</span>
              </li>
            );
          })}
        </ol>
      </section>

      {/* Actions — verify is open to viewers of this gated route; custody/hold/
          disposal are PBAC-gated (deny-by-omission). Hold ⇒ dispose disabled,
          fail-closed. */}
      <section aria-label={T.actions.section} style={sectionStyle}>
        <div style={chipRowStyle}>
          <button
            type="button"
            style={buttonStyle}
            disabled={outcome === "running" || (outcome !== null && typeof outcome === "object" && outcome.state === "denied")}
            onClick={() => {
              void runVerify();
            }}
          >
            {T.actions.verify}
          </button>
          {verifyChip(outcome)}
        </div>

        <PolicyGated action={EVIDENCE_ACTIONS.holdManage} resource={{ kind: "evidence_object", id: detail.id }}>
          {activeHold ? (
            <div style={stackedFormStyle}>
              <div style={chipRowStyle}>
                <span style={monoStyle}>{activeHold.caseRef}</span>
              </div>
              {releaseFlow.stage === "idle" || releaseFlow.stage === "requesting" ? (
                <button
                  type="button"
                  style={buttonStyle}
                  disabled={releaseFlow.stage === "requesting"}
                  onClick={() => {
                    void startReleaseRequest(activeHold.id);
                  }}
                >
                  {T.hold.requestRelease}
                </button>
              ) : null}
              {releaseFlow.stage === "pending" || releaseFlow.stage === "deciding" ? (
                <div style={stackedFormStyle}>
                  <StatusChip tone="warn" role="status">{T.hold.releasePending}</StatusChip>
                  {selfDecide ? (
                    <StatusChip tone="danger">{T.hold.selfDecideBlocked}</StatusChip>
                  ) : (
                    <div style={chipRowStyle}>
                      <button
                        type="button"
                        style={buttonStyle}
                        disabled={releaseFlow.stage === "deciding"}
                        onClick={() => {
                          void decideRelease("approved");
                        }}
                      >
                        {T.hold.decideApprove}
                      </button>
                      <button
                        type="button"
                        style={buttonStyle}
                        disabled={releaseFlow.stage === "deciding"}
                        onClick={() => {
                          void decideRelease("rejected");
                        }}
                      >
                        {T.hold.decideReject}
                      </button>
                    </div>
                  )}
                </div>
              ) : null}
              {releaseFlow.stage === "releasing" ? (
                <div style={inlineFormStyle}>
                  <input
                    aria-label={T.hold.reasonLabel}
                    placeholder={T.hold.reasonLabel}
                    style={inputStyle}
                    value={releaseReason}
                    onChange={(event) => {
                      setReleaseReason(event.target.value);
                    }}
                  />
                  <button
                    type="button"
                    style={buttonStyle}
                    onClick={() => {
                      void finalizeRelease();
                    }}
                  >
                    {T.hold.release}
                  </button>
                </div>
              ) : null}
              {releaseFlow.stage === "error" ? (
                <div style={chipRowStyle}>
                  <StatusChip tone="danger" role="alert">{releaseFlow.message}</StatusChip>
                  {releaseRefreshRetry ? (
                    <button type="button" style={buttonStyle} onClick={() => { void retryReleaseRefresh(); }}>
                      {T.hold.refreshRetry}
                    </button>
                  ) : null}
                </div>
              ) : null}
            </div>
          ) : (
            <div style={stackedFormStyle}>
              <input
                aria-label={T.hold.caseRef}
                placeholder={T.hold.caseRef}
                style={inputStyle}
                value={caseRef}
                onChange={(event) => {
                  setCaseRef(event.target.value);
                }}
              />
              <input
                aria-label={T.hold.basisLabel}
                placeholder={T.hold.basisLabel}
                style={inputStyle}
                value={basis}
                onChange={(event) => {
                  setBasis(event.target.value);
                }}
              />
              <input
                aria-label={T.hold.reasonLabel}
                placeholder={T.hold.reasonLabel}
                style={inputStyle}
                value={applyReason}
                onChange={(event) => {
                  setApplyReason(event.target.value);
                }}
              />
              <button
                type="button"
                style={buttonStyle}
                disabled={applying}
                onClick={() => {
                  void submitApplyHold();
                }}
              >
                {T.hold.apply}
              </button>
              {applyError ? (
                <div style={chipRowStyle}>
                  <StatusChip tone="danger" role="alert">{applyError}</StatusChip>
                  {applyRefreshRetry ? (
                    <button type="button" style={buttonStyle} disabled={applying} onClick={() => { void retryApplyRefresh(); }}>
                      {T.hold.refreshRetry}
                    </button>
                  ) : null}
                </div>
              ) : null}
            </div>
          )}
        </PolicyGated>

      </section>

      {/* §4-14 single object-detail substrate. */}
      <ObjectCard descriptor={toObjectCardDescriptor(detail, detail.holds, custody)} />
    </article>
  );
}

/** Per-copy fixity verdict chip, absent until a verify pass reports one. */
function CopyVerdictChip({ status }: { status: CopyFixityStatus | undefined }) {
  if (!status) return null;
  return <StatusChip tone={copyVerdictTone(status)}>{copyVerdictLabel(status)}</StatusChip>;
}

/** §4.7-3 default open gesture — the EV detail as a right-pin window entry. */
export function evidenceWindowEntry(
  detail: EvidenceObjectDetail,
  props: Omit<EvidenceCardProps, "detail">,
): WindowEntry {
  return {
    id: detail.code,
    title: detail.title,
    code: detail.code,
    render: () => <EvidenceCard detail={detail} {...props} />,
  };
}
