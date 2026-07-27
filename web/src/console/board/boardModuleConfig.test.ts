import { describe, expect, it, vi } from "vitest";

import { boardStrings as text } from "../../i18n/board";
import type { BoardApi, BoardNotice } from "./boardApi";
import {
  ackProgressLabel,
  audienceLabel,
  buildBoardModuleConfig,
  categoryLabel,
  noticeStatusChip,
  publishedDayLabel,
} from "./boardModuleConfig";

function notice(over: Partial<BoardNotice> = {}): BoardNotice {
  return {
    id: "n1",
    code: "NT-0707",
    author_user_id: "author-1",
    title: "취업규칙 개정 통지",
    body: "근로기준법 §94 개별 수령확인 대상입니다.",
    status: "published",
    published_at: "2026-07-22T09:00:00+09:00",
    created_at: "2026-07-20T09:00:00+09:00",
    category: "legal",
    audience_scope: "org",
    audience_branches: [],
    my_receipt: null,
    progress: null,
    ...over,
  };
}

function deps(canManage: boolean) {
  return {
    canManage,
    boardApi: { list: vi.fn() } as unknown as BoardApi,
    onReload: vi.fn(),
    onEditDraft: vi.fn(),
    onOpenReceipts: vi.fn(),
  };
}

describe("noticeStatusChip", () => {
  it("derives the four lifecycle chips from status and manager progress", () => {
    expect(noticeStatusChip(notice({ status: "draft" }))).toMatchObject({ label: text.status.draft, tone: "info" });
    expect(noticeStatusChip(notice())).toMatchObject({ label: text.status.published, tone: "ok" });
    expect(noticeStatusChip(notice({ progress: { total: 1284, acknowledged: 1192 } })))
      .toMatchObject({ label: text.status.ackInProgress, tone: "warn" });
    expect(noticeStatusChip(notice({ progress: { total: 44, acknowledged: 44 } })))
      .toMatchObject({ label: text.status.complete, tone: "neutral" });
  });
});

describe("labels", () => {
  it("maps categories with a truthful unknown fallback", () => {
    expect(categoryLabel("legal")).toBe(text.category.legal);
    expect(categoryLabel("hr_order")).toBe(text.category.hr_order);
    expect(categoryLabel(undefined)).toBe(text.category.unknown);
    expect(categoryLabel("mystery")).toBe(text.category.unknown);
  });

  it("labels org and branch audiences", () => {
    expect(audienceLabel(notice())).toBe(text.audienceOrg);
    expect(audienceLabel(notice({
      audience_scope: "branches",
      audience_branches: [{ id: "b1", name: "안산공장" }, { id: "b2", name: "창원지점" }],
    }))).toBe("안산공장 · 창원지점");
  });

  it("renders publish day as 오늘/어제/M-D and — for drafts", () => {
    // Local-time constructions keep the assertions timezone-independent.
    const now = new Date(2026, 6, 24, 10);
    expect(publishedDayLabel(new Date(2026, 6, 24, 1).toISOString(), now)).toBe(text.today);
    expect(publishedDayLabel(new Date(2026, 6, 23, 23).toISOString(), now)).toBe(text.yesterday);
    expect(publishedDayLabel(new Date(2026, 6, 1, 9).toISOString(), now)).toBe("7/1");
    expect(publishedDayLabel(null, now)).toBe("—");
  });

  it("formats acknowledgment progress per the design formula", () => {
    expect(ackProgressLabel(1192, 1284)).toBe("1,192 / 1,284 (93%)");
    expect(ackProgressLabel(0, 0)).toBe("0 / 0 (0%)");
  });
});

describe("buildBoardModuleConfig", () => {
  it("includes the draft stat only for managers", () => {
    const rows = [notice(), notice({ id: "n2", status: "draft", code: null, published_at: null })];
    const managerStats = buildBoardModuleConfig(deps(true)).statbar(rows).map((stat) => stat.key);
    const memberStats = buildBoardModuleConfig(deps(false)).statbar(rows).map((stat) => stat.key);
    expect(managerStats).toEqual(["published", "ackInProgress", "drafts"]);
    expect(memberStats).toEqual(["published", "ackInProgress"]);
  });

  it("offers actions by row state and capability, never as disabled ghosts", () => {
    const manager = buildBoardModuleConfig(deps(true));
    const member = buildBoardModuleConfig(deps(false));
    const draft = notice({ status: "draft", code: null, published_at: null });
    const pendingReceipt = notice({ my_receipt: { acknowledged_at: null } });
    const acknowledged = notice({ my_receipt: { acknowledged_at: "2026-07-23T10:00:00+09:00" } });

    expect(manager.detail.actions(draft).map((action) => action.key)).toEqual(["publish", "edit"]);
    expect(manager.detail.actions(notice()).map((action) => action.key)).toEqual(["receipts"]);
    expect(member.detail.actions(draft)).toEqual([]);
    expect(member.detail.actions(pendingReceipt).map((action) => action.key)).toEqual(["ack"]);
    expect(member.detail.actions(acknowledged)).toEqual([]);
  });

  it("search haystack covers code, title, published day, category, audience, and status label", () => {
    const config = buildBoardModuleConfig(deps(true));
    const draft = notice({ status: "draft", code: null, published_at: null });
    expect(config.search(draft)).toContain(text.status.draft.toLowerCase());
    const inProgress = notice({ progress: { total: 1284, acknowledged: 1192 } });
    const haystack = config.search(inProgress);
    expect(haystack).toContain("nt-0707");
    expect(haystack).toContain(text.status.ackInProgress.toLowerCase());
    expect(haystack).toContain(text.category.legal.toLowerCase());
    expect(haystack).toContain(text.audienceOrg.toLowerCase());
  });

  it("links audience branches as drillable object chips", () => {
    const config = buildBoardModuleConfig(deps(false));
    const row = notice({
      audience_scope: "branches",
      audience_branches: [{ id: "b1", name: "안산공장" }],
    });
    expect(config.detail.links(row)).toEqual([{ code: "b1", label: "안산공장" }]);
    expect(config.detail.links(notice())).toEqual([]);
  });
});
