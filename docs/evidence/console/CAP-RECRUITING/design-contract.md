# CAP-RECRUITING — Design Contract (API · DTO · FSM · DDL 0187-provisional)

> Feeds the backend build (crate `backend/crates/recruiting`) and the frontend build
> (`web/src/console/recruiting`). Route: `/console/recruiting`, screen key `recruit`.
> STORY-RECRUITING-001: posting publishes through preflight → applicants advance through
> evaluation to offer → acceptance creates the employee object with full audit lineage.

## 1. State machines

### 1.1 Posting (JP-)
```
DRAFT ──publish(preflight ✓ + exposure attest)──▶ PUBLISHED ──close──▶ CLOSED
```
- No hard delete; CLOSED rows retained (§3.9 archive-not-delete). Draft is editable in place
  (§3.9.0-③); PUBLISHED fields are immutable except `close` and hire-driven `hired_count`.
- Preflight (server-side, atomic with publish; §4-29):
  `role_defined` (role_title+worksite non-empty), `quota_defined` (headcount≥1, deadline set or
  open-ended), `no_duplicate_open` (no other PUBLISHED posting, same org, same role_title),
  plus `exposure_attested` = caller's explicit attest flag (recorded actor+ts). Any unmet
  auto-check or missing attest ⇒ 422 with the full check vector (fail-closed).
- `close` allowed from PUBLISHED any time (충원 완료 review is advisory).

### 1.2 Applicant (APL-)
```
stage:   APPLIED ─▶ SCREENING ─▶ INTERVIEW ─▶ OFFER ─▶ HIRED (terminal)
flags:   hold(bool, toggle)   doc_requested(bool)
reject:  any non-HIRED stage ─reject(reason enum)─▶ rejected (talent pool) ─reinstate─▶ prior stage
```
- `advance` moves exactly one stage; INTERVIEW→OFFER happens **only** via offer extension
  (guard: assessment recorded — fail-closed 422 `ASSESSMENT_REQUIRED`).
- `reject` requires `reason ∈ {CAREER_SHORTFALL, ROLE_MISMATCH, COMP_MISMATCH, ACCEPTED_ELSEWHERE, OTHER}`
  (+ optional note ≤500); archives to talent pool; reversible via `reinstate` (history preserved
  in stage events).
- HIRED is set exclusively by the hire handshake (§3 below), never by `advance`.
- All stage/flag transitions append a `recruit_stage_events` row AND a platform audit event.

### 1.3 Offer (versioned, per applicant)
```
EXTENDED ─adjust─▶ (v+1 EXTENDED, prior row → SUPERSEDED)
EXTENDED ─withdraw(reason)─▶ WITHDRAWN  (applicant stage → INTERVIEW)
EXTENDED ─record-reply─▶ ACCEPTED | DECLINED
```
- Extend guard: applicant at INTERVIEW (or OFFER for re-extend after withdraw+re-advance),
  assessment recorded. Extension sets applicant stage → OFFER.
- Adjust = immutable v+1 row (`기존 조건 이력 보존`); at most one non-terminal offer per applicant
  (partial unique index).
- DECLINED leaves the applicant at OFFER for recruiter decision (reject or re-offer).
- Hire guard: latest offer `ACCEPTED`.

## 2. REST API (crate `recruiting/rest`, mounted in `build_router`; openapi tag `recruiting`)

Conventions: authenticated console principal; org from principal (RLS-armed via request-context
middleware like `hr::router`); errors `{error:{message}}`; 403 without existence leakage (org-
scoped lookups return 404 for other-org ids); every mutation runs in `with_audits`.

| # | Method + path | Gate (Feature) | Request body (DTO) | Response |
|---|---|---|---|---|
| 1 | `POST /api/v1/recruiting/postings` | RecruitingManage | `CreateRecruitPostingRequest { role_title, company, worksite, employment_type, scope, headcount, deadline?, requirements[], position_ref? }` | 201 `RecruitPostingResponse` (status DRAFT, posting_no JP-…) |
| 2 | `GET /api/v1/recruiting/postings?status=&scope=` | RecruitingRead | — | `RecruitPostingListResponse { items: [RecruitPostingSummary { …, stage_counts{applied,screening,interview,offer}, hired_count, headcount }] }` |
| 3 | `GET /api/v1/recruiting/postings/{postingId}` | RecruitingRead | — | `RecruitPostingDetailResponse { posting, applicants: [RecruitApplicantSummary] }` |
| 4 | `PUT /api/v1/recruiting/postings/{postingId}` | RecruitingManage | same as create + `expected_updated_at` | 200 (DRAFT only; 409 otherwise) |
| 5 | `POST /api/v1/recruiting/postings/{postingId}/preflight` | RecruitingManage | — | `PostingPreflightResponse { checks: [{key, ok, note}], publishable }` (read-only evaluation) |
| 6 | `POST /api/v1/recruiting/postings/{postingId}/publish` | RecruitingManage | `PublishRecruitPostingRequest { attest_exposure_scope: bool, expected_updated_at }` | 200 posting; 422 `{checks…}` when gate unmet |
| 7 | `POST /api/v1/recruiting/postings/{postingId}/close` | RecruitingManage | `{ expected_updated_at }` | 200 posting |
| 8 | `POST /api/v1/recruiting/postings/{postingId}/applicants` | RecruitingManage | `CreateRecruitApplicantRequest { name, profile_lines[], source_document? }` | 201 `RecruitApplicantResponse` (stage APPLIED, applicant_no APL-…) |
| 9 | `GET /api/v1/recruiting/applicants/{applicantId}` | RecruitingRead | — | `RecruitApplicantDetailResponse { applicant, offers[], events[] }` — **server logs an audited PII view** (`recruiting.applicant.view`) |
| 10 | `POST /api/v1/recruiting/applicants/{applicantId}/advance` | RecruitingManage | `{ expected_updated_at }` | 200 applicant; 422 on INTERVIEW→OFFER attempt (offer-only) |
| 11 | `POST /api/v1/recruiting/applicants/{applicantId}/assess` | RecruitingManage | `{ score: SUITABLE\|NEUTRAL\|UNSUITABLE }` | 200 applicant (assessment {score, by, at}) |
| 12 | `POST /api/v1/recruiting/applicants/{applicantId}/hold` | RecruitingManage | `{ hold: bool }` | 200 applicant |
| 13 | `POST /api/v1/recruiting/applicants/{applicantId}/request-documents` | RecruitingManage | — | 200 applicant (doc_requested) |
| 14 | `POST /api/v1/recruiting/applicants/{applicantId}/reject` | RecruitingManage | `{ reason: enum, note? }` | 200 applicant (rejected, talent-pool row) |
| 15 | `POST /api/v1/recruiting/applicants/{applicantId}/reinstate` | RecruitingManage | — | 200 applicant |
| 16 | `POST /api/v1/recruiting/applicants/{applicantId}/offer` | RecruitingManage | `ExtendRecruitOfferRequest { amount, amount_period: MONTHLY\|DAILY, reply_deadline }` | 201 `RecruitOfferResponse`; 422 `ASSESSMENT_REQUIRED` |
| 17 | `POST /api/v1/recruiting/offers/{offerId}/adjust` | RecruitingManage | `{ amount, reply_deadline? }` | 201 offer v+1 |
| 18 | `POST /api/v1/recruiting/offers/{offerId}/withdraw` | RecruitingManage | `{ reason }` | 200 (applicant back to INTERVIEW) |
| 19 | `POST /api/v1/recruiting/offers/{offerId}/record-reply` | RecruitingManage | `{ decision: ACCEPTED\|DECLINED }` | 200 offer |
| 20 | `POST /api/v1/recruiting/applicants/{applicantId}/hire` | RecruitingManage **+ EmployeeDirectoryManage** | `HireRecruitApplicantRequest { employee_number, phone, org_unit, position, site, home_branch_id, base_pay }` | 201 `HireResponse { employee_id, applicant, posting {hired_count, headcount} }` |
| 21 | `GET /api/v1/recruiting/talent-pool` | RecruitingRead | — | `TalentPoolListResponse { items: [{applicant_no, name, role_title, reason, rejected_at}] }` |

DTO field notes: `amount` = string decimal (KRW, 2dp — matches hr `base_pay: String`);
`profile_lines` = ordered structured bullets (§4-13 — primary surface; `source_document` is
provenance only); timestamps RFC3339; list responses `{ items }`; optimistic concurrency via
`expected_updated_at` on posting/applicant mutations (hr `SetEmployeeHomeBranchRequest`
precedent).

Authz matrix additions (`platform/authz`): `RecruitingRead = [D,D,D,A,A,A]` (`recruiting_read`),
`RecruitingManage = [D,D,D,A,D,A]` (`recruiting_manage`) — column order
[MEMBER, RECEPTIONIST, MECHANIC, ADMIN, EXECUTIVE, SUPER_ADMIN].

## 3. Hire handshake (the one rule that cannot bend)

Acceptance → employee creation goes **through the owning HR use-case**:

1. `hire` handler validates: applicant stage OFFER, latest offer ACCEPTED, not already hired
   (409 with existing `employee_id` — idempotent replay), posting not CLOSED.
2. In ONE transaction (`with_audits`): call the extracted hr core
   (`create_employee_core` in `backend/app/src/hr.rs` — same normalization, same
   `employee_create_idempotency` reservation, same `employees` + `employee_employment_profiles`
   inserts, same `employee.create` audit) with
   `idempotency_key = "recruit-hire-" + applicant_id`, `company` from posting,
   `employment_type` mapped from the posting enum, `base_pay` defaulted from the accepted
   offer amount (caller may override).
3. Same transaction: set applicant `stage = HIRED`, `hired_employee_id = employee_id`;
   increment `recruit_postings.hired_count` (DB check `hired_count <= headcount`); append stage
   event `OFFER → HIRED`; audit `recruiting.applicant.hire` with linked employee id — audit
   lineage: `공고 JP-* → 지원자 APL-* → employee.create → 직원`.
4. `employment_type = POOL_DAILY` postings: hire returns 422 `POOL_REGISTRATION_UNAVAILABLE`
   (workforce-pool registry backend does not exist yet — gap-analysis §4; never create an
   employee for a pool posting, per design: 재직 명부 비합산).
5. Recruiting code never writes `employees`/`employee_employment_profiles` directly, and never
   issues a second audit `employee.create`.

## 4. DDL — provisional migration `0187_create_recruiting.sql`

> 0180 is the highest committed migration; renumber to the next free number right before push.
> House style per 0063/0172: org RLS (`app.current_org`), FORCE RLS, `console_rt` grants,
> composite same-org FKs, timestamptz defaults.

```sql
-- Recruiting pipeline: postings → applicants → offers, hire links into employees.
CREATE TABLE recruit_postings (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    posting_no      TEXT        NOT NULL CHECK (posting_no ~ '^JP-[0-9]{4,}$'),
    role_title      TEXT        NOT NULL CHECK (btrim(role_title) <> ''),
    company         TEXT        NOT NULL CHECK (btrim(company) <> ''),
    worksite        TEXT        NOT NULL CHECK (btrim(worksite) <> ''),
    employment_type TEXT        NOT NULL CHECK (employment_type IN ('REGULAR','RESIDENT_SHIFT','PART_TIME','POOL_DAILY')),
    scope           TEXT        NOT NULL CHECK (scope IN ('INTERNAL','EXTERNAL')),
    headcount       INTEGER     NOT NULL CHECK (headcount >= 1),
    hired_count     INTEGER     NOT NULL DEFAULT 0 CHECK (hired_count >= 0 AND hired_count <= headcount),
    deadline        DATE,                          -- NULL = 상시
    requirements    JSONB       NOT NULL DEFAULT '[]'::jsonb CHECK (jsonb_typeof(requirements) = 'array'),
    position_ref    TEXT,                          -- optional ontology position instance ref
    status          TEXT        NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT','PUBLISHED','CLOSED')),
    exposure_attested_by UUID,
    exposure_attested_at TIMESTAMPTZ,
    published_by    UUID,
    published_at    TIMESTAMPTZ,
    closed_at       TIMESTAMPTZ,
    created_by      UUID        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, posting_no),
    UNIQUE (id, org_id),
    FOREIGN KEY (created_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT,
    CHECK (status <> 'PUBLISHED' OR (published_at IS NOT NULL AND exposure_attested_at IS NOT NULL))
);
CREATE INDEX recruit_postings_org_status_idx ON recruit_postings (org_id, status);

CREATE TABLE recruit_applicants (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    posting_id      UUID        NOT NULL,
    applicant_no    TEXT        NOT NULL CHECK (applicant_no ~ '^APL-[0-9]{4,}$'),
    name            TEXT        NOT NULL CHECK (btrim(name) <> ''),
    profile         JSONB       NOT NULL DEFAULT '[]'::jsonb CHECK (jsonb_typeof(profile) = 'array'),
    source_document TEXT,                          -- provenance filename only (§4-13)
    stage           TEXT        NOT NULL DEFAULT 'APPLIED' CHECK (stage IN ('APPLIED','SCREENING','INTERVIEW','OFFER','HIRED')),
    hold            BOOLEAN     NOT NULL DEFAULT FALSE,
    doc_requested   BOOLEAN     NOT NULL DEFAULT FALSE,
    rejected_at     TIMESTAMPTZ,
    reject_reason   TEXT        CHECK (reject_reason IN ('CAREER_SHORTFALL','ROLE_MISMATCH','COMP_MISMATCH','ACCEPTED_ELSEWHERE','OTHER')),
    reject_note     TEXT,
    assessment_score TEXT       CHECK (assessment_score IN ('SUITABLE','NEUTRAL','UNSUITABLE')),
    assessed_by     UUID,
    assessed_at     TIMESTAMPTZ,
    hired_employee_id UUID,
    created_by      UUID        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, applicant_no),
    UNIQUE (id, org_id),
    FOREIGN KEY (posting_id, org_id) REFERENCES recruit_postings(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (hired_employee_id, org_id) REFERENCES employees(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (created_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT,
    CHECK ((rejected_at IS NULL) = (reject_reason IS NULL)),
    CHECK (stage <> 'HIRED' OR hired_employee_id IS NOT NULL),
    CHECK ((assessment_score IS NULL) = (assessed_by IS NULL))
);
CREATE INDEX recruit_applicants_org_posting_idx ON recruit_applicants (org_id, posting_id);
CREATE INDEX recruit_applicants_org_rejected_idx ON recruit_applicants (org_id, rejected_at) WHERE rejected_at IS NOT NULL;
CREATE UNIQUE INDEX recruit_applicants_hired_employee_uq ON recruit_applicants (org_id, hired_employee_id) WHERE hired_employee_id IS NOT NULL;

CREATE TABLE recruit_offers (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    applicant_id    UUID        NOT NULL,
    version         INTEGER     NOT NULL CHECK (version >= 1),
    amount          NUMERIC(14,2) NOT NULL CHECK (amount >= 0),
    amount_period   TEXT        NOT NULL CHECK (amount_period IN ('MONTHLY','DAILY')),
    currency        TEXT        NOT NULL DEFAULT 'KRW' CHECK (currency = 'KRW'),
    reply_deadline  DATE        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'EXTENDED' CHECK (status IN ('EXTENDED','SUPERSEDED','WITHDRAWN','ACCEPTED','DECLINED')),
    withdraw_reason TEXT,
    extended_by     UUID        NOT NULL,
    extended_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at     TIMESTAMPTZ,
    UNIQUE (org_id, applicant_id, version),
    UNIQUE (id, org_id),
    FOREIGN KEY (applicant_id, org_id) REFERENCES recruit_applicants(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (extended_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT,
    CHECK (status <> 'WITHDRAWN' OR withdraw_reason IS NOT NULL)
);
-- One live offer per applicant; history rows are terminal-status.
CREATE UNIQUE INDEX recruit_offers_live_uq ON recruit_offers (org_id, applicant_id) WHERE status = 'EXTENDED';

-- Domain history layer for the applicant timeline (platform audit_events remain the
-- tamper-evident stream; this table serves the UI history without audit-read privileges).
CREATE TABLE recruit_stage_events (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    applicant_id    UUID        NOT NULL,
    action          TEXT        NOT NULL CHECK (action IN ('APPLY','ADVANCE','ASSESS','HOLD','UNHOLD','REQUEST_DOCUMENTS','OFFER_EXTEND','OFFER_ADJUST','OFFER_WITHDRAW','OFFER_REPLY','REJECT','REINSTATE','HIRE')),
    from_stage      TEXT,
    to_stage        TEXT,
    reason          TEXT,
    actor           UUID        NOT NULL,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (applicant_id, org_id) REFERENCES recruit_applicants(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (actor, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT
);
CREATE INDEX recruit_stage_events_org_applicant_idx ON recruit_stage_events (org_id, applicant_id, occurred_at);

-- RLS + grants (every table, house pattern):
--   ALTER TABLE t ENABLE ROW LEVEL SECURITY; ALTER TABLE t FORCE ROW LEVEL SECURITY;
--   CREATE POLICY org_isolation ON t
--     USING (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid)
--     WITH CHECK (org_id = NULLIF(current_setting('app.current_org', true), '')::uuid);
--   GRANT SELECT, INSERT, UPDATE ON t TO console_rt;           -- no DELETE (archive-not-delete)
--   (recruit_stage_events: GRANT SELECT, INSERT only — append-only.)
```

Prerequisite check for the DDL: `employees` needs `UNIQUE (id, org_id)` and `users` needs
`UNIQUE (id, org_id)` — both already exist (0172 composite FKs reference them).
`posting_no`/`applicant_no` are allocated from per-org sequences in the adapter
(`JP-` / `APL-` + zero-padded counter), not DB sequences, mirroring house code style.

## 5. Audit action vocabulary (platform audit via `with_audits`)

`recruiting.posting.create` · `recruiting.posting.update` · `recruiting.posting.publish`
(snapshot: preflight check vector + attest) · `recruiting.posting.close` ·
`recruiting.applicant.create` · `recruiting.applicant.view` (PII view — GET #9 only) ·
`recruiting.applicant.advance` · `recruiting.applicant.assess` · `recruiting.applicant.hold` ·
`recruiting.applicant.request_documents` · `recruiting.applicant.reject` (reason snapshot) ·
`recruiting.applicant.reinstate` · `recruiting.offer.extend` · `recruiting.offer.adjust` ·
`recruiting.offer.withdraw` · `recruiting.offer.reply` · `recruiting.applicant.hire`
(+ the reused `employee.create` from the hr core — two events, one transaction, linked ids).

## 6. Frontend build contract (`web/src/console/recruiting/`)

- Files: `index.ts`, `routeContract.ts` (`RecruitingRouteContract {}` structural fixture),
  `useRecruitingConsoleAuthz.ts` (jwtFloor → fetchAuthzProjection → makePolicyGate),
  `recruitingCapabilities.ts`, `recruitingApi.ts`, `RecruitingScreen.tsx` +
  `RecruitingScreenBody` (prop-less registry export), `recruiting.css`, tests per file,
  `web/src/i18n/recruiting.ts` (module-owned Korean strings).
- Capabilities from feature grants (never roles):
  `canRead = recruiting_read | recruiting_manage`; `canManage/canPublish/canAdvance/canOffer =
  recruiting_manage`; `canHire = recruiting_manage && employee_directory_manage`. Recruiting is
  org-wide — gate query uses the org-wide branch scope the projection carries (no branchId
  prop; follow `authorize_hr_org_wide` semantics).
- Screen layers (module completion contract):
  1. **List/overview** — posting table (columns/chips/fill-bar/stage-counts per design-spec §2B),
     compact stat bar from list aggregates, J/K/Enter + row accordion.
  2. **Object detail** — applicant card (stepper, profile+provenance chip, scorecard, offer box,
     enum reject menu, fail-closed error surface) + posting detail.
  3. **Action/workflow** — composer modal (typed fields, draft/publish), preflight gate modal
     (checks + attest), advance/hold/doc/reject/reinstate/offer/hire actions — each disabled
     absent capability, each reconciled from the server response.
  4. **History** — applicant timeline from `events[]` (#9) rendered as the history layer.
  - ≥2 upstream links: posting → position_ref (explore), applicant → posting; ≥2 downstream:
    hire → employee (people screen), reject → talent pool list.
- Session fencing, abort/generation guards, denied-before-fetch, retry alert, server
  reconciliation, plain string classNames, no inline Hangul, token colors, no explanatory
  captions (header line = live counts) — all per the production exemplar conventions
  (gap-analysis §3).
- Registration entries (nav MOUNTED_SCREEN_KEYS + SCREEN_REGISTRY + openapi/clients) are
  integrator-owned — see `integration-manifest.json`. Module lands DARK
  (`EXPOSED_SCREEN_KEYS` unchanged).
