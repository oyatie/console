// 알림 module copy — module-owned i18n resource (same mechanism as
// production.ts / salesCrm.ts; the shared ko.ts stays integrator-owned).
import { ko } from "./ko";
import { categoryLabel } from "./notificationCategories";

export const notifStrings = {
  title: "알림",
  denied: "알림을 보려면 로그인이 필요합니다.",
  loading: "알림을 불러오는 중입니다.",
  loadError: "알림을 불러오지 못했습니다.",
  actionError: "요청을 처리하지 못했습니다. 다시 시도하세요.",
  retry: "다시 시도",
  empty: "표시할 알림이 없습니다.",
  emptyUnread: "미확인 알림이 없습니다.",
  emptyGroups: "표시할 개체 알림이 없습니다.",
  filterAll: "전체",
  filterUnread: "미확인",
  viewTimeline: "시간순",
  viewByObject: "개체별",
  filterLabel: "알림 필터",
  viewLabel: "알림 보기 방식",
  markAllRead: "모두 읽음",
  markRead: "읽음 처리",
  markUnread: "미확인으로 되돌리기",
  mute: "알림 끄기",
  unmute: "알림 켜기",
  loadMore: "더 보기",
  unreadBadge: "미확인 알림 수",
  mutedBadge: "숨긴 미확인 알림 수",
  mutedShort: "숨김",
  list: "알림 목록",
  groups: "개체별 알림 목록",
  groupUnread: "미확인",
  groupTotal: "전체",
  objectFilterClear: "개체 필터 해제",
} as const;

/**
 * Category chip tone map (§4-18 token tones). `NotificationSummary.category`
 * is an OPEN producer string authored as Korean literals by the backend
 * producers, so the keys are data values, not UI copy — kept in src/i18n/
 * (the Hangul-allowed directory, same rule as notificationCategories.ts).
 * Unknown categories fall back to the neutral chip.
 */
const CATEGORY_TONE: Record<
  string,
  "accent" | "purple" | "info" | "ok" | "warn"
> = {
  결재: "accent",
  멘션: "purple",
  문서: "info",
  급여: "ok",
  근태: "warn",
};

export function notifCategoryTone(
  category: string,
): "accent" | "purple" | "info" | "ok" | "warn" | "neutral" {
  // Normalize open producer keys (approval/leave/…) through categoryLabel so a
  // category always gets the same tone as its localized chip label.
  return CATEGORY_TONE[categoryLabel(category).trim()] ?? "neutral";
}

/** Korean nav label for a screen-link group; the raw key when unregistered. */
export function notifScreenLabel(screen: string): string {
  const label = (ko.console.shell.nav as Record<string, unknown>)[screen];
  return typeof label === "string" ? label : screen;
}
