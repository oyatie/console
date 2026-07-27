# CAP-BOARD-CONSOLE frontend — STAGE 3 fresh-eyes adversarial verification

Verifier: independent pass over the actual code on `claude/console-board-frontend-20260724`
(build commits `49bceb76`/`5405acdf`/`f8445781`), design authority re-extracted from
`docs/design/oyatie-console/"Oyatie Console.dc.html"` (change-log 190), API contract
cross-checked against the backend lane's sync-point fragment
(`console-board-backend-20260724:docs/evidence/console/CAP-BOARD-CONSOLE/manifests/openapi-fragment.yaml`).

## Verdict

Verified with 2 findings, both FIXED in this pass (commit below). Module status stays
**partial** on the same honestly-named contract gaps the build report already carried
(traversable-link count, generic-ModuleScreen shared-root gaps) — nothing newly hidden.

## Findings (fixed)

| # | Class | Finding | Fix |
|---|-------|---------|-----|
| F1 | keyboard/focus (module contract pt. 8) | `BoardReceiptsPanel` never moved focus into the dialog on open. The Escape handler lives on `.board-overlay`; the opener button (detail panel) is not an ancestor, so Escape could not close the drill until the user first clicked inside. Neither overlay restored focus to its opener on close (focus fell to `<body>`). `BoardComposer` only worked by accident of `autoFocus`. | Receipts panel: `tabIndex={-1}` + capture-opener/focus-panel/restore effect. Composer: replaced `autoFocus` with an explicit capture-opener → focus-title → restore effect (`autoFocus` fires at commit, before effects — the opener would have been captured as the already-focused title input). Tests: Escape-straight-after-open closes both dialogs and `document.activeElement` returns to the opener. |
| F2 | design fidelity (search) | Prototype MOD search haystack is `code + c1 + c2 + c3 + st + en` (dc.html:16051). Built haystack omitted the status-chip label (`st`) and published-day label (`c2`) — searching "초안"/"수령확인 중"/"오늘" found nothing. | Haystack now includes `noticeStatusChip(row).label` and `publishedDayLabel(row.published_at)` (body kept — richer than the mock). Config-level haystack test + screen-level test (typing 초안 narrows to the draft row). |

## 9-point module completion contract

| Point | Status | Evidence |
|---|---|---|
| List/overview layer | PASS | ModuleScreen list via `buildBoardModuleConfig`: 코드(mono)/공지/게시/대상/확인 columns — exactly the prototype's `["코드", ...cols]` track (dc.html:16100 `onSheetOpen`); statbar 게시 중 / 수령확인 진행(warn) / manager-only 초안. |
| Object detail layer | PASS | Generic detail panel: kv (유형/게시/대상/수령확인/내 수령확인), branch link chips, state-gated actions. |
| Action/workflow layer | PASS | draft→publish one-way FSM: compose (create draft), 초안 편집 (PATCH), 게시 (publish, 409/422 envelope surfaced), 수령확인 (ack, recipient-row-gated, reconciles from reloaded list). All real mutations; no dead controls. |
| History layer | PASS | Manager receipts drill = 직원 1:N receipt history (per-recipient acknowledged_at timestamps, 전체/확인/미확인 chase filter, 50-page 더보기); member's own history = 내 수령확인 kv. |
| ≥2 upstream + ≥2 downstream links | PARTIAL | Downstream: audience-branch chips (list drill w/ dismissible filter chip) + receipts drill (수령 대상 직원 1:N per ontology OT-19). Upstream: author link omitted (contract carries `author_user_id` only — a UUID chip would violate self-explanatory UI); 기록물 등재 1:1 out of scope (no records module). Named in open_items since the build report; unchanged. |
| Server-enforced deny-by-omission, no leakage | PASS | `notice_manage` via canonical authz projection (`/api/v1/me/authz`, jwt-floor fail-closed while loading); manager affordances absent (not disabled) without it — asserted (compose, drafts stat, publish/receipts absent for members). Backend is sole enforcer (drafts 404/list-omission per design-contract §3); ack advisory-allowed in UI but row-gated on `my_receipt` and principal-bound server-side (404 for non-recipients). |
| Keyboard/focus/contrast/Korean-expansion/responsive | PASS (after F1) | J/K/Enter grid nav asserted; dialogs: role=dialog + aria-modal + labelledby, focus moved in on open, Escape closes, focus restored to opener; token colors only (chips ok/warn/info/neutral, prog ok@100%/warn); column minWidths + ellipsis for Korean expansion; statbar overflow-x + grid overscroll containment asserted; ≤640px full-width panel. Real-viewport visual pass remains exposure-stage evidence (jsdom limit). |
| Selection + drafts survive refresh/retry/Back | PARTIAL | Composer fields survive remount via sessionStorage scoped `org:user:draftId` (asserted); server drafts are the durable copy; retry keeps ModuleScreen state. Row selection is component state — does not survive a full page refresh (shared-root gap: shell URL params; recorded in mount.json). |
| Truthfulness | PASS | No TODO/FIXME/skip/only/stubs (scan clean); every datum from the typed contract; unknown category/status → truthful "확인 필요" labels; member without progress sees 게시 (never a fabricated 완료); empty/error/denied/loading states all real and tested. |

## Design fidelity vs dc.html board section (11138–11143, 16047–16290, 13794)

Matches: title 게시판·공지; action label 공지 작성; column set + leading mono 코드 col; stats
(게시 중, 수령확인 진행 warn; drafts extra is manager-only and truthful); chip states 게시=ok,
수령확인 중=warn, 완료=neutral (bd4), 초안=info (no prototype exemplar — draft-authoring rows
use info); completion derived `acknowledged ≥ total` (design analytics `수령률 = 확인 ÷ 대상`);
게시 col renders 오늘/어제/M/D exactly; 대상 renders 전 직원 / branch names; categories
안내/법정 통지/인사명령/교육 = the four `en.유형` values; per-notice prog bar formula + label
`done / total (pct%)` with ok@100%/warn (receipts drill); search haystack (after F2).

Known deviations (all recorded, none silent):
- Prototype's 공지 작성 opens a mail composer (`mailTo: 전 직원`); real module opens the
  draft/scope composer per the backend contract — mail 병행 발송 is out of scope (mount.json).
- Prototype detail renders the prog bar inline; generic ModuleScreen detail has no prog field —
  bar renders in the board-owned receipts drill, kv text in generic detail (shared-root gap).
- Prototype `en` values (유형) drill the list on click; generic detail kv values are not
  clickable (shared-root gap — would need a kv `onGo` extension).
- Prototype MOD header columns click-sort and rows are drag-sources (`[NT-0707 …]` payload);
  the generic ModuleScreen has neither (program-wide shared-root gaps, all modules).
- Prototype board stat entries are NOT drills either (`onGo` no-ops without `scr`) — the built
  static statbar is not a fidelity regression; the "stat=drill" note in mount.json applies to
  user-added custom stats only.
- Simulated cross-module links (발송 메일/연차 촉진/개인 수신함/컴플라이언스/기록물) have no real
  contract counterpart; only audience branches are truthful links today.

## API contract fidelity (field-level, vs backend-lane openapi fragment)

- `BoardNotice` = generated `NoticeSummary` + `category` (enum 4), `audience_scope`
  (org|branches), `audience_branches[{id,name}]`, `my_receipt{acknowledged_at|null}|null`,
  `progress|null` — field names/shapes identical to the fragment's replaced `NoticeSummary`.
- `CreateNoticeDraftInput`/`UpdateNoticeDraftInput`/`NoticeAudienceInput` match (audience
  replace-whole on PATCH honored: composer always sends the full audience).
- Receipts query `acknowledged?/limit/offset` matches (limit 50 ≤ max 200; `acknowledged`
  omitted for 전체 — asserted at the transport boundary); no repeated-query params in module.
- Error envelope `{error:{code,message}}` (= `ErrorBody`) parsed in one place (`message()`);
  409/422 server messages surfaced verbatim (publish-conflict test).
- ack 204 handled without a body; no N+1 (list embeds progress; drill = 2 parallel calls).
- Contract-ahead raw GET/PATCH casts are ponytail-marked with the retirement path (client
  regen at integration) — mount.json documents the exact drop list.

## Gates (final state)

- `npx vitest run src/console/board` — 2 files, **25/25 passed** (21 + 4 added this pass)
- `npx tsc -b` — clean
- `npx eslint src/console/board src/i18n/board.ts --max-warnings 0` — clean
- `node scripts/check-console-purity.mjs` — 406 files clean
- `node scripts/check-ui-strings.mjs` — zero board hits; fails ONLY on pre-existing
  `src/features/facilities/FacilitiesWorkflowPage.tsx` (commit `fd93fbdd`, outside lane roots)
- Stub scan (TODO/FIXME/skip/only) — clean
