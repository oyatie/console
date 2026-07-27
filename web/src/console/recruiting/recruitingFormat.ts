import { recruitingStrings as text } from "../../i18n/recruiting";
import type {
  RecruitApplicantStage,
  RecruitApplicantRouting,
  RecruitAssessmentScore,
  RecruitEmploymentType,
  RecruitOfferStatus,
  RecruitOfferView,
  RecruitPostingStatus,
  RecruitPostingRow,
  RecruitRejectReason,
} from "./recruitingApi";

export const STAGE_ORDER: readonly RecruitApplicantStage[] = [
  "APPLIED",
  "SCREENING",
  "INTERVIEW",
  "OFFER",
  "HIRED",
];

export function stageLabel(stage: RecruitApplicantStage): string {
  return stage in text.stage
    ? text.stage[stage as keyof typeof text.stage]
    : text.stage.unknown;
}

export function postingStatusLabel(status: RecruitPostingStatus): string {
  return status in text.postingStatus
    ? text.postingStatus[status as keyof typeof text.postingStatus]
    : text.postingStatus.unknown;
}

export function employmentLabel(employment: RecruitEmploymentType): string {
  return employment in text.employment
    ? text.employment[employment as keyof typeof text.employment]
    : text.employment.unknown;
}

export function scoreLabel(score: RecruitAssessmentScore): string {
  return text.card.scores[score];
}

export function offerStatusLabel(status: RecruitOfferStatus): string {
  return status in text.card.offerStatus
    ? text.card.offerStatus[status as keyof typeof text.card.offerStatus]
    : text.card.offerStatus.unknown;
}

export function rejectReasonLabel(reason: RecruitRejectReason | null): string {
  if (reason && reason in text.card.rejectReasons) {
    return text.card.rejectReasons[reason as keyof typeof text.card.rejectReasons];
  }
  return text.card.rejectReasons.unknown;
}

/** `2026-07-20` → `7/20`; null/empty → 상시 (open-ended recruiting). */
export function deadlineLabel(deadline: string | null): string {
  if (!deadline) return text.always;
  const parts = deadline.split("-");
  if (parts.length === 3) {
    const month = Number.parseInt(parts[1], 10);
    const day = Number.parseInt(parts[2], 10);
    if (Number.isFinite(month) && Number.isFinite(day)) {
      return `${String(month)}/${String(day)}`;
    }
  }
  return deadline;
}

export function dateTimeLabel(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("ko-KR", {
    month: "numeric",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}

/** Group thousands in a decimal-string amount: `3400000` → `3,400,000`. */
export function formatWon(amount: string): string {
  const compact = amount.replaceAll(",", "").trim();
  if (!/^\d+(?:\.\d+)?$/.test(compact)) return amount;
  const [whole, fraction] = compact.split(".");
  const grouped = whole.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return fraction ? `${grouped}.${fraction}` : grouped;
}

export function offerAmountLabel(offer: RecruitOfferView): string {
  const prefix = offer.amount_period === "DAILY" ? text.card.dailyPrefix : text.card.monthlyPrefix;
  return `${prefix}${formatWon(offer.amount)}`;
}

/** The one live/latest offer: highest version wins. */
export function latestOffer(offers: readonly RecruitOfferView[]): RecruitOfferView | undefined {
  let latest: RecruitOfferView | undefined;
  for (const offer of offers) {
    if (!latest || offer.version > latest.version) latest = offer;
  }
  return latest;
}

function lookup(map: Record<string, string>, key: string): string | undefined {
  return key in map ? map[key] : undefined;
}

export function eventLabel(action: string): string {
  return lookup(text.card.event, action.toUpperCase()) ?? action;
}

export function headStatLine(postings: readonly RecruitPostingRow[]): string {
  let applicants = 0;
  let interviews = 0;
  for (const posting of postings) {
    const counts = posting.stage_counts;
    applicants += counts.applied + counts.screening + counts.interview + counts.offer;
    interviews += counts.interview;
  }
  return text.headStat(postings.length, applicants, interviews);
}

export function preflightCheckLabel(key: string): string {
  return lookup(text.preflight.checkLabel, key) ?? key;
}

/** Subrow status line, derived — the backend does not store prose. */
export function applicantStatusLine(applicant: RecruitApplicantRouting): string {
  if ((applicant.rejected_at !== null)) {
    return text.card.rejectedBanner(rejectReasonLabel(applicant.reject_reason));
  }
  return `${stageLabel(applicant.stage)} · ${dateTimeLabel(applicant.updated_at)}`;
}
