# CAP-NOTIF-CONSOLE frontend — Stage-3 fresh-eyes adversarial verification

Date: 2026-07-24 · Worktree: `console-notif-frontend-20260724` · Verifier: independent stage-3 lane (did not author the code)

Scope verified: `web/src/console/notif/**`, `web/src/i18n/notif.ts` against the dc.html 알림 풀뷰
(design mirror `docs/design/oyatie-console/"Oyatie Console.dc.html"`, lines ~4528–4554 template,
~17887–17896 view-model, ~14798–14816 click/mark-all handlers), the CAP-NOTIF-CONSOLE API
contract, and the module completion contract in `docs/program/console-enterprise-roadmap.md`.

## Verdict

PASS after 4 findings were found and fixed in this stage (see Findings). Full module suite,
type-check, lint, purity and ui-strings gates re-run green after the fixes.

## Module completion contract — point by point

1. **List/overview layer** — PASS. 시간순 timeline + 개체별 aggregation views; mute-aware unread
   summary chip + 숨김 chip; keyset paging (`before` cursor, 더 보기).
2. **Object detail layer** — PASS (surface-appropriate). Notification rows are atomic (the dc.html
   full view has no detail pane either); the detail affordance is the in-module object-filtered
   timeline drill plus resolved source-object head chips (`GET /api/objects/{kind}/{id}`).
3. **Action/workflow layer** — PASS. Per-row read toggle in both directions (`POST …/read`,
   `POST …/unread`), 모두 읽음 (`POST …/read-all`), per-object mute/unmute
   (`PUT`/`DELETE /me/notification-policies`). All server-authoritative: UI state is replaced from
   the response row, never optimistically invented.
4. **History layer** — PASS. The timeline is the chronological record; `read_at`/`resolved_at`
   ride the wire type; read-state changes are reversible and reload from the backend.
5. **≥2 upstream + ≥2 downstream traversable links** — PASS with a deliberate bound. Upstream:
   source objects of any kind via `link.type=object` (resolved head → live TokenText chip + group
   drill); producer screens via `link.type=screen`. Downstream: object drill (in-module filtered
   timeline) and exposed-screen navigation (`/console/sales`). Traversal to a source object's own
   screen is deliberately in-module because `EXPOSED_SCREEN_KEYS = ["sales"]` (ADR-0025): an
   unexposed target must not be navigable, so the row stays an ack-able non-link (tested).
6. **Server-enforced deny-by-omission without leakage** — PASS. Unauthenticated: denied state with
   zero fetches (tested). Backend 401/403 → denied state without a retry control (tested; denied ≠
   error). Head resolve failure (404 = absent or cross-tenant, indistinguishable by design) →
   plain text, never a dead link (tested). No speculative code→object resolution.
7. **Keyboard/focus/contrast/Korean-expansion/responsive** — PASS. Every control is a real
   `<button>`; stretched-primary rows are keyboard-activatable (tested via `{Enter}`);
   `:focus-visible` outlines; `aria-pressed`/`aria-label`/`aria-busy`/`role=status|alert`; token
   colors only (both themes come from `tokens.css`); header flex-wraps; ≤900px media query stacks
   row bodies. jsdom cannot assert rendered layout — visual responsive check remains a browser
   concern (open item carried over).
8. **Selection/drafts survive refresh/retry/Back** — PASS as scoped. This surface has no drafts;
   retry re-issues the load; filter/view/drill state is in-memory (deliberate skip, recorded by
   the build lane and confirmed reasonable: notification triage state is not a draft).
9. **UI grammar + truthfulness** — PASS. No captions/subtitles/meta text; status = chips; no
   big-number KPI cards (mono count chips); `className` plain string literals (purity gate, 407
   files clean); no inline Hangul in components (module ui-strings clean; copy lives in
   `web/src/i18n/notif.ts`, the exemplar mechanism); grep for
   `TODO|FIXME|XXX|HACK|.skip|.only|placeholder` over the module: zero hits; no dead controls
   (every rendered control mutates, navigates, filters, or is omitted).

## Design fidelity vs dc.html 알림 풀뷰

Zone-by-zone comparison (template line ~4529):

| Zone | dc.html | Built | Match |
|---|---|---|---|
| Page | flex column, gap 12px | `--sp-5` = 12px | ✓ |
| Header | h1 17px/800/-0.3px · mono warn unread chip 11px/800 · 전체\|미확인 ink-filled segment (3.5px 12px, r14) · spring · 모두 읽음 outline (6px 13px, r8, steel 11.5/700) | `--text-h1`/`--fw-strong`/`--tracking-tight`, same chip, same segment, `--radius` = 8px, `--text-sm`/`--fw-medium` | ✓ |
| Card | hairline border, r11, shadow, overflow hidden, overscroll-contain scroll, 14px tail | `--radius-card` = 11px, identical | ✓ |
| Row | 11px 16px, gap 10, border-soft divider, 7px signal dot (unread 1 / read **0.12**), muted category chip 9.5px/800, 12.5px/1.55 token-segment body, mono 10px faint time, whole-row activation | identical (`--sp-4` = 10px, `--text-micro` = 9.5px, `--text-xs` = 10px); dot read-opacity was 0.15 → **fixed to 0.12** (Finding 1) | ✓ after fix |
| Row weight | unread 800 / read 600 | unread `--fw-strong`(800) / read `--fw-body`(500) — console token scale has no 600; accepted token mapping | ≈ (accepted) |
| Touch swipe read-toggle | `sw.onTouchStart/Move/End` | replaced by an explicit per-row toggle button — keyboard-accessible superset of the same `notifReadToggle` behavior; swipe gesture itself not implemented | ≈ (noted) |
| Click fallback: bare `AP-\d+` regex → objectLinkGo | TokenText chips over authorized-resolved heads only; no speculative resolution (deny-by-omission) | ≈ (deliberate, safer) |

Contract-driven additions beyond the dc.html full view (개체별 view segment, 숨김 chip, per-object
mute bells with `aria-pressed`, drill filter chip, 더 보기, truthful loading/empty/denied/error
states): each is grammar-consistent (chips, token tones, no meta text) and maps to a real
CAP-NOTIF-CONSOLE route. The by-object category chip tones reuse the design's rail `catTone` map
(결재 accent · 멘션 purple · 문서 info · 급여 ok · 근태 warn · else neutral) — verified identical.

## API-module contract fidelity (field level)

- Typed routes (`/me/notifications`, `/summary`, `/{id}/read`, `/read-all`,
  `/api/objects/{kind}/{id}`) verified present in `clients/ts/src/schema.d.ts`;
  `NotificationSummary`/`NotificationLink`/`ObjectHead` fields match usage exactly
  (incl. `read_at`/`resolved_at` nullability, `ObjectHead.exists`, nullable `code|title|status`).
- The 5 contract routes not yet in the generated client (`/{id}/unread`, `/by-object`,
  `/notification-policies` GET/PUT/DELETE) go through one structural view of the same
  authenticated client; every response is boundary-validated (`isSummary`/`isGroupPage`/
  `isPolicy*`) — malformed payload → typed `NotifApiError`, never fabricated rows (tested).
- Error envelope `{error:{code,message}}` parsed with status fallback (tested at 401/500).
- Query serialization: `unread`/`before`/`limit` only-when-set (tested: `unread=true` on filter).
- N+1 discipline: head resolution deduped per link, cached per instance, capped at 24 per page,
  single parallel batch; no per-row request waterfalls.
- Principal-scoped reads send `Cache-Control: no-store, no-cache` (comms-rail rule; tested).

## Findings (all fixed in this stage)

1. **Design deviation — read-row dot opacity** (`notif.css`): built 0.15 vs dc.html full-view
   0.12. Fixed to 0.12.
2. **Label/tone inconsistency for open category keys** (`web/src/i18n/notif.ts`):
   `notifCategoryTone` keyed only on Korean literals while `categoryLabel` also localizes English
   producer keys — an `approval` group chip rendered "결재 n" with a *neutral* tone while a literal
   `결재` got *accent*. Fixed by normalizing through `categoryLabel` inside the tone lookup.
   Evidence: by-object fixture switched to `category: "approval"`; test asserts the chip reads
   "결재 2" **and** carries `var(--accent-bg)`.
3. **Aborted head-resolve batch poisoned the cache** (`NotifScreen.tsx`): links were marked
   resolved before the fetch; an abort (filter/view switch mid-load) left them marked with no head
   stored, so those codes stayed plain text for the instance lifetime. Fixed: the claims are
   released on failure so a later load retries.
4. **Stuck loading state after a mutation interrupts a load** (`NotifScreen.tsx`): `mutate`
   aborts an in-flight `load` and bumps the generation, so the aborted load's `finally` skipped
   `setLoading(false)` — `loading` stuck true, 더 보기 vanished, `aria-busy` never cleared.
   Fixed by clearing the flag at the abort site. Red-green proven: the new test
   "keeps paging available when a row action interrupts an in-flight page load" fails without the
   fix (1 failed) and passes with it.

## Verification runs (after fixes)

- `vitest run src/console/notif` → 4 files, **25/25 passed** (was 24; +1 regression test)
- `tsc -b` → clean
- `eslint src/console/notif src/i18n/notif.ts --max-warnings 0` → clean
- `node scripts/check-console-purity.mjs` → OK (407 files)
- `node scripts/check-ui-strings.mjs` → 0 hits in this module's files (branch-wide failure in
  `web/src/features/facilities/FacilitiesWorkflowPage.tsx` is another lane's pre-existing issue)
- Stub/dead-control grep (`TODO|FIXME|XXX|HACK|.skip|.only|placeholder`) → zero hits

## Honest residual gaps (carried forward, not blockers)

- Integrator wiring per `manifests/mount.json` (MOUNTED_SCREEN_KEYS + SCREEN_REGISTRY; module
  lands DARK — `EXPOSED_SCREEN_KEYS` stays `["sales"]`).
- Swap the RawClient escape hatch to typed paths after `clients/ts` regeneration.
- Group drill filters client-side over loaded pages only (the contract's list route has no
  object-filter param), so a drilled timeline may show fewer rows than the group's `total` until
  more pages load — real data, never fabricated; a server-side filter param would remove the gap.
- Touch swipe read-toggle from the design is not implemented (explicit toggle button covers the
  function, keyboard-accessible); revisit if a mobile-web pass is chartered.
- Visual/responsive layout and contrast are asserted structurally (tokens + media query), not by
  a rendered-browser check; jsdom cannot measure layout.
- comms-rail badge should adopt the mute-aware summary — rail is a LIVE codex lane, flagged in
  `mount.json`, not edited here.
