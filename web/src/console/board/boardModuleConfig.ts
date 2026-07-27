import { boardStrings as text } from "../../i18n/board";
import type { Tone } from "../composer/objectKinds";
import type { ModuleAction, ModuleConfig, ModuleKv, ModuleStat } from "../module/config";
import { BOARD_ACK_ACTION, NOTICE_MANAGE_FEATURE } from "./boardCapabilities";
import type { BoardApi, BoardNotice } from "./boardApi";

const DAY_MS = 86_400_000;

export function categoryLabel(category: string | undefined): string {
  if (category && category in text.category) {
    return text.category[category as keyof typeof text.category];
  }
  return text.category.unknown;
}

export function audienceLabel(notice: BoardNotice): string {
  if (notice.audience_scope === "branches") {
    const names = notice.audience_branches.map((branch) => branch.name).filter((name) => name.length > 0);
    return names.length > 0 ? names.join(" · ") : "—";
  }
  return text.audienceOrg;
}

export function publishedDayLabel(iso: string | null | undefined, now = new Date()): string {
  if (!iso) return "—";
  const at = new Date(iso);
  if (Number.isNaN(at.getTime())) return "—";
  const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const days = Math.round((startOfDay(now) - startOfDay(at)) / DAY_MS);
  if (days <= 0) return text.today;
  if (days === 1) return text.yesterday;
  return `${String(at.getMonth() + 1)}/${String(at.getDate())}`;
}

const timestampFormat = new Intl.DateTimeFormat("ko-KR", { dateStyle: "medium", timeStyle: "short" });

export function timestampLabel(iso: string | null | undefined): string {
  if (!iso) return "—";
  const at = new Date(iso);
  return Number.isNaN(at.getTime()) ? "—" : timestampFormat.format(at);
}

export interface NoticeStatusChip {
  key: "draft" | "published" | "ackInProgress" | "complete";
  label: string;
  tone: Tone;
}

/**
 * Lifecycle chip: draft → published; completion is DERIVED from the manager
 * progress embed (`acknowledged == total`). Callers without progress (non
 * managers) truthfully see the published state only.
 */
export function noticeStatusChip(notice: BoardNotice): NoticeStatusChip {
  if (notice.status === "draft") {
    return { key: "draft", label: text.status.draft, tone: "info" };
  }
  const progress = notice.progress;
  if (progress && progress.total > 0) {
    return progress.acknowledged >= progress.total
      ? { key: "complete", label: text.status.complete, tone: "neutral" }
      : { key: "ackInProgress", label: text.status.ackInProgress, tone: "warn" };
  }
  return { key: "published", label: text.status.published, tone: "ok" };
}

export function ackProgressLabel(done: number, total: number): string {
  const pct = Math.round((done / Math.max(1, total)) * 100);
  return `${done.toLocaleString("ko-KR")} / ${total.toLocaleString("ko-KR")} (${String(pct)}%)`;
}

export interface BoardConfigDeps {
  canManage: boolean;
  boardApi: BoardApi;
  /** Refetch the list after a real mutation (ack/publish). */
  onReload: () => void;
  /** Open the board-owned draft composer prefilled with the row. */
  onEditDraft: (row: BoardNotice) => void;
  /** Open the board-owned receipts drill (직원 1:N history layer). */
  onOpenReceipts: (row: BoardNotice) => void;
}

/** Data-only specialization of the ONE generic ModuleScreen (never a fork). */
export function buildBoardModuleConfig(deps: BoardConfigDeps): ModuleConfig<BoardNotice> {
  const { canManage, boardApi, onReload, onEditDraft, onOpenReceipts } = deps;
  return {
    key: "board",
    title: text.title,
    rowId: (row) => row.id,
    rowTitle: (row) => row.title,
    columns: [
      { key: "code", header: text.columns.code, width: 96, minWidth: 72, cell: (row) => ({ text: row.code ?? "—", mono: true }) },
      { key: "notice", header: text.columns.notice, width: 280, minWidth: 160, cell: (row) => ({ text: row.title }) },
      { key: "published", header: text.columns.published, width: 88, minWidth: 56, cell: (row) => ({ text: publishedDayLabel(row.published_at) }) },
      { key: "audience", header: text.columns.audience, width: 160, minWidth: 96, cell: (row) => ({ text: audienceLabel(row) }) },
      {
        key: "ack",
        header: text.columns.ack,
        width: 120,
        minWidth: 88,
        cell: (row) => {
          const chip = noticeStatusChip(row);
          return { text: chip.label, tone: chip.tone };
        },
      },
    ],
    statbar: (rows) => {
      const published = rows.filter((row) => row.status === "published").length;
      const ackInProgress = rows.filter((row) => noticeStatusChip(row).key === "ackInProgress").length;
      const stats: ModuleStat[] = [
        { key: "published", label: text.stats.published, value: String(published) },
        { key: "ackInProgress", label: text.stats.ackInProgress, value: String(ackInProgress), tone: "warn" },
      ];
      if (canManage) {
        const drafts = rows.filter((row) => row.status === "draft").length;
        stats.push({ key: "drafts", label: text.stats.drafts, value: String(drafts) });
      }
      return stats;
    },
    // Haystack mirrors the prototype MOD search: code + title + published day +
    // audience + status chip + category (+ body, richer than the mock).
    search: (row) =>
      [
        row.code ?? "",
        row.title,
        row.body,
        publishedDayLabel(row.published_at),
        categoryLabel(row.category),
        audienceLabel(row),
        noticeStatusChip(row).label,
      ]
        .join(" ")
        .toLowerCase(),
    detail: {
      kv: (row) => {
        const kv: ModuleKv[] = [
          { key: "category", label: text.detail.category, value: categoryLabel(row.category) },
          { key: "published", label: text.detail.published, value: timestampLabel(row.published_at) },
          { key: "audience", label: text.detail.audience, value: audienceLabel(row) },
        ];
        if (row.progress) {
          kv.push({
            key: "ack",
            label: text.detail.ack,
            value: ackProgressLabel(row.progress.acknowledged, row.progress.total),
          });
        }
        if (row.my_receipt) {
          kv.push({
            key: "myAck",
            label: text.detail.myAck,
            value: row.my_receipt.acknowledged_at
              ? timestampLabel(row.my_receipt.acknowledged_at)
              : text.detail.myAckPending,
          });
        }
        return kv;
      },
      // Audience branch chips — the upstream object references; the screen
      // body's onOpenObject drills the LIST to that branch's notices.
      links: (row) => row.audience_branches.map((branch) => ({ code: branch.id, label: branch.name })),
      actions: (row) => {
        const actions: ModuleAction<BoardNotice>[] = [];
        if (row.my_receipt && !row.my_receipt.acknowledged_at) {
          actions.push({
            key: "ack",
            label: text.actions.ack,
            policy: BOARD_ACK_ACTION,
            tone: "ok",
            run: async () => {
              await boardApi.ack(row.id);
              onReload();
              return text.toasts.acked;
            },
          });
        }
        if (canManage && row.status === "draft") {
          actions.push({
            key: "publish",
            label: text.actions.publish,
            policy: NOTICE_MANAGE_FEATURE,
            run: async () => {
              await boardApi.publish(row.id);
              onReload();
              return text.toasts.published;
            },
          });
          actions.push({
            key: "edit",
            label: text.actions.edit,
            policy: NOTICE_MANAGE_FEATURE,
            // Opens the composer; the empty toast label is skipped by the body.
            run: () => {
              onEditDraft(row);
              return Promise.resolve("");
            },
          });
        }
        if (canManage && row.status === "published") {
          actions.push({
            key: "receipts",
            label: text.actions.receipts,
            policy: NOTICE_MANAGE_FEATURE,
            run: () => {
              onOpenReceipts(row);
              return Promise.resolve("");
            },
          });
        }
        return actions;
      },
    },
    primaryAction: { key: "compose", label: text.compose, policy: NOTICE_MANAGE_FEATURE },
    load: () => boardApi.list(200),
  };
}
