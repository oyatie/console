/**
 * 고객·현장 (field) console module strings. Module-owned i18n resource — the
 * same mechanism as `production.ts`. Ticket-domain labels (status, priority,
 * category, origin, comments, transitions) are NOT duplicated here: the module
 * reuses `ko.support` via `features/support/support-format` (§4-18).
 *
 * Design anchor: dc.html MOD_SCREENS.field (고객·현장 — cols 현장·고객·계약·SLA,
 * stats SLA 위반·진행 이슈·상주 현장). The 계약 column and 상주 headcount are
 * simulated business data with no backing field in the FieldSiteRow contract,
 * so this module renders the truthful columns (현장·고객·이슈·작업·SLA) and a
 * total-sites stat instead of fabricating them.
 */
export const fieldStrings = {
  title: "고객·현장",
  denied: "고객·현장 화면을 볼 권한이 없습니다.",
  loading: "현장 목록을 불러오는 중입니다.",
  loadError: "현장 목록을 불러오지 못했습니다.",
  retry: "다시 시도",
  actionError: "요청을 처리하지 못했습니다. 잠시 후 다시 시도하세요.",
  empty: "표시할 현장이 없습니다.",
  emptyFiltered: "조건에 맞는 현장이 없습니다.",
  clearFilters: "필터 해제",
  searchLabel: "현장·고객·주소 검색",
  intake: "이슈 접수",
  listLabel: "현장 목록",
  statsLabel: "현장 통계 필터",
  stats: {
    breached: "SLA 위반",
    openIssues: "진행 이슈",
    total: "현장",
  },
  cols: {
    site: "현장",
    customer: "고객",
    load: "이슈·작업",
    sla: "SLA",
  },
  slaFilterLabel: "SLA 필터",
  allFilter: "전체",
  sla: {
    OK: "정상",
    AT_RISK: "임박",
    BREACHED: "위반",
  },
  issueCount: (count: number) => `이슈 ${String(count)}`,
  workOrderCount: (count: number) => `작업 ${String(count)}`,
  detail: {
    label: "현장 상세",
    loading: "현장 상세를 불러오는 중입니다.",
    select: "목록에서 현장을 선택하세요.",
    absent: "권한 범위에 없는 현장입니다.",
    loadError: "현장 상세를 불러오지 못했습니다.",
    customer: "고객",
    address: "주소",
    contact: "담당",
    geofence: "지오펜스",
    nextDue: "다음 기한",
    sla90d: "SLA 90일",
    sla90dValue: (within: number, breached: number) =>
      `준수 ${String(within)} · 위반 ${String(breached)}`,
    meters: (radius: number) => `${String(radius)}m`,
    filterByCustomer: (name: string) => `고객 ${name} 현장만 보기`,
    searchContact: (name: string) => `담당 ${name} 검색`,
    tickets: "이슈",
    workOrders: "작업 지시",
    attendance: "근태",
    acceptances: "고객 확인",
    sectionEmpty: "없음",
    back: "현장 상세로",
  },
  ticket: {
    label: "이슈 상세",
    loadError: "이슈 상세를 불러오지 못했습니다.",
    open: (title: string) => `이슈 ${title} 열기`,
    linkSite: "이 현장에 연결",
    linking: "연결 중",
    linkFailed: "현장 연결에 실패했습니다.",
  },
  workOrder: {
    open: (code: string) => `작업 지시 ${code} 배차 화면 열기`,
    reportSubmitted: "보고 제출",
  },
  attendanceKind: {
    ARRIVAL: "도착",
    DEPARTURE: "철수",
  },
  acceptance: {
    record: "확인 기록",
    recording: "기록 중",
    kind: "구분",
    kinds: {
      CUSTOMER_ACCEPTED: "고객 인수",
      CUSTOMER_DECLINED: "인수 거절",
    },
    channel: "채널",
    channels: {
      IN_PERSON: "대면",
      PHONE: "전화",
      EMAIL: "이메일",
      MESSENGER: "메신저",
    },
    acceptedBy: "확인자",
    acceptedByPlaceholder: "고객 측 확인자 성명",
    note: "메모",
    failed: "고객 확인을 기록하지 못했습니다.",
  },
  intakeForm: {
    site: "대상 현장",
    noSite: "현장 미지정",
    submit: "접수",
    submitting: "접수 중",
    failed: "이슈를 접수하지 못했습니다.",
  },
} as const;
