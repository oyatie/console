import { useId, useState } from "react";

import { recruitingStrings as text } from "../../i18n/recruiting";
import type {
  HireRecruitApplicantRequest,
  RecruitApplicantDetailResponse,
  RecruitAssessmentScore,
  RecruitOfferDecision,
  RecruitOfferPeriod,
  RecruitPostingView,
  RecruitRejectReason,
} from "./recruitingApi";
import type { RecruitingCapabilities } from "./recruitingCapabilities";
import {
  STAGE_ORDER,
  dateTimeLabel,
  eventLabel,
  latestOffer,
  offerAmountLabel,
  offerStatusLabel,
  rejectReasonLabel,
  scoreLabel,
  stageLabel,
} from "./recruitingFormat";

const SCORES: readonly RecruitAssessmentScore[] = ["SUITABLE", "NEUTRAL", "UNSUITABLE"];
const REJECT_REASONS: readonly RecruitRejectReason[] = [
  "CAREER_SHORTFALL",
  "ROLE_MISMATCH",
  "COMP_MISMATCH",
  "ACCEPTED_ELSEWHERE",
  "OTHER",
];

interface Branch {
  id: string;
  name: string;
}

type Props = {
  posting: RecruitPostingView;
  detail: RecruitApplicantDetailResponse;
  capabilities: RecruitingCapabilities;
  busy: boolean;
  /** Server-reported action failure (422/409/…) — rendered fail-closed. */
  error?: string;
  branches?: Branch[];
  branchesError: boolean;
  onLoadBranches: () => void;
  onClose: () => void;
  onOpenPosting: () => void;
  onOpenEmployee: (employeeId: string) => void;
  onOpenPosition: (positionRef: string) => void;
  onAdvance: () => void;
  onAssess: (score: RecruitAssessmentScore) => void;
  onSendOffer: (input: { amount: string; amount_period: RecruitOfferPeriod; reply_deadline: string }) => void;
  onAdjustOffer: (offerId: string, input: { amount: string; reply_deadline?: string }) => void;
  onWithdrawOffer: (offerId: string, reason: string) => void;
  onRecordReply: (offerId: string, decision: RecruitOfferDecision) => void;
  onRequestDocuments: () => void;
  onReject: (reason: RecruitRejectReason) => void;
  onReinstate: () => void;
  onHire: (input: HireRecruitApplicantRequest) => void;
  /** Client fail-closed guard message setter (assessment-before-offer). */
  onGuard: (message: string) => void;
};

/**
 * 지원자 카드 — right-side modal: stepper, profile + provenance, scorecard,
 * offer chain, enum-reason rejection, reinstate, hire handshake. Every
 * mutation is server-reconciled by the parent; this component renders state.
 */
export function CandidateCard(props: Props) {
  const { posting, detail, capabilities, busy, error } = props;
  const applicant = detail.applicant;
  const offer = latestOffer(detail.offers);
  const isPool = posting.employment_type === "POOL_DAILY";
  const [rejectMenuOpen, setRejectMenuOpen] = useState(false);
  const [offerFormOpen, setOfferFormOpen] = useState(false);
  const [offerAmount, setOfferAmount] = useState("");
  const [offerPeriod, setOfferPeriod] = useState<RecruitOfferPeriod>(isPool ? "DAILY" : "MONTHLY");
  const [offerDeadline, setOfferDeadline] = useState("");
  const [adjustAmount, setAdjustAmount] = useState("");
  const [hireOpen, setHireOpen] = useState(false);
  const [hireForm, setHireForm] = useState<Record<keyof HireRecruitApplicantRequest, string>>({
    employee_number: "",
    phone: "",
    org_unit: "",
    position: posting.role_title,
    site: posting.worksite,
    home_branch_id: "",
    base_pay: offer ? offer.amount : "",
  });
  const offerAmountId = useId();
  const offerPeriodId = useId();
  const offerDeadlineId = useId();
  const hireIds = {
    employeeNumber: useId(),
    phone: useId(),
    orgUnit: useId(),
    position: useId(),
    site: useId(),
    branch: useId(),
    basePay: useId(),
  };

  const stageIndex = STAGE_ORDER.indexOf(applicant.stage);
  const canManage = capabilities.canManage && !busy;
  const hireable =
    applicant.stage === "OFFER" &&
    !(applicant.rejected_at !== null) &&
    !isPool &&
    offer?.status === "ACCEPTED" &&
    capabilities.canHire;

  const primary = (() => {
    if (!capabilities.canManage || (applicant.rejected_at !== null)) return undefined;
    if (applicant.stage === "APPLIED") return { label: text.card.ctaScreening, run: props.onAdvance };
    if (applicant.stage === "SCREENING") return { label: text.card.ctaInterview, run: props.onAdvance };
    if (applicant.stage === "INTERVIEW") {
      return {
        label: text.card.ctaOffer,
        run: () => {
          if (!applicant.assessment) {
            props.onGuard(text.card.assessmentRequired);
            return;
          }
          setOfferFormOpen(true);
        },
      };
    }
    if (hireable) {
      return {
        label: text.card.ctaHire,
        run: () => {
          setHireOpen(true);
          props.onLoadBranches();
        },
      };
    }
    return undefined;
  })();

  const updateHire = (key: keyof HireRecruitApplicantRequest, value: string) => {
    setHireForm((current) => ({ ...current, [key]: value }));
  };
  const hireReady = [
    hireForm.employee_number,
    hireForm.phone,
    hireForm.org_unit,
    hireForm.position,
    hireForm.site,
    hireForm.home_branch_id,
    hireForm.base_pay,
  ].every((value) => value.trim() !== "");

  return (
    <div
      className="recruiting__overlay recruiting__overlay--end"
      onClick={(event) => { if (event.target === event.currentTarget) props.onClose(); }}
      onKeyDown={(event) => { if (event.key === "Escape") props.onClose(); }}
    >
      <div role="dialog" aria-modal="true" aria-labelledby="recruiting-card-title" className="recruiting__card">
        <div className="recruiting__card-head">
          <span className="recruiting__avatar" aria-hidden="true">{applicant.name.slice(0, 1)}</span>
          <div className="recruiting__card-who">
            <div id="recruiting-card-title" className="recruiting__card-name">{applicant.name}</div>
            <div className="recruiting__card-meta">
              <button type="button" className="recruiting__link" onClick={props.onOpenPosting}>
                {posting.role_title}
              </button>
              <span className="recruiting__card-meta-sep">· {posting.company} · {posting.worksite}</span>
              {posting.position_ref !== null && (
                <button
                  type="button"
                  className="recruiting__chip recruiting__chip--mono recruiting__chip--action"
                  onClick={() => { if (posting.position_ref !== null) props.onOpenPosition(posting.position_ref); }}
                >{posting.position_ref}</button>
              )}
            </div>
          </div>
          <button type="button" className="recruiting__icon-button" aria-label={text.card.close} autoFocus onClick={props.onClose}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M18 6 6 18 M6 6l12 12" /></svg>
          </button>
        </div>
        <div className="recruiting__stepper-rail" aria-hidden="true">
          {STAGE_ORDER.map((stage, index) => (
            <span key={stage} className="recruiting__step-slot">
              {index > 0 && (
                <span className={!(applicant.rejected_at !== null) && index <= stageIndex ? "recruiting__step-line recruiting__step-line--done" : "recruiting__step-line"} />
              )}
              <span className="recruiting__step">
                <span className={
                  (applicant.rejected_at !== null) ? "recruiting__step-dot"
                    : index < stageIndex ? "recruiting__step-dot recruiting__step-dot--done"
                      : index === stageIndex ? "recruiting__step-dot recruiting__step-dot--now"
                        : "recruiting__step-dot"
                } />
                <span className={!(applicant.rejected_at !== null) && index === stageIndex ? "recruiting__step-label recruiting__step-label--now" : "recruiting__step-label"}>{stageLabel(stage)}</span>
              </span>
            </span>
          ))}
        </div>
        <div className="recruiting__card-body">
          {(applicant.rejected_at !== null) && (
            <div className="recruiting__banner recruiting__banner--danger recruiting__banner--row" role="status">
              <span className="recruiting__banner-grow">{text.card.rejectedBanner(rejectReasonLabel(applicant.reject_reason))}</span>
              {capabilities.canManage && (
                <button type="button" className="recruiting__ghost recruiting__ghost--danger" disabled={busy} onClick={props.onReinstate}>{text.card.reinstate}</button>
              )}
            </div>
          )}
          <section aria-label={text.card.profile}>
            <div className="recruiting__section-head">
              <span className="recruiting__section-title">{text.card.profile}</span>
              <span className="recruiting__spacer" />
              {applicant.source_document !== null && (
                <span className="recruiting__chip recruiting__chip--faint" title={text.card.sourceHint}>
                  {text.card.sourcePrefix}{applicant.source_document}
                </span>
              )}
            </div>
            <ul className="recruiting__profile">
              {applicant.profile_lines.map((line) => <li key={line}>{line}</li>)}
            </ul>
          </section>
          <section aria-label={text.card.assessment}>
            <div className="recruiting__section-head">
              <span className="recruiting__section-title">{text.card.assessment}</span>
              {applicant.assessment !== null && (
                <span className="recruiting__section-note">{applicant.assessment.by ?? ""} · {applicant.assessment.at === null ? "" : dateTimeLabel(applicant.assessment.at)}</span>
              )}
            </div>
            <div className="recruiting__chip-row">
              {SCORES.map((score) => {
                const active = applicant.assessment?.score === score;
                const className = !active ? "recruiting__score"
                  : score === "SUITABLE" ? "recruiting__score recruiting__score--ok"
                    : score === "UNSUITABLE" ? "recruiting__score recruiting__score--danger"
                      : "recruiting__score recruiting__score--warn";
                return capabilities.canManage && !(applicant.rejected_at !== null) ? (
                  <button key={score} type="button" className={className} aria-pressed={active} disabled={busy} onClick={() => { props.onAssess(score); }}>{scoreLabel(score)}</button>
                ) : (
                  active && <span key={score} className={className}>{scoreLabel(score)}</span>
                );
              })}
            </div>
          </section>
          {offer !== undefined && !(applicant.rejected_at !== null) && (
            <section aria-label={text.card.offer} className="recruiting__offer">
              <div className="recruiting__section-head">
                <span className="recruiting__section-title">{text.card.offer}</span>
                <span className="recruiting__mono recruiting__offer-amount">{offerAmountLabel(offer)}</span>
                <span className={offer.status === "ACCEPTED" ? "recruiting__chip recruiting__chip--ok" : offer.status === "EXTENDED" ? "recruiting__chip recruiting__chip--warn" : "recruiting__chip recruiting__chip--muted"}>
                  {offerStatusLabel(offer.status)}
                </span>
                <span className="recruiting__section-note">{text.card.offerSentPrefix}{dateTimeLabel(offer.extended_at)}</span>
              </div>
              <div className="recruiting__section-note">{text.card.replyDeadline} {dateTimeLabel(offer.reply_deadline)}</div>
              {capabilities.canManage && offer.status === "EXTENDED" && (
                <>
                  <div className="recruiting__req-row">
                    <input
                      className="recruiting__input"
                      value={adjustAmount}
                      placeholder={text.card.adjustPlaceholder}
                      onChange={(event) => { setAdjustAmount(event.target.value.replaceAll(/[^\d]/g, "")); }}
                    />
                    <button
                      type="button"
                      className="recruiting__ghost"
                      disabled={busy || adjustAmount.trim() === ""}
                      onClick={() => { props.onAdjustOffer(offer.id, { amount: adjustAmount.trim() }); setAdjustAmount(""); }}
                    >{text.card.adjustSend}</button>
                    <button type="button" className="recruiting__ghost recruiting__ghost--danger" disabled={busy} onClick={() => { props.onWithdrawOffer(offer.id, text.card.withdrawReason); }}>{text.card.withdraw}</button>
                  </div>
                  <div className="recruiting__req-row">
                    <button type="button" className="recruiting__ghost recruiting__ghost--ok" disabled={busy} onClick={() => { props.onRecordReply(offer.id, "ACCEPTED"); }}>{text.card.recordAccept}</button>
                    <button type="button" className="recruiting__ghost" disabled={busy} onClick={() => { props.onRecordReply(offer.id, "DECLINED"); }}>{text.card.recordDecline}</button>
                  </div>
                </>
              )}
            </section>
          )}
          {offerFormOpen && capabilities.canManage && !(applicant.rejected_at !== null) && applicant.stage === "INTERVIEW" && (
            <section aria-label={text.card.ctaOffer} className="recruiting__offer">
              <div className="recruiting__section-head">
                <span className="recruiting__section-title">{text.card.ctaOffer}</span>
              </div>
              <div className="recruiting__field-grid recruiting__field-grid--tail">
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={offerAmountId}>{text.card.offerAmount}</label>
                  <input
                    id={offerAmountId}
                    className="recruiting__input"
                    inputMode="numeric"
                    value={offerAmount}
                    onChange={(event) => { setOfferAmount(event.target.value.replaceAll(/[^\d]/g, "")); }}
                  />
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={offerPeriodId}>{text.card.offer}</label>
                  <select
                    id={offerPeriodId}
                    className="recruiting__input"
                    value={offerPeriod}
                    onChange={(event) => { setOfferPeriod(event.target.value === "DAILY" ? "DAILY" : "MONTHLY"); }}
                  >
                    <option value="MONTHLY">{text.card.offerPeriod.MONTHLY}</option>
                    <option value="DAILY">{text.card.offerPeriod.DAILY}</option>
                  </select>
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={offerDeadlineId}>{text.card.replyDeadline}</label>
                  <input
                    id={offerDeadlineId}
                    className="recruiting__input recruiting__input--date"
                    type="date"
                    value={offerDeadline}
                    onChange={(event) => { setOfferDeadline(event.target.value); }}
                  />
                </div>
              </div>
              <div className="recruiting__req-row">
                <span className="recruiting__spacer" />
                <button
                  type="button"
                  className="recruiting__primary"
                  disabled={busy || offerAmount.trim() === "" || offerDeadline === ""}
                  onClick={() => {
                    props.onSendOffer({ amount: offerAmount.trim(), amount_period: offerPeriod, reply_deadline: offerDeadline });
                    setOfferFormOpen(false);
                  }}
                >{text.card.offerSend}</button>
              </div>
            </section>
          )}
          {hireOpen && hireable && (
            <section aria-label={text.hire.title} className="recruiting__offer">
              <div className="recruiting__section-head">
                <span className="recruiting__section-title">{text.hire.title}</span>
              </div>
              {props.branchesError && (
                <div className="recruiting__banner recruiting__banner--danger recruiting__banner--row" role="alert">
                  <span className="recruiting__banner-grow">{text.hire.branchesError}</span>
                  <button type="button" className="recruiting__ghost" onClick={props.onLoadBranches}>{text.retry}</button>
                </div>
              )}
              <div className="recruiting__field-grid">
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.employeeNumber}>{text.hire.employeeNumber}</label>
                  <input id={hireIds.employeeNumber} className="recruiting__input" value={hireForm.employee_number} onChange={(event) => { updateHire("employee_number", event.target.value); }} />
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.phone}>{text.hire.phone}</label>
                  <input id={hireIds.phone} className="recruiting__input" value={hireForm.phone} onChange={(event) => { updateHire("phone", event.target.value); }} />
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.orgUnit}>{text.hire.orgUnit}</label>
                  <input id={hireIds.orgUnit} className="recruiting__input" value={hireForm.org_unit} onChange={(event) => { updateHire("org_unit", event.target.value); }} />
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.position}>{text.hire.position}</label>
                  <input id={hireIds.position} className="recruiting__input" value={hireForm.position} onChange={(event) => { updateHire("position", event.target.value); }} />
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.site}>{text.hire.site}</label>
                  <input id={hireIds.site} className="recruiting__input" value={hireForm.site} onChange={(event) => { updateHire("site", event.target.value); }} />
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.branch}>{text.hire.homeBranch}</label>
                  <select id={hireIds.branch} className="recruiting__input" value={hireForm.home_branch_id} onChange={(event) => { updateHire("home_branch_id", event.target.value); }}>
                    <option value="" />
                    {(props.branches ?? []).map((branch) => (
                      <option key={branch.id} value={branch.id}>{branch.name}</option>
                    ))}
                  </select>
                </div>
                <div className="recruiting__field">
                  <label className="recruiting__field-label" htmlFor={hireIds.basePay}>{text.hire.basePay}</label>
                  <input id={hireIds.basePay} className="recruiting__input" inputMode="numeric" value={hireForm.base_pay} onChange={(event) => { updateHire("base_pay", event.target.value.replaceAll(/[^\d]/g, "")); }} />
                </div>
              </div>
              <div className="recruiting__req-row">
                <button type="button" className="recruiting__ghost" onClick={() => { setHireOpen(false); }}>{text.hire.cancel}</button>
                <span className="recruiting__spacer" />
                <button type="button" className="recruiting__primary" disabled={busy || !hireReady} onClick={() => { props.onHire({ ...hireForm, base_pay: hireForm.base_pay === "" ? null : hireForm.base_pay }); }}>{text.hire.submit}</button>
              </div>
            </section>
          )}
          {applicant.stage === "HIRED" && applicant.hired_employee_id !== null && (
            <button type="button" className="recruiting__ghost recruiting__ghost--ok" onClick={() => { if (applicant.hired_employee_id !== null) props.onOpenEmployee(applicant.hired_employee_id); }}>
              {text.hire.openEmployee}
            </button>
          )}
          {error !== undefined && (
            <div className="recruiting__banner recruiting__banner--danger" role="alert">{error}</div>
          )}
          <section aria-label={text.card.history}>
            <div className="recruiting__section-head">
              <span className="recruiting__section-title">{text.card.history}</span>
            </div>
            {detail.events.length === 0 ? (
              <p className="recruiting__state">{text.card.historyEmpty}</p>
            ) : (
              <ol className="recruiting__events">
                {detail.events.map((event) => (
                  <li key={event.id} className="recruiting__event">
                    <span className="recruiting__mono recruiting__event-at">{dateTimeLabel(event.occurred_at)}</span>
                    <span className="recruiting__event-label">{eventLabel(event.action)}</span>
                    {typeof event.actor === "string" && event.actor !== "" && (
                      <span className="recruiting__section-note">{event.actor}</span>
                    )}
                    {typeof event.reason === "string" && event.reason !== "" && (
                      <span className="recruiting__section-note recruiting__event-note">{event.reason}</span>
                    )}
                  </li>
                ))}
              </ol>
            )}
          </section>
        </div>
        {capabilities.canManage && !(applicant.rejected_at !== null) && applicant.stage !== "HIRED" && (
          <div className="recruiting__card-foot">
            <div className="recruiting__menu-anchor">
              <button type="button" className="recruiting__ghost recruiting__ghost--danger" disabled={busy} onClick={() => { setRejectMenuOpen((open) => !open); }} aria-expanded={rejectMenuOpen}>
                {text.card.reject}
              </button>
              {rejectMenuOpen && (
                <div className="recruiting__menu recruiting__menu--up" role="menu">
                  {REJECT_REASONS.map((reason) => (
                    <button
                      key={reason}
                      type="button"
                      role="menuitem"
                      className="recruiting__menu-item"
                      onClick={() => { setRejectMenuOpen(false); props.onReject(reason); }}
                    >{rejectReasonLabel(reason)}</button>
                  ))}
                </div>
              )}
            </div>
            <button type="button" className="recruiting__ghost" disabled={busy || !canManage} onClick={props.onRequestDocuments}>{text.card.requestDocuments}</button>
            <span className="recruiting__spacer" />
            {primary && (
              <button type="button" className="recruiting__primary" disabled={busy} onClick={primary.run}>{primary.label}</button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
