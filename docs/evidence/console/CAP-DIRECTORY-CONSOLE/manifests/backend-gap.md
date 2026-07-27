# CAP-DIRECTORY-CONSOLE — backend gaps (identity crate HOT — no backend lane; gaps only)

Existing REST reviewed: identity (`/api/v1/users*`, `/api/v1/employees*`, `/api/v1/branches`,
`/api/v1/hr/org-chart`, `/api/v1/me/authz`) and messenger (`/api/messenger/members*`). What the
directory design (design-spec.md) needs but the current contract cannot serve:

## GAP-DIR-1 — general-employee directory list is contact-poor and single-branch

`GET /api/messenger/members` (the only all-employee people read) returns `{id, display_name,
team|null}` for **one** `branch_id`, `limit ≤ 100`, no offset, no search. STORY-DIRECTORY-001 needs,
for every employee: 이름·직책(position)·소속(team + 법인/branch)·contact channels (내선/현장 code,
email/mail-to)·입사, with server-side typeahead search and pagination (design §4-27-4: production
scale 3,000명), scoped to the caller's authorized branch union (PBAC "전체 = 인가 법인 합집합").
Proposed (owner: identity or messenger crate owners' call): either extend the messenger member
DTO + add `search`/`offset`/multi-branch union, or a new `GET /api/v1/directory/members` bound to a
new general-tier feature (e.g. `directory_read` allowed for all six roles), reusing the messenger
branch-scope + deny-by-omission semantics. Fields must stay strictly non-privileged (no phone_e164,
no base_pay — the privileged detail stays on `/api/v1/employees/{id}`).

## GAP-DIR-2 — no email / extension fields anywhere

Neither `MessengerMemberSummary`, `UserSummary`, nor `Employee` carries a work email or extension
(`내선`). The design's contact-channel column and the 메일 action need an email per person (mail
module already exists). Requires schema + DTO addition on whichever surface serves GAP-DIR-1.

## GAP-DIR-3 — person card fields for the non-privileged view

`GET /api/messenger/members/{userId}` (which already has the exact person.view read-audit +
deny-by-omission semantics the design requires) returns only `{id, display_name, team}`. The person
card (design-spec §3 zones 1–3) additionally needs: position/직책, entity/법인 or home branch name,
입사(hire_date), 고용 형태, email/ext — the "전체 공개" + "팀 내 공개" categories only. Sensitive
categories (평가, 급여, 상세 정보) remain the privileged `/api/v1/employees/{id}` +
`/lifecycle-events` tier and are deny-by-omission for ordinary viewers.

## GAP-DIR-4 — directory stats

Header stats (임직원 count, 법인/branch count) need a PBAC-scoped aggregate. Derivable client-side
from a paginated list only if totals are returned (messenger members has no `total`; `EmployeePage`
does). GAP-DIR-1's response should include `total`.

## GAP-DIR-5 — team card aggregation

Team drill (design-spec §4) wants `{team name, member count, lead}` — no endpoint groups members by
team below the management-only `/api/v1/hr/org-chart`. Acceptable v1 fallback: client-side grouping
of the GAP-DIR-1 list within the fetched scope; org-wide team cards stay management-tier.

## Non-gaps (already served)

- person.view read-audit, self-view exemption, no-audit-on-denied: messenger `member_profile`
  (transactional audit) — reuse, do not reimplement.
- Branch names for org placement: `GET /api/v1/branches` (any authenticated user).
- Thread creation for 메시지 action: `POST /api/messenger/threads`.
- Privileged HR directory/detail/lifecycle: `/api/v1/employees*` with `EmployeeDirectoryRead/Manage`.

# Shared-root integration manifest (consolidation integrator applies; JSON sibling: shared-roots.json)

1. `web/src/console/shell/nav.ts`: append `"directory"` to `MOUNTED_SCREEN_KEYS` (nav item already
   exists in the comms group, ungated; `EXPOSED_SCREEN_KEYS` untouched — DARK until approved).
2. `web/src/console/screens/registry.ts`: `directory: DirectoryScreenBody` (import from
   `../directory`).
3. `web/src/i18n/ko.ts`: no change (console.shell.nav.directory already present).
4. `backend/openapi/**` + `clients/**`: only if a GAP-DIR endpoint lands (per-domain `tags:`
   required on every operation — client-generation constraint).
