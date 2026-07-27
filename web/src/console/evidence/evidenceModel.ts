// Pure display/derivation logic for the EV- evidence surface.
import { ko } from "../../i18n/ko";
import type { ObjectCardDescriptor, StatusTone } from "../objectcard";
import type { AuditRecord } from "../audit";
import type {
  AdmissibilityStatus,
  CopyFixityStatus,
  CustodyStage,
  EvidenceCopy,
  EvidenceLegalHold,
  EvidenceObjectDetail,
  FixityStatus,
  TsaStatus,
  WormStatus,
} from "./types";

const T = ko.console.evidence;

export function admissibilityLabel(status: AdmissibilityStatus): string {
  return T.admissibility[status];
}

export function admissibilityTone(status: AdmissibilityStatus): StatusTone {
  switch (status) {
    case "ADMISSIBLE":
      return "ok";
    case "REVIEW_NEEDED":
      return "warn";
    case "BLOCKED":
      return "purple";
    case "INADMISSIBLE":
      return "danger";
  }
}

export function fixityTone(fixity: FixityStatus): StatusTone {
  return fixity === "VERIFIED" ? "ok" : fixity === "PENDING" ? "info" : "danger";
}

export function tsaTone(tsa: TsaStatus): StatusTone {
  if (tsa === "VERIFIED") return "ok";
  if (tsa === "FAILED" || tsa === "REVOKED" || tsa === "EXPIRED_CA") return "danger";
  return "neutral";
}

export function copyVerdictTone(status: CopyFixityStatus): StatusTone {
  return status === "MATCH" ? "ok" : status === "MISMATCH" ? "danger" : "warn";
}

export function copyVerdictLabel(status: CopyFixityStatus): string {
  return T.copyVerdict[status];
}

export function wormTone(status: WormStatus): StatusTone {
  return status === "VERIFIED" ? "ok" : status === "PENDING" ? "info" : "danger";
}

/** First 12 hex chars — enough to eyeball, full digest stays in the detail. */
export function shortDigest(sha256: string): string {
  return `${sha256.slice(0, 12)}…`;
}

export function holdActive(holds: readonly EvidenceLegalHold[]): boolean {
  return holds.some((hold) => hold.status === "ACTIVE");
}

export function originalOf(copies: readonly EvidenceCopy[]): EvidenceCopy | undefined {
  return copies.find((copy) => copy.kind === "ORIGINAL");
}

export function derivativesOf(copies: readonly EvidenceCopy[]): EvidenceCopy[] {
  return copies.filter((copy) => copy.kind === "DERIVATIVE");
}

// Custody stages are emitted by docs-rest as the generated CustodyStage enum.
// The console accepts only that authoritative ledger value; it never infers a
// stage from a read/audit action or from a local transition suggestion.
const WIRE_CUSTODY_STAGES: ReadonlySet<string> = new Set<CustodyStage>([
  "REGISTERED",
  "HASH_RECORDED",
  "TSA_SUBMITTED",
  "TSA_VERIFIED",
  "WORM_REPLICATED",
  "CUSTODY_TRANSFERRED",
  "UNDER_REVIEW",
  "ADMISSIBILITY_EVALUATED",
  "LEGAL_HOLD_APPLIED",
  "LEGAL_HOLD_RELEASED",
  "EXPORTED",
  "ARCHIVED",
  "DISPOSAL_REQUESTED",
  "DISPOSED",
]);

export function custodyStageOfAudit(action: string): CustodyStage | null {
  return WIRE_CUSTODY_STAGES.has(action) ? (action as CustodyStage) : null;
}

export function custodyStageLabel(stage: CustodyStage): string {
  return T.custody.stages[stage];
}

/** Format bytes for display, KB/MB with one decimal. */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${String(bytes)}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}

/**
 * §4-14 composition: the EV object rendered through the single object-detail
 * surface. Evidence-specific chrome (fixity/TSA/WORM/custody/hold) is layered
 * by EvidenceCard; evidence actions are PBAC-gated there, so no ObjectCard
 * actions here.
 */
export function toObjectCardDescriptor(
  detail: EvidenceObjectDetail,
  holds: readonly EvidenceLegalHold[],
  custody: readonly AuditRecord[],
): ObjectCardDescriptor {
  const original = originalOf(detail.copies);
  const locked = holdActive(holds);
  const lifecycleState = detail.disposed ? "disposed" : locked ? "locked" : "active";
  return {
    id: detail.id,
    code: detail.code,
    title: detail.title,
    objectType: { key: "evidence_object", title: T.title },
    lifecycleState,
    properties: [
      { key: "classification", title: T.fields.classification, type: "choice", value: detail.classification },
      { key: "custodian", title: T.fields.custodian, type: "user", value: detail.custodian },
      { key: "registered_at", title: T.fields.registeredAt, type: "datetime", value: detail.registeredAt },
      {
        key: "sha256",
        title: T.fields.sha256,
        type: "text",
        value: original ? original.digestSha256 : null,
      },
      {
        key: "content_type",
        title: T.fields.contentType,
        type: "text",
        value: original ? original.contentType : null,
      },
    ],
    relations: detail.source
      ? [
          {
            linkId: `src-${detail.id}`,
            linkType: T.fields.source,
            direction: "from",
            cardinality: "one_one",
            code: detail.source.code,
            title: detail.source.title,
          },
        ]
      : [],
    lifecycle: [
      { state: "draft", reached: true, current: false },
      { state: "active", reached: true, current: lifecycleState === "active" },
      { state: "locked", reached: locked || detail.disposed, current: lifecycleState === "locked" },
      { state: "disposed", reached: detail.disposed, current: lifecycleState === "disposed" },
    ],
    history: custody.map((event, index) => ({
      version: custody.length - index,
      at: event.occurred_at,
      actor: event.actor ?? ko.console.audit.values.systemActor,
      hashVerified: detail.fixity === "VERIFIED",
      action: event.action,
    })),
    actions: [],
  };
}
