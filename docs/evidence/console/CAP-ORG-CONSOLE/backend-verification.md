# CAP-ORG-CONSOLE — Stage-3 Fresh-Eyes Backend Verification

Date: 2026-07-24 · Verifier: independent stage-3 lane (did not write the code)
Scope: `backend/crates/orgchange/{domain,adapter-postgres,rest}`, migration
`0189_create_org_change.sql`, `backend/app/tests/org_change_api.rs`, router mount in
`backend/app/src/lib.rs`, lane manifests.

## Verdict

GO with three findings, all fixed in this stage (commits below). No security finding
survived. Honest gaps are listed at the end — they are follow-ups, not defects.

## What was verified against the actual code (not the build report)

### Tenant isolation / RLS
- Migration 0189: all four tables (`org_change_requests`, `org_change_approval_steps`,
  `org_change_settlement_items`, `org_change_events`) get ENABLE + FORCE ROW LEVEL
  SECURITY and the standard `org_isolation` USING/WITH CHECK policy on
  `app.current_org`, plus `enforce_org_id_immutable` triggers. Events additionally get
  the 0153 `governance_append_only_record` UPDATE/DELETE triggers and a
  `REVOKE UPDATE, DELETE … FROM mnt_rt`.
- Every mutation runs inside `with_audits(pool, org, …)` and every read inside
  `with_org_conn`, both of which arm the transaction-local GUC **before** the closure
  runs (verified in `platform/db/src/audit_tx.rs`). No orgchange query runs on a bare
  pool except `org_entities`, which uses only the identity-scoped SECURITY DEFINER
  resolvers (`group_role_grants_for_user`, `group_member_org_ids`) plus the
  column-granted, non-tenant `groups` metadata — verified in migration 0060 that
  `groups` has no RLS by design (platform table, `GRANT SELECT (id, slug, name,
  status)` only) and that both resolvers authorize by actor and are EXECUTE-granted to
  `mnt_rt`.
- `apply_op` mutates `regions`/`branches`/`registry_sites`/`employees` with bare
  `WHERE id = $1` **inside the armed transaction**; verified migrations 0030/0035/0063
  FORCE RLS + `org_isolation` on every one of those tables, so a cross-tenant UUID in
  a proposal op resolves to zero rows → conflict, never a cross-tenant write.

### Runtime-role integration tests
- `org_change_api.rs` builds a second pool with `after_connect → SET ROLE mnt_rt` and
  routes **all** HTTP through `build_router` on that pool; the superuser pool is used
  only for seeding and direct-SQL readback assertions. Confirmed the router receives
  the `mnt_rt` pool (the `send` helper builds `AppState` from the pool passed in, and
  every call passes `&rt`).
- Count-leak-free isolation is proven: outsider ADMIN of a second org gets detail 404
  (same shape as absent), `total: 0` list, 404 on mutation attempts — not filtered
  rows, not 403.

### Authorization
- Deny-by-default: every handler resolves the principal then checks an explicit role
  floor before touching the store; no route is floor-less. Floors byte-match the scout
  matrices (read/draft = ADMIN|EXECUTIVE|SUPERADMIN, approve/apply =
  EXECUTIVE|SUPERADMIN). 401 without bearer, 403 with the canonical envelope and zero
  data for floored roles — both test-proven.
- Feature keys are seeded in `feature_catalog` but the shared `Feature` enum is
  integrator-owned; until the variants land, unknown keys are skipped fail-closed by
  `Feature::from_str`, so the role floors are the complete surface (module doc states
  this; integration-manifest.json carries the handoff).

### Audit + history
- Every mutation returns its `AuditEvent`s from inside the `with_audits` closure, so
  audit rows commit atomically with the mutation. Test reads back all seven
  `org_change.*` actions from `audit_events` and asserts exactly 4
  `gov_approvals(kind='org_change_step')` rows — the DB-level approver<>requester
  CHECK (0153) is exercised as the second SoD net (`UNIQUE (org_id, request_ref)`
  additionally forbids double-recording a step).
- `org_change_events` append-only history is asserted for create/preflight/
  draft.update/submit/step.decide/effectuate.

### Fail-closed gates (all test-proven)
- submit before preflight → 409; blockers (REORG deactivation with live resident
  user) hold DRAFT and refuse submit; submit **recomputes** preflight in-transaction
  (never trusts the stored receipt; staleness flag verified after a draft edit).
- SoD: self-approval → 409, out-of-order step → 409, re-decide → 409; reject →
  REJECTED; revision only via `supersedes_id` of a REJECTED request (non-rejected →
  409).
- Effective-date: past date refused at the draft door (422); fully-approved request
  refuses effectuate before 발효일 (409, KST).
- Dissolve: effectuate opens the six settlement items instead of applying; archive is
  double-gated — unsettled items → 409, then the deferred ops re-run the referential
  guards so a checked-off settlement with a still-active resident user → 409; only
  after the dependent is genuinely cleared does archive apply the deactivation
  (readback-asserted `deactivated_at`).
- Terminal states: FOR UPDATE row lock + FSM `can_transition_to` + the DB
  `org_change_terminal_immutable` trigger as third net; double-effectuate → 409.

### Idempotency
- `Idempotency-Key` required (422 absent); replay with a byte-equal body fingerprint
  returns the stored request (200, same id — test-proven), changed body under the
  same key → 409. Fingerprint is computed over the re-serialized DTO (SHA-256), so
  whitespace differences don't break replay. The lookup runs inside the armed
  transaction — org-scoped by RLS, matching the `UNIQUE (org_id, idempotency_key)`
  constraint.

### Rejection-class checks
- Error envelope `{"error":{"code","message"}}` matches the production/rest exemplar;
  list endpoint is 2 queries (page + count), detail is 4 constant queries — no N+1;
  filters are validated enums (422 on unknown); limit/offset clamped server-side;
  extractor rejections behave identically to the exemplar crates (bare axum
  extractors, parity confirmed).
- Zero TODO/FIXME/unimplemented!/dbg!/test-skips in any lane-owned file (grep-clean).
- No fabricated data: headcount is a real scoped COUNT; site/team counts are
  deterministic derivations of the proposal; the two review-chip warnings
  (OPEN_DOCS_REVIEW / FREEZE_WINDOW_REVIEW) deliberately carry `count: null` —
  reminders, not computed claims (registered follow-up).
- Wire dates: `effectiveDate` pinned to ISO `YYYY-MM-DD` on both serialize and
  deserialize (the workspace `time` build has no serde-human-readable; a locking unit
  test guards the pin).

## Findings (fixed in this stage)

1. **Code-sequence lexicographic max — permanent create outage at OC-YYYY-10000**
   (`adapter-postgres/src/lib.rs`, `create`). The next display code was derived from
   `ORDER BY code DESC LIMIT 1` + parse: as text `OC-2026-9999 > OC-2026-10000`, so
   after request 9999 the generator would recompute 10000 forever and every create in
   that org-year would 409 on `UNIQUE (org_id, code)`. Fixed with a numeric max
   (`max((substring(code FROM 9))::bigint)`).
2. **Idempotency-key length validated in bytes, DB CHECK in characters.** A short
   multibyte key (e.g. 8 Hangul chars = 24 bytes) passed the app gate, hit the DB
   `char_length` CHECK, and surfaced as a 409 with the unrelated self-approval
   message. Fixed by validating `chars().count()`; the DB 23514 mapping is now
   genuinely reachable only by the gov_approvals self-approval net (which the app
   also pre-checks with the same message).
3. **`/org-entities` positive path untested** — only the fail-closed empty case was
   proven, which a permanently-dead endpoint would also pass. Added group + membership
   + grant seeding (owner pool; `mnt_rt` correctly cannot write those tables) and
   asserted the granted 법인 appears while the grant-less outsider still gets `[]`.

## Pre-existing, out-of-lane observations (NOT fixed here)

- `cargo fmt --all --check` is dirty in four upstream spine files
  (`app/tests/cedar_freshness_mint.rs`, `app/tests/logistics_pilot_story.rs`,
  `platform/auth-rest/src/lib.rs`, `platform/auth/tests/jwt_es256.rs`) from commits
  `b0d31af8`/`4a85405c` — in-flight codex writer territory, outside this lane's
  roots. Lane-owned files are rustfmt-clean (verified file-scoped).
- The stage-2 report's "cargo fmt --check: clean" claim did not hold workspace-wide;
  it holds for lane-owned files.

## Honest gaps / follow-ups (design-contract deltas, documented, not silent)

- **Per-step approver-role binding is not enforced**: the contract (design-contract.md
  §3) says the approver should hold the step's role feature (hr/finance/legal/
  executive); the platform has no such role/feature vocabulary yet, so any
  approve-floor principal (EXECUTIVE/SUPERADMIN) can decide every step. The ordered
  chain, SoD approver≠drafter (app + DB CHECK), and one-decision-per-step remain
  enforced. Needs the integrator's Feature work (or step-role features) — added to
  open_items.
- Apply executor performs op SQL inline (same guards identity/registry enforce, DB
  constraints as second net) instead of calling their application commands — a
  deliberate, module-doc'd divergence because those commands each open their own
  audited transaction and would break single-transaction apply. The manifest's
  `hot_crate_notes` still describes the scout-stage plan (commands) — superseded by
  the module doc.
- Preflight open-docs/freeze-window signals are uncounted reminder chips (slice-1
  scope per scout gap-analysis); first-class org_units, per-person lifecycle events
  for REASSIGN_ORG_UNIT, and effective-date automation remain registered follow-ups.
- `org_change_approval_steps` decided rows have no DB-level immutability trigger
  (app logic + `gov_approvals` UNIQUE are the nets); acceptable, noted for parity
  with the requests table's terminal trigger if hardening is wanted later.

## Commands run (this stage, from the committed tree)

- `rustfmt --check` on all four lane-owned files — clean.
- `cargo clippy -p mnt-orgchange-domain -p mnt-orgchange-adapter-postgres
  -p mnt-orgchange-rest --all-targets -- -D warnings` — clean.
- `cargo test -p mnt-orgchange-domain` — 8/8 pass.
- `cargo test -p mnt-app --test org_change_api` — 3/3 pass (runtime role `mnt_rt`,
  fresh scratch DBs through the full migration chain, assembled `build_router`).
