# CAP-BOARD-CONSOLE — STAGE 3 backend verification (fresh-eyes adversarial)

Verified 2026-07-24 against the actual code in this worktree (branch
`claude/console-board-backend-20260724`), not the stage-2 build report.
Verifier: independent stage-3 lane. Two defects found; both FIXED in commit
`6565b268` and re-verified green.

## Verdict

GO for consolidation, with the already-declared integrator items (openapi
fragment merge + client regen, migration renumber) and the pre-existing spine
blockers unchanged.

## Findings (both fixed)

### F1 — publish was two transactions; failure between them bricked the notice (FIXED)

`PgNoticeStore::publish` ran as two sequential `with_audit` transactions:
(1) FOR UPDATE guard + empty-audience EXISTS preflight + NT- code + status
flip, then (2) the receipts snapshot. A crash or transient DB error between
them left a notice permanently `published` with **zero receipts**: every ack
404s, progress reads 0/0, and republish is dead on the 409 already-published
guard — no repair path short of SQL. The preflight also did not hold
transactionally (audience membership could change between the two
transactions, re-creating the "silent empty snapshot" the preflight exists to
prevent). Terminal-state write-race class.

Fix: merged into ONE `with_audits` transaction (the platform helper built for
exactly this — multiple audit events, one atomic commit): lock → snapshot
INSERT..RETURNING (its returned rows replace the EXISTS preflight: exact,
fail-closed) → `issue_code` → status flip → both `notice.publish` and
`notice.publish_recipients` audit events recorded in the same commit. Any
failure rolls the entire publish back. Notification fan-out stays best-effort
post-commit (receipts are the record). Net: −2 queries, −54 lines.

Pinned by: `notices_rls_surfaces_as_runtime_role.rs` — a rejected
empty-audience publish now provably leaves status `draft`, **no NT- code**,
and **zero receipt rows** (atomic rollback, asserted as `mnt_rt`).

### F2 — progress fabricated 0/0 for a nonexistent notice (FIXED)

`GET /api/v1/notices/{id}/progress` returned `200 {total:0, acknowledged:0}`
for any random UUID, while `backend/openapi/openapi.yaml` (line ~13375)
already declares `404` on the route and `list_receipts` performs the
existence check. Fabricated-data / spec-contradiction class. Fix: same
fail-closed existence check as `list_receipts` → 404.

Pinned by: `rest/tests/api.rs` — ghost-id progress asserts 404 over the real
router with ES256 tokens.

## Adversarial checklist — verified against code, with evidence

- **FORCE RLS + org policy on every new table**: `notice_audience_branches`
  (migration 0197) has ENABLE + FORCE RLS + `org_isolation` USING/WITH CHECK
  on `app.current_org`; `notices`/`notice_receipts` already forced in 0162;
  `user_branches` (new `GRANT SELECT ... TO mnt_rt`) is in the 0035 FORCE-RLS
  rollout list — the grant does not open a cross-tenant read. Composite
  same-org FKs `(notice_id, org_id)`/`(branch_id, org_id)`; explicit
  `REVOKE UPDATE` ordered after CREATE TABLE so it also wins over the 0031
  default-privileges auto-grant in production.
- **Tests genuinely run as `mnt_rt`**: both adapter RLS tests and the
  app-level `board_ack_api` build their pools with `SET ROLE mnt_rt` in
  `after_connect` (NOSUPERUSER/NOBYPASSRLS), seeding via the owner pool only.
  Cross-tenant proof is count-based, not error-based: other-org GUC sees
  `COUNT(*) = 0` on `notice_audience_branches` and an empty notices list —
  no count leak. GUC arming verified in the helpers themselves:
  `with_audit`/`with_audits`/`with_org_conn` bind transaction-local
  `app.current_org` before the closure runs (audit_tx.rs).
- **Deny-by-default authz without leakage**: `Feature::NoticeManage`
  ([D,D,D,A,A,A]) via `authorize_org_wide`; `is_notice_manager` treats ANY
  authz error as non-manager. Drafts: 404 + list omission for non-managers
  (existence isolation); manager-only aggregates (progress/receipts): 403;
  ack for a non-recipient: 404, never 403. All asserted over the real
  per-crate router AND the assembled `build_router` app (incl. forged-key 401
  before any data access).
- **Audit event per mutation + readback**: `notice.create_draft`,
  `notice.update_draft`, `notice.publish`, `notice.publish_recipients`,
  `notice.acknowledge` — `board_ack_api` reads all five back from
  `audit_events`, and pins BOTH idempotent ack attempts audited to the
  recipient (count = 2).
- **Fail-closed gates**: `NoticeAudience::new` rejects unknown scope and
  branch_ids/scope incoherence (non-empty iff `branches`); unknown category
  422; foreign/unknown branch id surfaces as 422 validation via the composite
  FK (23503 mapped), not a 500; empty effective audience rolls the publish
  back (422). REST 422 matrix asserted.
- **Idempotency replay returns the stored outcome**: ack is
  `COALESCE(acknowledged_at, $3)` — second ack returns 204 and preserves the
  first timestamp (asserted twice in two suites); publish replay is 409 under
  the FOR UPDATE guard. No `Idempotency-Key` header — none of the notices
  routes declare one in openapi.yaml (checked), consistent with the scout
  contract.
- **N+1 / repeated-query parsing**: list/get/summary hydration is ONE query
  (`SUMMARY_SELECT` + two LEFT JOIN LATERAL for audience names and progress,
  one LEFT JOIN for the viewer receipt); `AssertSqlSafe` composes only
  compile-time constants, never request text. Receipts drill: existence check
  + count + page (3 fixed queries, pager-truth `total`).
- **Canonical error envelope**: `{"error":{"code","message"}}` — byte-shape
  match with the repo-wide `ErrorPayload` pattern (analytics-quant, benefit,
  comms, compliance, dispatch, …); DB errors log server-side and return an
  opaque `internal` (no sqlx/schema leakage).
- **Terminal-state write races**: draft edit takes `FOR UPDATE` and 409s once
  published; publish takes `FOR UPDATE` and is now fully atomic (F1); the
  frozen-after-publish rule asserted at adapter, REST, and app level.
- **Design-contract fidelity** (design-contract.md §1–§5 vs code): FSM
  one-way draft→published, NT- code at publish, audience frozen, receipts
  immutable record — all implemented; DTO field names/types match the §4
  extract and the openapi fragment; publish preflight semantics per §1
  (409 non-draft, 422 empty audience); progress 404 now matches §3's declared
  response set. Korean label mapping is frontend-owned (§4) — correctly
  absent from the backend.
- **No stubs/placeholders**: zero TODO/FIXME/unimplemented!/todo!/skip/ignore
  in the four notices crates + board_ack_api (one doc-comment *reference* to
  the design mirror's TODO.md file — not a code placeholder).

## Verification runs (all in this worktree, shared dev postgres scratch DBs)

| Suite | Result |
|---|---|
| `cargo fmt --check` (4 notices crates) | clean |
| `cargo clippy --all-targets -- -D warnings` (4 notices crates) | clean |
| `cargo test -p mnt-notices-domain` | 6/6 |
| `cargo test -p mnt-notices-adapter-postgres` (RLS as `mnt_rt`) | 2/2 |
| `cargo test -p mnt-notices-rest` (real router, ES256) | 2/2 |
| `cargo test -p mnt-app --test board_ack_api` (assembled router, `mnt_rt`) | 1/1 |
| `cargo test -p mnt-app --test openapi_drift` | 4/5 — the single documented expected-red: `/api/v1/notices/{id}/receipts` not yet in openapi.yaml (integrator collision root; fragment in `manifests/openapi-fragment.yaml`) |

## Open items (unchanged from the build report, still true)

1. INTEGRATOR: merge `manifests/openapi-fragment.yaml`, regenerate + commit
   `clients/{ts,kotlin,swift}` → clears the one red drift assertion.
2. INTEGRATOR: renumber provisional migration 0197 to the next free slot.
3. SPINE (pre-existing, not this lane's): 0170 migration-number collision;
   compile breakage in mnt-platform-auth / logistics adapter /
   production-rest / facilities-rest at spine HEAD. This worktree carries
   LOCAL UNCOMMITTED mechanical workarounds for verification only
   (catalogued in `integration-manifest.json`); owning lanes must land real
   fixes.
4. STAGE 3 frontend lane: build `web/src/console/board/**` per
   design-contract.md §6 against the extended NoticeSummary + PATCH/receipts.
