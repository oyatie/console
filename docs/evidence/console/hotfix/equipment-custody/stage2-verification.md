# Stage 2 — fresh-eyes adversarial verification

**Lane** `hf-equipment-custody` · branch `claude/hf-equipment-custody-20260725`
· verified against `c08a12db` (build-stage tip) · 2026-07-25.

Verifier did not write the stage-1 code. Every claim below was re-derived from
the tree and from executed commands, not from `report.md`.

Harness: a disposable `postgres:18.4`
(`sha256:65f70a15…`) bootstrapped exactly the way
`tools/buck/test_needs_postgres.sh` does it — `ops/postgres-reconcile-topology.sh`
from a `mnt_buck_admin` cluster admin, `DATABASE_URL` carrying
`options[mnt.sqlx_test_bootstrap]=buck-sqlx-superuser-v1`. Every HTTP assertion
crosses the assembled router on a `SET ROLE mnt_rt` pool
(`runtime_role_pool`). The shared `mnt-dev` stack was never touched and
`dev-up` was never cycled.

## 1. Was the reported defect real, and is the fix at the root cause?

**Yes to both, reproduced independently.**

Reverting the four changed files to the spine tip and re-running:

```
$ git checkout 4cabe239 -- backend/crates/equipment/{adapter-postgres,application,rest}/src/lib.rs \
                           backend/app/tests/equipment_3r_api.rs
$ cargo test -p mnt-app --test equipment_3r_api
thread 'resale_disposition_sells_unit_and_blocks_further_quotes'  panicked at :706:5  left: 500  right: 200
thread 'repair_lifecycle_completes_with_audits_history_and_no_finance_posting' panicked at :166:5  left: 500  right: 200
test result: FAILED. 2 passed; 2 failed
```

Byte-identical to the reproduction in `report.md §1`. Root cause confirmed by
reading `0184_create_docs_equipment_handover_custody.sql`: it `DROP COLUMN
handover_evidence_reference`s and adds `handover_evidence_object_id UUID` with
`FOREIGN KEY (handover_evidence_object_id, org_id) REFERENCES
docs_evidence_objects(id, org_id)`, while `PgEquipment3rStore::handover_case`
and `case_detail_tx` still named the dropped column. A repo-wide grep now finds
**zero** remaining references to `handover_evidence_reference` outside 0182/0184
themselves. The fix is at the column, not at the symptom.

`backend/crates/logistics/**` keeps its own unrelated `evidence_reference`
string (`openapi.yaml:14322`); it is a different module and correctly untouched.

## 2. Do the tests genuinely fail without the fix?

Four mutations, each applied to the committed tree, run, then reverted with
`git checkout HEAD --`. The tree was verified clean after each.

| # | Mutation | Result | What it proves |
|---|---|---|---|
| M-revert | all four files → `4cabe239` | 2 passed / **2 FAILED** (500 vs 200) | the historical red is real and is exactly the handover POST |
| M-relax | strip the eligibility predicate from `bind_handover_custody`, leaving `o.id=$4 AND c.copy_kind='ORIGINAL'` | **FAILED** — `mutable must be concealed … left: 500 right: 404` | the predicate is load-bearing; and the trigger really is the independent second authority (it raised where the query admitted) |
| M-null | `case_detail_tx` returns `"evidenceObjectId": Value::Null` | **FAILED** at `:187` | the read-path mapping is asserted, not assumed |
| M-nobind | delete the `bind_handover_custody(…)` call from `handover_case` | **2 FAILED** — concealment at `:928`, `handover must bind exactly one immutable custody row: RowNotFound` at `:206` | the custody write itself is load-bearing |

**The foreign-tenant leg is proven to be RLS, not the trigger.** Under M-relax
the four rejection classes split: `foreign-org` still returned **404** while
`mutable`, `disposed` and `non-admissible` became **500** (the trigger raising
`23514`). Only row-level security can produce zero rows for an object that is
fully eligible in its own tenant — the org-2 fixture is seeded with the *same*
`(admissible, not-disposed, verified-WORM)` triple as the passing one and its
`worm_status` promotion is asserted. A second, independent cross-org guard also
exists: under M-nobind the foreign-org handover still failed, this time on the
case column's composite FK to `docs_evidence_objects(id, org_id)`.

## 3. Defects found and fixed by this stage

**F1 — the positive half of the custody claim was never asserted.**
Stage 1 proved four refusals and one 403 with `docs_equipment_handover_custody`
empty, but nothing asserted that a *permitted* handover writes a custody row, or
what that row contains. `bind_handover_custody` could have resolved the wrong
copy, written the wrong branch or the wrong actor and stayed green.

**F2 — the read path was entirely unasserted.** No test exercised
`GET /api/v1/equipment-3r/rental-cases/{id}`, so `case_detail_tx`'s new
`evidenceObjectId` projection had no coverage; returning `null` passed (proven by
M-null above).

Both fixed in `repair_lifecycle_completes_with_audits_history_and_no_finance_posting`
— the natural home, since it already owns the successful handover and already
makes DB-level audit/history assertions. The addition asserts the detail view
surfaces the bound object, and that the custody tuple is exactly
`(evidence_object_id, original_copy_id, branch_id, created_by)` =
`(the object, its verified ORIGINAL copy, the case branch, the acting operator)`.
Both new assertions are proven load-bearing by M-null and M-nobind.

## 4. Enterprise bar

| Requirement | Evidence |
|---|---|
| FORCE RLS org isolation on new tables | no new table; `docs_equipment_handover_custody` (0184) is `ENABLE` + `FORCE ROW LEVEL SECURITY` with an `app.current_org` policy and `REVOKE UPDATE, DELETE … FROM mnt_rt`. `with_audits` arms `app.current_org` before the closure runs (`platform/db/src/audit_tx.rs:111-121`). |
| tested as `mnt_rt`, not as superuser | every HTTP assertion goes through `runtime_role_pool` (`SET ROLE mnt_rt` in `after_connect`). Superuser `pool` is used only to seed and to read back assertions. |
| deny-by-default authorization | `authorize(branch)` runs **before** `bind_handover_custody`; the foreign-branch story asserts 403 *and* that custody stayed empty. |
| audit on every mutation | the custody insert and the FSM transition share one `with_audits` transaction that emits `equipment_3r.case.handover`; the lifecycle story still asserts exactly 8 case audits. `mnt-gate-audit-coverage` PASSED. |
| canonical error envelope | refusals return `KernelError::not_found` → `{"error":{"code":…}}`; observed in the mutation output. |
| idempotency | unchanged from the sibling transitions (`approve`, `dispatch`): the FSM guard `state.can_transition_to` plus `UPDATE … WHERE status=$7` is the replay defence, and 0184 adds `UNIQUE (org_id, equipment_case_id)` on custody. No regression, no new surface. |
| story-level integration test | `backend/app/tests/equipment_3r_api.rs`, 5 stories. |
| no fabricated values | the WORM verification is produced by migration 0195's `docs_evidence_copies_bind_storage_attestation` trigger from a real `evidence_media` attestation; the fixture asserts the trigger's verdict rather than setting the flag. Verified against the migration source and against the sibling fixture at `backend/crates/docs/rest/tests/evidence_rest_rls_surfaces_as_runtime_role.rs`. |
| statutory parameters | none in scope — this lane implements no rule with regulatory parameters. |
| frontend bar | not applicable: **zero** `web/**` diff. |

Stage 1's two departures from `codex/equipment-evidence-custody-20260724` were
re-checked against that branch's own bytes and both hold:

- `8cd1092f` really does add `mnt-docs-adapter-postgres` + `mnt-docs-application`
  to `backend/crates/equipment/adapter-postgres/Cargo.toml`, and
  `Layer::Adapter.allowed_deps()` really is
  `[Application, Domain, Kernel, Platform]`
  (`backend/ci/gates/layer-boundary/src/lib.rs:96-102`) — adapter → adapter is
  forbidden with no exemption.
- its fixture really does write
  `VALUES ($1,…,$7,$8,$9,$8,$8)` where `$8` is the `disposed_by` bind
  (`NULL` unless disposed) into the `NOT NULL` `created_by`/`updated_by`
  columns, and really does bind `worm_status`/`verified_at` directly.

Its docs-crate half was dropped entirely: `git diff 4cabe239 HEAD --
backend/crates/docs` is empty, so no orphaned
`bind_equipment_handover_evidence_tx` was left behind in a shared root.

## 5. Ownership and stubs

- `git diff --name-only 4cabe239 HEAD -- web/ backend/openapi/ clients/ backend/crates/platform/db/migrations/ backend/app/src/ backend/crates/docs/` → **empty**. No shared collision root was touched; the openapi change is a manifest.
- Grep of the added lines for `TODO|FIXME|XXX|HACK|placeholder|unimplemented|stub|#[ignore]|.only(` → **no hits**.
- No dead code introduced: `evidence_reference()` was deleted with its last caller; every new helper has a caller (clippy `-D warnings` clean).
- Fixture identifiers respect the real column shapes —
  `equipment_no ~ '^[A-Z]{3}[A-Z0-9]{2}-[0-9]{4}$'` (`EVD01-0001`) and
  `request_no ~ '^[0-9]{8}-[0-9]{3}$'` (`20260725-001`), both verified against
  `0007`/`0008`.

## 6. Gates re-run at the verified tip

```
cargo test -p mnt-app --test equipment_3r_api                            -> 5 passed; 0 failed
cargo test -p mnt-equipment-{domain,application,rest,adapter-postgres}   -> 7 passed; 0 failed
cargo fmt --all -- --check                                               -> clean
SQLX_OFFLINE=true cargo clippy \
  -p mnt-equipment-{domain,application,adapter-postgres,rest} \
  --all-targets -- -D warnings                                           -> clean
SQLX_OFFLINE=true cargo clippy -p mnt-app --test equipment_3r_api \
  --no-deps -- -D warnings                                               -> no hits in the story test
cargo run -p mnt-gate-layer-boundary     -> PASSED (166 crates, 0 violations)
cargo run -p mnt-gate-audit-coverage     -> PASSED
cargo run -p mnt-gate-rls-arming         -> PASSED
cargo run -p mnt-gate-tenant-isolation   -> PASSED
cargo run -p mnt-gate-dev-auth-absence   -> PASSED
cargo run -p mnt-gate-migration-safety   -> FAILED, PRE-EXISTING
```

`SQLX_OFFLINE=true` is required for clippy (CI sets it,
`.github/workflows/ci.yml:359`): with a live `DATABASE_URL` pointing at the
disposable harness, `mnt-platform-db`'s compile-time `sqlx::query!` macro tries
to prepare `INSERT INTO audit_events` against a database that has no migrations
applied and fails to compile. That is a harness interaction, not a code defect.

`migration-safety` is red for `[NonContiguousMigrationVersion] missing migration
version 0201 before 0202`. Confirmed pre-existing and not attributable to this
lane: `git ls-tree 4cabe239 backend/crates/platform/db/migrations/` already goes
`0200 → 0202`, and this branch adds and edits no migration. `0201` is the
charter's RESERVED slot (§0).

## 7. Open items this stage did not fix

1. **`backend/openapi/openapi.yaml` + `clients/**` — CLOSED, verified from this
   worktree.** openapi-integrator landed it in the §5.5 order. On
   `wave23-consolidation-20260724` (`33ee3344`): `:14453` requires
   `evidenceObjectId`, `:32564` returns it, `clients/ts/src/schema.d.ts` is
   regenerated, and `:14322` — the *logistics* POD endpoint — correctly still
   carries its own unrelated `^evidence://` string. `c08a12db` is an ancestor of
   the spine, so **only the two stage-2 commits are outstanding** and they touch
   one test file plus this evidence directory.
2. **`web/src/console/equipment/**` is now the last unfixed half, and it has no
   gate.** Re-checked against the spine at `33ee3344`, not against this branch:
   `equipmentApi.ts:83,170` still declare `evidenceReference: string` and
   `EquipmentCaseDetail.tsx:171,334,446` still post and render it. The server
   rejects unknown fields, so the console handover now **422s**, and
   `detail.handover.evidenceReference` is `undefined`. **Nothing catches this:**
   those are hand-written local interfaces, not generated types, so `tsc -b` and
   both api-drift gates stay green while the screen is broken. No live exposure
   (`EXPOSED_SCREEN_KEYS` is `[]`, the screen is DARK), which is the only reason
   this is not a production incident. **For whoever owns it:** the fix is not a
   rename. The server's deliberate flat `not_found` cannot tell an operator *why*
   a document was refused, so the field becomes a picker over Docs/Evidence
   objects that are `ADMISSIBLE`, not disposed, and carry a `VERIFIED` ORIGINAL
   copy — the 404 is a last-resort guard, not the UI's explanation channel.
3. **`docs_equipment_handover_custody` permits one evidence object on many
   cases** — `UNIQUE (org_id, equipment_case_id)` binds a case to one object but
   not an object to one case. Defensible (one signed delivery sheet can cover a
   multi-unit drop) but it is an evidentiary-integrity decision that belongs to
   the Docs owner of 0184, not to Equipment. Recorded, not changed — the
   migration is a shared collision root.
4. **`mnt-gate-migration-safety` red on the spine** (§6) — integrator /
   slot-ledger decision.
5. **A third spine-level red, found by widening the clippy scope.** CI runs
   `cargo clippy --all-targets -- -D warnings` over the whole workspace
   (`.github/workflows/ci.yml:359`) on the pinned `1.96.0` toolchain. That
   command fails today at **13 `clippy::double_must_use` sites** across
   `crates/dispatch/{application,adapter-postgres}`, `crates/production/rest`,
   `crates/facilities/rest`, `crates/attendance/rest`,
   `crates/financial/adapter-postgres`, `app/src/workbench.rs` and
   `app/src/workflow_object_context.rs` — every one of them in a file with a
   **zero diff between `4cabe239` and this branch**, so the verdict is
   attributable to the spine, not to this lane. Stage 1's "clippy clean" claim
   was true for the crate set it named; the equipment crates are clean on their
   own (`cargo clippy -p mnt-equipment-{domain,application,adapter-postgres,rest}
   --all-targets -- -D warnings` → exit 0). Integrator: CI will be red on merge
   until these are cleared.
