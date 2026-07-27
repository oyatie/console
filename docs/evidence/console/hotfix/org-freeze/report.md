# Hotfix — org 변경 동결창 (change freeze window) enforcement on effectuate

Lane `hf-org-freeze` · worktree `hf-org-freeze-20260725` · 2026-07-25

## 1. The defect

DESIGN.md §3.9.1 lists **변경 동결 창(freeze window)** as one of the six
governance mechanisms every org change is subject to:

> **변경 동결 창(freeze window)**: 급여 마감·회계 결산 중 조직 변경 제한.

The org-change engine implemented five of the six. The freeze window existed
only as an **advisory chip pushed unconditionally on every preflight**
(`compute_preflight`, `FREEZE_WINDOW_REVIEW` — its own comment read *"imperative
reminders, not computed claims"*), and **no gate at all on the apply path**:

- `PgOrgChangeStore::effectuate` gated on status (`APPROVED`) and on the
  effective date (`today_kst() < effective_date`) — nothing else.
- `PgOrgChangeStore::archive` replays the deferred DISSOLVE ops (branch/region
  deactivation) and had the same hole.

So an approved org change could be applied with an effective date inside a
closed payroll or accounting period, rewriting the scope and attribution of a
run that is already sealed. Proven, not asserted — see §5 (red run).

## 2. The mechanism reused (no second freeze concept)

`period_locks` (migration `0107_create_period_locks_versioning_lifecycle.sql`)
already is the platform's freeze-window enforcement: an append-only row with
`unlocked_at IS NULL` closes `[period_start, period_end]` for one domain
(`payroll` | `accounting`), FORCE RLS + `org_isolation`, one-shot-unlock
trigger, `GRANT SELECT, INSERT, UPDATE … TO console_rt`. Its Rust half is
`console_platform_db::period_lock::{assert_period_open, assert_period_open_range}`,
already consumed by `console-financial-adapter-postgres` (cost ledger) and
`console-workflow-adapter-postgres` (payroll draft drain).

This lane **adds no table, no migration, no new error vocabulary and no second
lock concept** — it calls the existing guard.

> **Superseded by §7 F-2.** This section originally reused the platform's English
> refusal (`"<domain> period <start>..<end> is locked; write dated … refused"`)
> verbatim. Stage-2 verification found that string rendering raw in the Korean
> approval modal, so the refusal and the preflight chip are now Korean in this
> crate. The platform helper and the financial surface are unchanged.

## 3. What changed

`backend/crates/orgchange/adapter-postgres/src/lib.rs`

1. **`assert_change_window_open(tx, effective_date)`** — checks
   `assert_period_open` for **both** freeze domains (§3.9.1 names 급여 마감 *and*
   회계 결산) inside the caller's already-armed transaction, so the lookup is
   RLS-scoped to the caller's tenant and a refusal rolls the whole apply back.
2. **Called from `apply_ops`**, which is the single function both live-apply
   entry points route through (`effectuate` for NEW/REORG, `archive` for the
   deferred DISSOLVE ops) — the root-cause placement, not a per-caller patch.
   Plus one explicit call in `effectuate`'s `Dissolve` arm, which opens
   settlement without going through `apply_ops`.
3. **`PgOrgChangeError::Frozen(KernelError)`** — a typed variant distinct from
   `Domain`, so the refusal is recognisable to the caller. It maps through the
   existing `kind()`/`message()` surface to the canonical envelope
   `409 {"error":{"code":"conflict","message":"…"}}` naming the blocking domain
   and window. No REST change, no openapi change (status code and envelope are
   unchanged; `409` was already documented for this route).
4. **`record_freeze_refusal`** — the refused business transaction rolls back and
   takes every audit row inside it with it, so the §3.10-⑥ detection record is
   committed in its **own** transaction: action
   `org_change.effectuate.refused` / `org_change.archive.refused`,
   `target_type = org_change_request`, `anomaly = true`, `reason` = the refusal
   message. Anything but a `Frozen` outcome passes through untouched.
5. **`compute_preflight` freeze signal is now computed.** The unconditional
   `FREEZE_WINDOW_REVIEW` chip is replaced by a real read of the same period
   locks for the request's effective date; the warning appears only when a lock
   genuinely covers it, and its label names every blocking domain — **one** chip
   however many domains block (§7 F-1 amends this from one-per-domain). It stays a *warning*
   rather than a *blocker* because a lock can be lifted before the effective
   date arrives — the hard stop lives at effectuate (§3.10 예방 gate).
   A non-conflict error from the lock read propagates; it is not swallowed into
   a warning.

## 4. Legitimate paths preserved

- Effective date outside every active lock → applies, **even while the tenant
  has other active locks** over different periods (asserted).
- Another tenant's lock over the same window is invisible: the read is
  RLS-scoped (asserted with a second org's active lock covering today).
- Status/effective-date/SoD gates are unchanged; a refused apply leaves the
  request `APPROVED` and retryable.

## 5. Evidence

All backend assertions execute through the assembled HTTP router against a
**runtime-role (`console_rt`) pool** — `runtime_role_pool()` issues `SET ROLE
console_rt` on every connection and `app_state` is built from it, so RLS is real
and superuser BYPASSRLS cannot mask it. Superuser is used only to seed fixtures.

Harness: disposable PostgreSQL 18.4 container with the repo's own topology
bootstrap (`ops/postgres-reconcile-topology.sh`), `console_buck_admin` +
`mnt.sqlx_test_bootstrap=buck-sqlx-superuser-v1`, `SQLX_OFFLINE=true`. The
shared dev stack was not touched.

### Red — the defect reproduced

With `assert_change_window_open` neutered to `Ok(())` and nothing else changed:

```
test effectuate_is_frozen_inside_a_locked_period_and_records_the_attempt ... FAILED
  left: 200, right: 409
  → "status":"APPLIED" … "action":"effectuate","fromStatus":"APPROVED","toStatus":"APPLIED"
```

i.e. the org change applied cleanly **inside an active payroll lock**.

### Green

```
cd backend && SQLX_OFFLINE=true <pg-harness> cargo test -p console-app --test org_change_api
running 4 tests
test authorization_denies_without_leakage_and_conceals_other_tenants ... ok
test dissolve_settles_then_archives_with_referential_net ... ok
test effectuate_is_frozen_inside_a_locked_period_and_records_the_attempt ... ok
test reorg_lifecycle_runs_draft_to_applied_with_ordered_sod ... ok
test result: ok. 4 passed; 0 failed
```

`effectuate_is_frozen_inside_a_locked_period_and_records_the_attempt`
(`backend/app/tests/org_change_api.rs`) asserts, as `console_rt`:

| # | Assertion |
|---|---|
| 1 | With **both** domains locked, preflight surfaces exactly **one** `FREEZE_WINDOW_REVIEW` warning naming both (`급여 마감·회계 결산`) — computed from the locks, not pushed, and not one row per domain (see §7 F-1) |
| 2 | `POST …/effectuate` inside an active **payroll** lock → `409`, message names `급여 마감` and the effective date |
| 3 | Same for an active **accounting** lock (`회계 결산`) |
| 4 | The request stays `APPROVED` after each refusal |
| 5 | No org row was written (`branches` count 0) and **zero** `org_change.effectuate` audit rows |
| 6 | Exactly two `org_change.effectuate.refused` audit rows, each `org_id` = KNL, `actor` = the applying executive, `anomaly = true`, `reason` naming the blocking domain |
| 7 | With both locks lifted but an own-tenant lock over a *past* month still active **and** a foreign tenant's lock covering today still active → `effectuate` returns `200 APPLIED` and the region really renamed |
| 8 | A fresh draft whose effective date falls outside those still-active locks carries **no** freeze warning |

`dissolve_settles_then_archives_with_referential_net` gained the sibling proof:
a lock opened after effectuate refuses `archive` with a `409` naming
`회계 결산`, the branch stays active, one `org_change.archive.refused` audit
row lands, and after unlock the archive applies the deferred deactivation.

### Gates

```
cargo fmt --check                                          # clean (whole workspace)
cargo clippy -p console-orgchange-adapter-postgres \
             -p console-orgchange-domain -p console-orgchange-rest \
             --all-targets -- -D warnings                  # clean
cargo test -p console-orgchange-domain                         # 8 passed
cargo run -p console-gate-audit-coverage                       # PASSED
cargo run -p console-gate-tenant-isolation                     # PASSED
cargo run -p console-gate-rls-arming                           # PASSED
cargo run -p console-gate-dev-auth-absence                     # PASSED
```

## 6. Honest gaps

- **Pre-existing, not this lane:** workspace-wide
  `cargo clippy --all-targets -- -D warnings` fails on
  `backend/crates/dispatch/application/src/lib.rs:267`
  (`clippy::double_must_use` on `DispatchQueueCursor::encode`). Byte-identical
  on the integration spine, so it is not a wave-4 regression — but CI's clippy
  step is workspace-wide and will be red until someone owning that crate drops
  the redundant `#[must_use]`. Out of this lane's roots.
- **`OPEN_DOCS_REVIEW` is still an unconditional reminder chip.** The sibling
  §3.9.1 signal has no existing mechanism to reuse (there is no "open
  approvals/postings" query surface yet), so it was left as found rather than
  fabricated. Registered follow-up.
- **No Buck2 run.** Verified with cargo; `buck2 test //backend/...` was not
  executed in this lane.
- **`docs/evidence/console/CAP-ORG-CONSOLE/backend-verification.md:101`** still
  describes `FREEZE_WINDOW_REVIEW` as a `count: null` reminder chip. That file
  is another lane's root; the description is now stale for the freeze half.
- **Frontend untouched.** The console renders `error.message` and the preflight
  warnings as-is, so the refusal and the computed warning surface without a web
  change; no Korean-localised freeze copy was added (the shared platform message
  is English, matching the financial surface). If a Korean-only console string
  is required, that is a follow-up in the web lane, not a backend change.
- **No migration, no openapi/client change** — deliberately. Nothing was
  emitted to the integrator manifests.

---

## 7. Stage-2 adversarial verification (fresh eyes, 2026-07-25)

Re-derived against the code, not the §1–§6 narrative. Harness rebuilt
independently: disposable `postgres:18.4` container + `ops/postgres-reconcile-topology.sh`,
`console_buck_admin` + `mnt.sqlx_test_bootstrap=buck-sqlx-superuser-v1`, `SQLX_OFFLINE=true`,
shared dev stack untouched. (The Docker VM was out of disk — 677 dangling anonymous
volumes left behind by `--rm` postgres harnesses; removed, no named volume touched.)

### 7.1 The claim held

- **Root cause, not symptom — confirmed.** `apply_ops` is genuinely the single
  function both live-apply entry points route through (`effectuate` NEW/REORG at
  `:1611`, `archive` at `:1793`), and the DISSOLVE arm that opens settlement outside
  it takes the gate explicitly (`:1628`). No third live-apply path exists inside the
  crate.
- **The mechanism is the platform's.** `assert_period_open` runs on the caller's
  `with_audits` transaction, which arms `app.current_org` before the closure
  (`audit_tx.rs:119`), so `period_locks`' FORCE-RLS `org_isolation` scopes the read.
  Effective-date semantics match the only other consumer,
  `console-financial-adapter-postgres` (`lib.rs:1254`, cost-ledger entry date).
- **RLS is real in the test.** `runtime_role_pool` issues `SET ROLE console_rt` per
  connection and the router is built from it; the superuser pool only seeds fixtures.

### 7.2 Red proof, re-run by the verifier (not taken on trust)

| Mutation | Result |
|---|---|
| Whole adapter reverted to `885e0b52~1`, test kept | `effectuate_is_frozen…` FAILED, `dissolve_settles…` FAILED (**200/`ARCHIVED`** — the branch deactivated inside an active accounting lock) |
| Only `assert_change_window_open` neutered to `Ok(())` | both FAILED, `left: 200 right: 409` on **both** apply paths |
| Only the single-chip collapse reverted (one warning per domain) | `effectuate_is_frozen…` FAILED, `left: 2 right: 1` |
| Unmutated | 4 passed / 0 failed |

### 7.3 Findings fixed in this stage

- **F-1 · The computed warning emitted one row per blocking domain.** With both a
  payroll and an accounting lock covering the effective date, `compute_preflight`
  pushed **two** warnings with `code: "FREEZE_WINDOW_REVIEW"`. The console renders
  `report.warnings.map(w => <div key={w.code}>)` (`web/src/console/org/OrgChangeModal.tsx:408`),
  so the second row is a duplicate React key, not a second signal — a defect the
  build stage introduced and no assertion covered. Now one chip naming every
  blocking domain, with a regression assertion that goes red on the old shape.
- **F-2 · Both user-visible strings were English inside a Korean modal.** The chip
  read `발효일 동결 — 적용 불가: payroll period 2026-07-22..2026-07-28 is locked; write
  dated … refused`, and `orgApi.ts:50` hands `error.message` straight to
  `OrgChangeModal.tsx:301`, so the 409 rendered the same raw platform sentence beside
  this crate's own Korean refusals (`발효일 이전에는 적용할 수 없습니다.`). §6's
  "reuse the platform phrasing" call is **deliberately overridden** for this crate
  only: the chip is now `급여 마감·회계 결산 동결 — 발효일 조정 필요` and the refusal
  `회계 결산 기간에 포함된 발효일(2026-07-25)에는 조직 변경을 적용할 수 없습니다.`
  The financial surface keeps its English message; nothing shared was touched. Reverse
  this if a single cross-surface wording is wanted instead.
- `FREEZE_DOMAINS` is now one list feeding both the chip and the gate, so the two can
  no longer check a different set of domains.

### 7.4 Checked and found sound (no change)

- `record_freeze_refusal` correctly commits in a second transaction; `with_audit` arms
  RLS from `event.org_id`, and the test proves `org_id`/`actor`/`anomaly`/`reason`.
- `complete_settlement_item` is deliberately **not** gated: it writes only
  `org_change_settlement_items`, and gating it would make a dissolve unclearable
  during a close.
- `effectuate`/`archive` carry no idempotency key, so a refusal cannot poison a
  replay; the request stays `APPROVED` and retryable.
- Ownership: the lane touched only `backend/crates/orgchange/**`,
  `backend/app/tests/org_change_api.rs`, `docs/evidence/console/hotfix/org-freeze/**`.
  No shared collision root, no migration, no openapi/client change, no manifest owed.
  `409` was already documented for `effectuateOrgChange` (`openapi.yaml:16447`) and
  `archiveOrgChange` (`:16525`).
- No TODO/FIXME/stub/`#[ignore]`/dead control in the lane diff.

### 7.5 Open, out of this lane's roots

- **`console-identity-rest` bypasses the freeze entirely.** `POST/PATCH/DELETE
  /api/v1/regions` and `/api/v1/branches` (`crates/identity/rest/src/lib.rs:236-247`)
  mutate the same org tree with no effective date, no SoD chain and no period-lock
  check — `assert_period_open` has zero hits in `crates/identity/**`. §3.9.1 is now
  enforced on the governed path and still open on the ungoverned one (which §3.9.3
  already calls an anti-pattern, 라이브 직접 편집). Needs an owning lane.
- **`openapi_drift` is red on this branch, and it is a one-character bug.**
  `backend/app/tests/openapi_drift.rs:446,484,487,502,505,521,524` search for
  `"…:\\n"` — a literal backslash-`n`, never a newline — so
  `openapi_documents_evidence_register_snapshot_and_evidentiary_contract` can never
  match, although `/api/v1/evidence/objects:` is present at `openapi.yaml:12874`.
  Introduced by `e1ee2199 feat(docs): expose evidence snapshot contract`, not by this
  lane. 12 passed / 1 failed.
- **Workspace clippy is red, pre-existing**: `clippy::double_must_use` on
  `crates/dispatch/application/src/lib.rs:267`. Verified byte-identical to the
  pr488 spine.
- **Semantics worth a founder call:** the gate keys on the *effective date*, matching
  the financial precedent, so a close covering *today* does not block a change whose
  effective date sits in an open period. The literal §3.9.1 reading ("급여 마감 **중**")
  could instead freeze all org changes while any close is in progress. The
  effective-date reading is the one that protects sealed data and the one the platform
  already uses; flagged, not changed.
- **TOCTOU ceiling (platform-wide, not this lane):** at READ COMMITTED a lock
  committed microseconds after the check still lets the in-flight apply through.
  Same property as the cost ledger; would need SERIALIZABLE or an advisory lock.
- `docs/evidence/console/CAP-ORG-CONSOLE/backend-verification.md:101` and
  `design-spec.md:72` still describe `FREEZE_WINDOW_REVIEW` as a `count: null`
  reminder chip with the old label. Another lane's root.
- `OPEN_DOCS_REVIEW` remains an unconditional reminder — unchanged, and honest about it.
- No Buck2 run; cargo only.

### 7.6 Stage-2 gate re-run

```
cargo test -p console-app --test org_change_api        4 passed / 0 failed
cargo test -p console-orgchange-domain                 8 passed / 0 failed
cargo fmt --check (whole workspace)                clean
cargo clippy -p console-orgchange-adapter-postgres \
             -p console-orgchange-domain \
             -p console-orgchange-rest --all-targets -- -D warnings   clean
cargo run -p console-gate-audit-coverage               PASSED
cargo run -p console-gate-tenant-isolation             PASSED
cargo run -p console-gate-rls-arming                   PASSED
cargo run -p console-gate-dev-auth-absence             PASSED
```
